use clap::Parser;
use qrcodegen::{QrCode, QrCodeEcc};
use image::{Luma, ImageBuffer, imageops::FilterType};
use std::fs::File;
use std::io::{self, Read, IsTerminal};

#[derive(Parser, Debug)]
#[command(version, about = "Create a QR Code from text file or piped data")]
struct Cli {
    #[arg(short, long, value_name = "ECC", help = "Error correction level. Use L, M, Q, or H.", default_value = "M")]
    ecc: String,

    #[arg(short, long, value_name = "INPUT", help = "Unicode text file or piped data.")]
    input: Option<String>,

    #[arg(short = 't', long, value_name = "OUTPUT_TYPE", help = "Output file/data types. Use Text, SVG or PNG", default_value = "Text")]
    output_type: OutputType,

    #[arg(short = 'o', long, value_name = "OUTPUT_FILE", help = "Output file path only used for PNG.", default_value = "qrcode.png")]
    output_file: String,
}


#[derive(Debug, Clone)]
enum OutputType {
    TXT,
    SVG,
    PNG,
}

impl std::str::FromStr for OutputType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(OutputType::TXT),
            "svg" => Ok(OutputType::SVG),
            "png" => Ok(OutputType::PNG),
            _ => Err(format!("Unknown output type: {}. Use Text, SVG or PNG", s)),
        }
    }
}


fn main() -> io::Result<()> {
    let args = Cli::parse();

    let ecc: QrCodeEcc = match args.ecc.to_lowercase().as_str() {
        "l" => QrCodeEcc::Low,
        "m" => QrCodeEcc::Medium,
        "q" => QrCodeEcc::Quartile,
        "h" => QrCodeEcc::High,
        _ => {
            eprintln!("Invalid error correction level. Use L, M, Q, or H.");
            return Ok(());
        }
    };

    // let mut text = String::new();
    // if let Some(file_path) = args.input {
    // 	let mut file = File::open(file_path)?;
    // 	file.read_to_string(&mut text)?;
    // } else if atty::isnt(atty::Stream::Stdin) {
    // 	io::stdin().read_to_string(&mut text)?;
    // } else {
    // 	eprintln!("No input provided. Please specify a file or pipe data.");
    // 	return Ok(());
    // }

    // Check if there's data in the standard input (pipe)
    // Otherwise, read from a file (atty is unmaintaned and the stable IsTerminal trait should be used now)
    let mut text = String::new();
    if let Some(file_path) = args.input {
        let mut file = File::open(file_path)?;
        file.read_to_string(&mut text)?;
    } else if !io::stdin().is_terminal() {
        io::stdin().read_to_string(&mut text)?;
    } else {
        eprintln!("No input provided. Please specify a file or pipe data.");
        return Ok(());
    }

    // Attempt to encode the text into a QR code
    match QrCode::encode_text(&text, ecc) {
        Ok(qr) => {
            match args.output_type {
                OutputType::TXT => print_qr(&qr),
                OutputType::SVG => println!("{}", to_svg_string(&qr, 4, 10)),
                OutputType::PNG => write_to_png_scaled(&qr, 4, 10, &args.output_file),
            }
        }
        Err(e) => {
            eprintln!("Failed to generate QR code: {}", e);
        }
    }

    Ok(())
}


/*---- Utilities ----*/

// Returns a string of SVG code for an image depicting
// the given QR Code, with the given number of border modules.
// The string always uses Unix newlines (\n), regardless of the platform.
fn to_svg_string(qr: &QrCode, border: i32, scale: i32) -> String {
    assert!(border >= 0, "Border must be non-negative");
    assert!(scale > 0, "Scale must be positive");
    let mut result = String::new();
    result += "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";
    result += "<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd\">\n";
    let dimension = qr.size().checked_add(border.checked_mul(2).unwrap()).unwrap() * scale;
    result += &format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" viewBox=\"0 0 {0} {0}\" width=\"{0}\" height=\"{0}\" stroke=\"none\">\n", dimension);
    result += "\t<rect width=\"100%\" height=\"100%\" fill=\"#FFFFFF\"/>\n";
    result += "\t<path d=\"";
    for y in 0..qr.size() {
        for x in 0..qr.size() {
            if qr.get_module(x, y) {
                if x != 0 || y != 0 {
                    result += " ";
                }
                result += &format!("M{},{}h{}v{}h-{}z", (x + border) * scale, (y + border) * scale, scale, scale, scale);
            }
        }
    }
    result += "\" fill=\"#000000\"/>\n";
    result += "</svg>\n";
    result
}


// Prints the given QrCode object to the console.
fn print_qr(qr: &QrCode) {
    let border: i32 = 4;
    for y in -border .. qr.size() + border {
        for x in -border .. qr.size() + border {
            let c: char = if qr.get_module(x, y) { '█' } else { ' ' };
            print!("{0}{0}", c);
        }
        println!();
    }
    println!();
}


// Writes the given QrCode object to a PNG image with the specified scale and border width.
fn write_to_png_scaled(qr: &QrCode, border: i32, scale_factor: u32, file_path: &str) {
    assert!(border >= 0, "Border must be non-negative");
    assert!(scale_factor > 0, "Scale must be positive");
    // Convert QR code to an image
    let size = qr.size();
    // let border = 4;
    let img_size = (size + 2 * border) as u32;
    let mut img = ImageBuffer::from_pixel(img_size, img_size, Luma([255u8]));

    for y in 0..size {
        for x in 0..size {
            if qr.get_module(x, y) {
                img.put_pixel((x + border) as u32, (y + border) as u32, Luma([0u8]));
            }
        }
    }

    // Scale the image
    let scaled_img = image::imageops::resize(&img, img_size * scale_factor, img_size * scale_factor, FilterType::Nearest);

    // Save the scaled image as a PNG file
    scaled_img.save(file_path).unwrap();
}
