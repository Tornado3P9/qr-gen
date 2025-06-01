# qr-gen

Create a QR Code from text file or piped data

```bash
Usage: qr-gen [OPTIONS]

Options:
  -e, --ecc <ECC>                  Error correction level. Use L, M, Q, or H. [default: M]
  -i, --input <INPUT>              Unicode text file or piped data.
  -t, --output-type <OUTPUT_TYPE>  Output file/data types. Use Text, SVG or PNG [default: Text]
  -o, --output-file <OUTPUT_FILE>  Output file path only used for PNG. [default: qrcode.png]
  -h, --help                       Print help
  -V, --version                    Print version

Examples:
  echo "Hello World!" | qr-gen
  qr-gen -i input.txt -t svg > qrcode.svg
  echo "Hello World!" | qr-gen -t png -o ~/qrcode.png
```

Inspired by: https://github.com/nayuki/QR-Code-generator/blob/2c9044de6b049ca25cb3cd1649ed7e27aa055138/rust/examples/qrcodegen-demo.rs

Test with: https://secuso.aifb.kit.edu/QR_Scanner.php
