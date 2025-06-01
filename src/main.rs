use clap::Parser;
use qrcodegen::{QrCode, QrCodeEcc};
use image::{Luma, ImageBuffer, imageops::FilterType};
use std::fs::File;
use std::io::{self, Read, IsTerminal};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about = "Create a QR code from text file or piped data")]
struct Cli {
    #[arg(short, long, value_name = "ECC", help = "Error correction level. Use L, M, Q, or H.", default_value = "M")]
    ecc: String,

    #[arg(short, long, value_name = "INPUT", help = "Unicode text file or piped data.")]
    input: Option<PathBuf>,

    #[arg(short = 't', long, value_name = "OUTPUT_TYPE", help = "Output file/data types. Use Text, SVG or PNG", default_value = "Text")]
    output_type: OutputType,

    #[arg(short = 'o', long, value_name = "OUTPUT_FILE", help = "Output file path only used for PNG.", default_value = "qrcode.png")]
    output_file: String,

    #[arg(short = 'b', long, value_name = "BORDER_WIDTH", help = "SVG or PNG border surrounding the QR code.", default_value_t = 4)]
    border_width: i32,

    #[arg(short = 's', long, value_name = "SCALE", help = "Scale of the SVG or PNG image.", default_value_t = 10)]
    scale: i32,
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

    // Call the read_input function
    let text: String = read_input(&args.input)?;

    // Attempt to encode the text into a QR code
    match QrCode::encode_text(&text, ecc) {
        Ok(qr) => {
            match args.output_type {
                OutputType::TXT => print_qr(&qr),
                OutputType::SVG => println!("{}", to_svg_string(&qr, args.border_width, args.scale)),
                OutputType::PNG => {
                    if let Err(e) = write_to_png_scaled(&qr, args.border_width, args.scale as u32, &args.output_file) {
                        eprintln!("Error writing PNG: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to generate QR code: {}", e);
        }
    }

    Ok(())
}


/*---- Utilities ----*/


// Check if there's data in the standard input
// Otherwise, read from a file
fn read_input(input: &Option<PathBuf>) -> Result<String, io::Error> {
    let mut text = String::new();

    if let Some(file_path) = input {
        File::open(file_path)
            .and_then(|mut file| file.read_to_string(&mut text))
            .map_err(|e| {
                eprintln!("Error reading file '{}': {}", file_path.display(), e);
                e
            })?;
    } else if !io::stdin().is_terminal() {
        io::stdin().read_to_string(&mut text)?;
    } else {
        eprintln!("No input provided. Please specify a file or pipe data.");
        return Ok(String::new());
    }

    Ok(text)
}


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
fn write_to_png_scaled(qr: &QrCode, border: i32, scale_factor: u32, file_path: &str) -> Result<(), String> {
    // Validate inputs
    if border < 0 {
        return Err("Border must be non-negative".to_string());
    }
    if scale_factor < 1 {
        return Err("Scale factor must be positive".to_string());
    }

    // Calculate image size
    let size: i32 = qr.size();
    let img_size: u32 = (size + 2 * border) as u32;
    let mut img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_pixel(img_size, img_size, Luma([255u8]));

    // Draw QR code onto the image
    for y in 0..size {
        for x in 0..size {
            if qr.get_module(x, y) {
                img.put_pixel((x + border) as u32, (y + border) as u32, Luma([0u8]));
            }
        }
    }

    // Scale the image
    let scaled_img: ImageBuffer<Luma<u8>, Vec<u8>> = image::imageops::resize(&img, img_size * scale_factor, img_size * scale_factor, FilterType::Nearest);

    // Save the scaled image as a PNG file
    scaled_img.save(file_path).map_err(|e| format!("Failed to save PNG file: {}", e))
}
