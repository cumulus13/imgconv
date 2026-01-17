// File: src\main.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2026-01-17
// Description: 
// License: MIT

use clap::{Parser, ValueEnum, ArgAction};
use clap_version_flag::colorful_version;
use image::{ImageFormat, ImageReader, GenericImageView};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use colored::*;

const ABOUT: &str = "
imgconv - Professional Image Format Converter

A fast and reliable command-line tool for converting images between different formats.
Supports PNG, JPEG, GIF, BMP, ICO, TIFF, WebP, AVIF, and more.

EXAMPLES:
    # Simple conversion (format auto-detected from extension)
    imgconv input.webp output.png
    
    # With explicit input/output flags
    imgconv -i image.jpg -o image.webp
    
    # Specify quality for lossy formats
    imgconv input.png output.jpg -q 85
    
    # Force output format
    imgconv input.jpg output -f png
    
    # Batch conversion pattern
    for f in *.webp; do imgconv \"$f\" \"${f%.webp}.png\"; done
";

#[derive(Debug, Clone, ValueEnum)]
#[value(rename_all = "lowercase")]
enum Format {
    Png,
    Jpeg,
    Jpg,
    Gif,
    Bmp,
    Ico,
    Tiff,
    Tif,
    Webp,
    Avif,
    Pnm,
    Tga,
    Dds,
    Hdr,
    Farbfeld,
}

impl Format {
    fn to_image_format(&self) -> ImageFormat {
        match self {
            Format::Png => ImageFormat::Png,
            Format::Jpeg | Format::Jpg => ImageFormat::Jpeg,
            Format::Gif => ImageFormat::Gif,
            Format::Bmp => ImageFormat::Bmp,
            Format::Ico => ImageFormat::Ico,
            Format::Tiff | Format::Tif => ImageFormat::Tiff,
            Format::Webp => ImageFormat::WebP,
            Format::Avif => ImageFormat::Avif,
            Format::Pnm => ImageFormat::Pnm,
            Format::Tga => ImageFormat::Tga,
            Format::Dds => ImageFormat::Dds,
            Format::Hdr => ImageFormat::Hdr,
            Format::Farbfeld => ImageFormat::Farbfeld,
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "imgconv",
    author = "Hadi Cahyadi <cumulus13@gmail.com>",
    about = ABOUT,
    long_about = None,
    disable_version_flag = true
)]
struct Args {
    /// Input image file
    #[arg(short, long, value_name = "FILE")]
    input: Option<PathBuf>,

    /// Output image file or directory
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Output format (auto-detected from extension if not specified)
    #[arg(short, long, value_name = "FORMAT")]
    format: Option<Format>,

    /// Quality for lossy formats like JPEG (1-100)
    #[arg(short, long, default_value = "90", value_name = "NUM")]
    quality: u8,

    /// Positional input file (alternative to -i)
    #[arg(value_name = "INPUT")]
    pos_input: Option<PathBuf>,

    /// Positional output file (alternative to -o)
    #[arg(value_name = "OUTPUT")]
    pos_output: Option<PathBuf>,

    #[arg(short = 'V', long = "version", action = ArgAction::SetTrue)]
    version: bool,
}

fn main() -> Result<()> {
    let os_args: Vec<String> = std::env::args().collect();
    if os_args.len() == 2 && (os_args[1] == "-V" || os_args[1] == "--version") {
        let version = colorful_version!();
        version.print_and_exit();
    }

    let args = Args::parse();

    if args.version {
        let version = colorful_version!(); 
        version.print_and_exit();
    }

    // Determine input and output from either flags or positional args
    let input = args.input
        .or(args.pos_input)
        .context("Input file is required. Usage: imgconv <input> <output>")?;

    let output = args.output
        .or(args.pos_output)
        .context("Output file is required. Usage: imgconv <input> <output>")?;

    // Validate quality
    if args.quality == 0 || args.quality > 100 {
        anyhow::bail!("Quality must be between 1 and 100, got: {}", args.quality);
    }

    // Validate input exists
    if !input.exists() {
        anyhow::bail!("Input file not found: {}", input.display());
    }

    // Read image input
    print_info(&format!("Reading image from: {}", input.display()));
    let img = ImageReader::open(&input)
        .with_context(|| format!("Failed to open input file: {}", input.display()))?
        .with_guessed_format()
        .with_context(|| format!("Failed to detect image format from: {}", input.display()))?
        .decode()
        .context("Failed to decode image")?;

    let (width, height) = img.dimensions();
    print_success(&format!("Image loaded: {}x{} pixels", width, height));

    // Determine output format
    let (output_path, output_format) = determine_output(&output, args.format)?;

    // Convert and save
    print_info(&format!("Converting to format: {:?}", output_format));
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
    }

