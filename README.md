# imgconv

[![Crates.io](https://img.shields.io/crates/v/imgconv.svg)](https://crates.io/crates/imgconv)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![CI](https://github.com/cumulus13/imgconv/workflows/CI/badge.svg)](https://github.com/cumulus13/imgconv/actions)
[![Release](https://github.com/cumulus13/imgconv/workflows/Release/badge.svg)](https://github.com/cumulus13/imgconv/actions)


A professional, fast, and reliable command-line tool for converting images between different formats.

## Features

- 🚀 **Fast & Efficient** - Built with Rust for maximum performance
- 🎨 **Multiple Formats** - Supports PNG, JPEG, GIF, BMP, ICO, TIFF, WebP, AVIF, and more
- 🔧 **Flexible CLI** - Multiple input methods (flags or positional arguments)
- 📦 **Smart Detection** - Automatically detects output format from file extension
- 🎯 **Quality Control** - Adjustable quality settings for lossy formats
- 🌈 **Colored Output** - Beautiful, informative terminal output
- ⚡ **Zero Configuration** - Works out of the box

## Installation

### From crates.io

```bash
cargo install imgconv
```

### From source

```bash
git clone https://github.com/cumulus13/imgconv
cd imgconv
cargo install --path .
```

### Build from source

```bash
git clone https://github.com/cumulus13/imgconv
cd imgconv
cargo build --release
# Binary will be at ./target/release/imgconv
```

## Usage

### Basic Usage

The simplest way to convert an image:

```bash
imgconv input.webp output.png
```

### With Flags

```bash
imgconv -i photo.jpg -o photo.webp
```

### Quality Control

Specify quality for lossy formats (1-100, default: 90):

```bash
imgconv input.png output.jpg -q 85
```

### Force Output Format

When output filename doesn't have an extension:

```bash
imgconv input.jpg output -f png
```

This will create `output.png`

### Mixed Syntax

All these commands are valid:

```bash
# Positional arguments
imgconv image.bmp image.png

# With input flag only
imgconv -i photo.webp output.jpg

# With output flag only
imgconv input.png -o result.jpg -q 95

# All flags
imgconv -i input.avif -o output.png -f png -q 100
```

## Supported Formats

| Format | Extension(s) | Read | Write |
|--------|-------------|------|-------|
| PNG | `.png` | ✅ | ✅ |
| JPEG | `.jpg`, `.jpeg` | ✅ | ✅ |
| GIF | `.gif` | ✅ | ✅ |
| BMP | `.bmp` | ✅ | ✅ |
| ICO | `.ico` | ✅ | ✅ |
| TIFF | `.tiff`, `.tif` | ✅ | ✅ |
| WebP | `.webp` | ✅ | ✅ |
| AVIF | `.avif` | ✅ | ✅ |
| PNM | `.pnm`, `.pbm`, `.pgm`, `.ppm` | ✅ | ✅ |
| TGA | `.tga` | ✅ | ✅ |
| DDS | `.dds` | ✅ | ✅ |
| HDR | `.hdr` | ✅ | ✅ |
| Farbfeld | `.ff` | ✅ | ✅ |

## Examples

### Convert WebP to PNG

```bash
imgconv photo.webp photo.png
```

### Batch Conversion

Convert all WebP files to PNG in the current directory:

**Linux/macOS:**
```bash
for f in *.webp; do imgconv "$f" "${f%.webp}.png"; done
```

**Windows PowerShell:**
```powershell
Get-ChildItem *.webp | ForEach-Object { imgconv $_.Name ($_.BaseName + ".png") }
```

**Windows CMD:**
```cmd
for %f in (*.webp) do imgconv "%f" "%~nf.png"
```

### Compress JPEG

Reduce JPEG file size with lower quality:

```bash
imgconv large.jpg compressed.jpg -q 75
```

### Convert to WebP

Modern format with great compression:

```bash
imgconv photo.png photo.webp -q 80
```

### Create Thumbnail

Combine with other tools like ImageMagick:

```bash
# Resize then convert
convert input.jpg -resize 200x200 temp.jpg && imgconv temp.jpg thumbnail.webp -q 85
```

## CLI Options

```
Usage: imgconv [OPTIONS] [INPUT] [OUTPUT]

Arguments:
  [INPUT]   Positional input file (alternative to -i)
  [OUTPUT]  Positional output file (alternative to -o)

Options:
  -i, --input <FILE>     Input image file
  -o, --output <FILE>    Output image file or directory
  -f, --format <FORMAT>  Output format (auto-detected from extension if not specified)
                         [possible values: png, jpeg, jpg, gif, bmp, ico, tiff, tif, 
                          webp, avif, pnm, tga, dds, hdr, farbfeld]
  -q, --quality <NUM>    Quality for lossy formats like JPEG (1-100) [default: 90]
  -V, --version          Print version information
  -h, --help             Print help
```

## Performance

imgconv is built with Rust and optimized for performance:

- **Fast Decoding**: Efficient image parsing
- **Optimized Encoding**: Smart compression algorithms
- **Low Memory**: Minimal memory footprint
- **Release Build**: LTO and optimizations enabled

## Error Handling

imgconv provides clear, helpful error messages:

```bash
$ imgconv missing.jpg output.png
Error: Input file not found: missing.jpg

$ imgconv input.jpg output.xyz
Error: Could not determine output format from 'output.xyz'. 
Please specify --format or use a recognized extension
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## 👤 Author
        
[Hadi Cahyadi](mailto:cumulus13@gmail.com)
    

[![Buy Me a Coffee](https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png)](https://www.buymeacoffee.com/cumulus13)

[![Donate via Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/cumulus13)
 
[Support me on Patreon](https://www.patreon.com/cumulus13)

## Acknowledgments

- Built with the excellent [image](https://github.com/image-rs/image) crate
- CLI powered by [clap](https://github.com/clap-rs/clap)
- Thanks to the Rust community for amazing tools and libraries

Made with ❤️ using Rust