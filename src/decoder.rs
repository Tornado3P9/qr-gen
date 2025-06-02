use std::{fs::File, io::{self, IsTerminal, Read}, path::PathBuf};
use image::ImageReader;

fn main() -> io::Result<()> {
    let myimage: Option<PathBuf> = Some(PathBuf::from("qrcode.png"));
    // let myimage: Option<PathBuf> = None;

    let img = read_image(myimage)?;

    let img_gray = img.into_luma8();

    // create a decoder
    let mut decoder = quircs::Quirc::default();

    // identify all qr codes
    let codes = decoder.identify(img_gray.width() as usize, img_gray.height() as usize, &img_gray);

    for code in codes {
        let code: quircs::Code = code.expect("failed to extract qr code");
        let decoded: quircs::Data = code.decode().expect("failed to decode qr code");
        println!("{}", std::str::from_utf8(&decoded.payload).unwrap());
    }

    Ok(())
}

fn read_image(input: Option<PathBuf>) -> io::Result<image::DynamicImage> {
    let mut buffer: Vec<u8> = Vec::new();

    if !io::stdin().is_terminal() {
        io::stdin().read_to_end(&mut buffer)?;
    } else if let Some(file_path) = input {
        File::open(file_path)?.read_to_end(&mut buffer)?;
    } else {
        eprintln!("No input provided. Please specify a file or pipe data.");
        std::process::exit(1);
    }

    let img = ImageReader::new(io::Cursor::new(buffer))
        .with_guessed_format()
        .expect("Failed to guess image format")
        .decode()
        .expect("Failed to decode image");

    Ok(img)
}