    // Save with appropriate encoder
    match output_format {
        ImageFormat::Jpeg => {
            let file = std::fs::File::create(&output_path)
                .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(file, args.quality);
            encoder.encode_image(&img)
                .context("Failed to encode JPEG image")?;
            print_success(&format!("JPEG quality: {}", args.quality));
        }
        _ => {
            img.save_with_format(&output_path, output_format)
                .with_context(|| format!("Failed to save image to: {}", output_path.display()))?;
        }
    }

    // Get file size
    if let Ok(metadata) = std::fs::metadata(&output_path) {
        let size_kb = metadata.len() / 1024;
        print_success(&format!("Output size: {} KB", size_kb));
    }

    print_success(&format!("Successfully converted to: {}", output_path.display()));
    Ok(())
}

fn determine_output(output: &Path, format: Option<Format>) -> Result<(PathBuf, ImageFormat)> {
    if let Some(fmt) = format {
        // Format explicitly specified
        let output_format = fmt.to_image_format();
        let output_path = add_extension_if_needed(output, &fmt);
        Ok((output_path, output_format))
    } else {
        // Try to detect from output extension
        if let Some(detected_format) = detect_format_from_path(output) {
            Ok((output.to_path_buf(), detected_format))
        } else {
            anyhow::bail!(
                "Could not determine output format from '{}'. Please specify --format or use a recognized extension",
                output.display()
            )
        }
    }
}

fn add_extension_if_needed(path: &Path, format: &Format) -> PathBuf {
    // If path already has the correct extension, return as-is
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            let ext_lower = ext_str.to_lowercase();
            let format_ext = format_to_extension(format);
            if ext_lower == format_ext || ext_lower == format_ext.replace("jpeg", "jpg") {
                return path.to_path_buf();
            }
        }
    }
    
    // Add extension
    let ext = format_to_extension(format);
    let mut new_path = path.to_path_buf();
    new_path.set_extension(ext);
    new_path
}

fn format_to_extension(format: &Format) -> &str {
    match format {
        Format::Png => "png",
        Format::Jpeg | Format::Jpg => "jpg",
        Format::Gif => "gif",
        Format::Bmp => "bmp",
        Format::Ico => "ico",
        Format::Tiff | Format::Tif => "tiff",
        Format::Webp => "webp",
        Format::Avif => "avif",
        Format::Pnm => "pnm",
        Format::Tga => "tga",
        Format::Dds => "dds",
        Format::Hdr => "hdr",
        Format::Farbfeld => "ff",
    }
}

fn detect_format_from_path(path: &Path) -> Option<ImageFormat> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "png" => Some(ImageFormat::Png),
        "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
        "gif" => Some(ImageFormat::Gif),
        "bmp" => Some(ImageFormat::Bmp),
        "ico" => Some(ImageFormat::Ico),
        "tiff" | "tif" => Some(ImageFormat::Tiff),
        "webp" => Some(ImageFormat::WebP),
        "avif" => Some(ImageFormat::Avif),
        "pnm" | "pbm" | "pgm" | "ppm" => Some(ImageFormat::Pnm),
        "tga" => Some(ImageFormat::Tga),
        "dds" => Some(ImageFormat::Dds),
        "hdr" => Some(ImageFormat::Hdr),
        "ff" => Some(ImageFormat::Farbfeld),
        _ => None,
    }
}

fn print_info(msg: &str) {
    eprintln!("{} {}", "[INFO]".blue().bold(), msg);
}

fn print_success(msg: &str) {
    eprintln!("{} {}", "[✓]".green().bold(), msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert!(matches!(
            detect_format_from_path(Path::new("test.png")),
            Some(ImageFormat::Png)
        ));
        assert!(matches!(
            detect_format_from_path(Path::new("test.jpg")),
            Some(ImageFormat::Jpeg)
        ));
        assert!(matches!(
            detect_format_from_path(Path::new("test.jpeg")),
            Some(ImageFormat::Jpeg)
        ));
        assert!(matches!(
            detect_format_from_path(Path::new("test.webp")),
            Some(ImageFormat::WebP)
        ));
    }

    #[test]
    fn test_format_to_extension() {
        assert_eq!(format_to_extension(&Format::Png), "png");
        assert_eq!(format_to_extension(&Format::Jpeg), "jpg");
        assert_eq!(format_to_extension(&Format::Webp), "webp");
    }
}