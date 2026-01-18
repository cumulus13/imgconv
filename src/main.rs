// File: src\main.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2026-01-17
// Description: 
// License: MIT

use clap::{Parser, ValueEnum, ArgAction};
use clap_version_flag::colorful_version;
use image::{ImageFormat, ImageReader, GenericImageView, DynamicImage};
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
    
    # Paste from clipboard
    imgconv -c output_image
    
    # Paste from clipboard with specific extension
    imgconv -c output_image.png
    
    # Paste and convert to specific format
    imgconv -c output_image -e jpg
    imgconv -c output_image.png -e jpg
    
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
    #[arg(short, long, value_name = "FILE", conflicts_with = "clipboard")]
    input: Option<PathBuf>,

    /// Output image file or directory
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Paste image from clipboard
    #[arg(short = 'c', long)]
    clipboard: bool,

    /// Output format (auto-detected from extension if not specified)
    #[arg(short, long, value_name = "FORMAT")]
    format: Option<Format>,

    /// Extension for output file (use with -c for conversion)
    #[arg(short = 'e', long, value_name = "EXT")]
    extension: Option<String>,

    /// Quality for lossy formats like JPEG (1-100)
    #[arg(short, long, default_value = "90", value_name = "NUM")]
    quality: u8,

    /// Positional input file (alternative to -i)
    #[arg(value_name = "INPUT", conflicts_with = "clipboard")]
    pos_input: Option<PathBuf>,

    /// Positional output file (alternative to -o)
    #[arg(value_name = "OUTPUT")]
    pos_output: Option<PathBuf>,

    #[arg(short = 'V', long = "version", action = ArgAction::SetTrue)]
    version: bool
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

    // Validate quality
    if args.quality == 0 || args.quality > 100 {
        anyhow::bail!("Quality must be between 1 and 100, got: {}", args.quality);
    }

    // Determine input source: clipboard or file
    let (img, detected_input_format) = if args.clipboard {
        // Get from clipboard
        print_info("Reading image from clipboard...");
        get_image_from_clipboard()?
    } else {
        // Get from file
        let input = args.input
            .or(args.pos_input)
            .context("Input file is required. Usage: imgconv <input> <output> OR imgconv -c <output>")?;

        // Validate input exists
        if !input.exists() {
            anyhow::bail!("Input file not found: {}", input.display());
        }

        // Read image input
        print_info(&format!("Reading image from: {}", input.display()));
        let reader = ImageReader::open(&input)
            .with_context(|| format!("Failed to open input file: {}", input.display()))?
            .with_guessed_format()
            .with_context(|| format!("Failed to detect image format from: {}", input.display()))?;
        
        let detected_format = reader.format();
        let img = reader.decode()
            .context("Failed to decode image")?;
        
        (img, detected_format)
    };

    let (width, height) = img.dimensions();
    if let Some(fmt) = detected_input_format {
        print_success(&format!("Image loaded: {}x{} pixels, format: {:?}", width, height, fmt));
    } else {
        print_success(&format!("Image loaded: {}x{} pixels", width, height));
    }

    // Determine output path
    let output = args.output
        .or(args.pos_output)
        .context("Output file is required. Usage: imgconv <input> <output> OR imgconv -c <output>")?;

    // Determine output format with smart logic for clipboard mode
    let (output_path, output_format) = if args.clipboard {
        determine_output_from_clipboard(
            &output, 
            args.format, 
            args.extension.as_deref(), 
            detected_input_format
        )?
    } else {
        determine_output(&output, args.format)?
    };

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

fn get_image_from_clipboard() -> Result<(DynamicImage, Option<ImageFormat>)> {
    use arboard::Clipboard;
    
    let mut clipboard = Clipboard::new()
        .context("Failed to access clipboard")?;
    
    let img_data = clipboard.get_image()
        .context("No image found in clipboard. Please copy an image first.")?;
    
    // Convert RGBA to image format
    let width = img_data.width;
    let height = img_data.height;
    let rgba_data = img_data.bytes;
    
    // Create DynamicImage from RGBA data
    let img = image::RgbaImage::from_raw(width as u32, height as u32, rgba_data.to_vec())
        .context("Failed to create image from clipboard data")?;
    
    let dynamic_img = DynamicImage::ImageRgba8(img);
    
    // Clipboard images are typically in PNG format
    Ok((dynamic_img, Some(ImageFormat::Png)))
}

fn determine_output_from_clipboard(
    output: &Path,
    explicit_format: Option<Format>,
    extension: Option<&str>,
    detected_format: Option<ImageFormat>
) -> Result<(PathBuf, ImageFormat)> {
    // Priority:
    // 1. -e flag (extension) with conversion
    // 2. -f flag (format)
    // 3. output file extension
    // 4. detected format from clipboard
    
    if let Some(ext) = extension {
        // User specified -e flag, convert to that format
        let target_format = extension_to_format(ext)
            .with_context(|| format!("Unknown extension: {}", ext))?;
        
        let mut output_path = output.to_path_buf();
        
        // If output has different extension, correct it
        if let Some(current_ext) = output.extension() {
            if current_ext.to_string_lossy().to_lowercase() != ext.to_lowercase() {
                print_info(&format!(
                    "Correcting extension from .{} to .{} (conversion mode)", 
                    current_ext.to_string_lossy(), 
                    ext
                ));
                output_path.set_extension(ext);
            }
        } else {
            output_path.set_extension(ext);
        }
        
        return Ok((output_path, target_format));
    }
    
    if let Some(fmt) = explicit_format {
        // User specified -f flag
        let output_format = fmt.to_image_format();
        let output_path = add_extension_if_needed(output, &fmt);
        return Ok((output_path, output_format));
    }
    
    // Check if output has extension
    if let Some(output_ext) = output.extension() {
        let ext_str = output_ext.to_string_lossy().to_lowercase();
        
        // Check if extension matches detected format
        if let Some(detected) = detected_format {
            let detected_ext = format_to_main_extension(&detected);
            
            if ext_str != detected_ext && ext_str != "jpg" && detected_ext != "jpeg" {
                // Extension doesn't match, correct it
                print_info(&format!(
                    "Output extension .{} doesn't match clipboard format .{}, correcting...", 
                    ext_str, 
                    detected_ext
                ));
                
                let mut corrected_path = output.to_path_buf();
                corrected_path.set_extension(detected_ext);
                return Ok((corrected_path, detected));
            }
        }
        
        // Try to detect format from output extension
        if let Some(format_from_ext) = detect_format_from_path(output) {
            return Ok((output.to_path_buf(), format_from_ext));
        }
    }
    
    // Fallback to detected format or PNG
    let final_format = detected_format.unwrap_or(ImageFormat::Png);
    let ext = format_to_main_extension(&final_format);
    
    let mut output_path = output.to_path_buf();
    output_path.set_extension(ext);
    
    print_info(&format!("Auto-adding extension: .{}", ext));
    
    Ok((output_path, final_format))
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

fn extension_to_format(ext: &str) -> Option<ImageFormat> {
    match ext.to_lowercase().as_str() {
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

fn format_to_main_extension(format: &ImageFormat) -> &str {
    match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        ImageFormat::Gif => "gif",
        ImageFormat::Bmp => "bmp",
        ImageFormat::Ico => "ico",
        ImageFormat::Tiff => "tiff",
        ImageFormat::WebP => "webp",
        ImageFormat::Avif => "avif",
        ImageFormat::Pnm => "pnm",
        ImageFormat::Tga => "tga",
        ImageFormat::Dds => "dds",
        ImageFormat::Hdr => "hdr",
        ImageFormat::Farbfeld => "ff",
        _ => "png",
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
