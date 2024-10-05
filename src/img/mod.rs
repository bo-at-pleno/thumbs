use image::ImageOutputFormat;
use serde::Deserialize;
use std::fs;
use std::io::Cursor;
use warp::hyper::body::Bytes;

#[derive(Debug, Deserialize)]
pub struct ThumbnailParams {
    pub width: u32,
    pub height: u32,
}

pub fn create_thumbnail(
    image_path: &str,
    width: u32,
    height: u32,
) -> Result<Bytes, std::io::Error> {
    // Check if the image file exists and is accessible.
    if !fs::metadata(image_path).is_ok() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Image file not found",
        ));
    }

    // Open and load the image from the file path.
    let img = image::open(image_path).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to open image file: {}", e),
        )
    })?;

    // Resize the image to the given dimensions.
    let thumbnail = img.thumbnail(width, height);

    // Encode the resized image to PNG format.
    let mut buffer = Cursor::new(Vec::new());
    thumbnail
        .write_to(&mut buffer, ImageOutputFormat::Png)
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to write image to buffer: {}", e),
            )
        })?;

    // Convert the buffer to a warp-compatible response.
    Ok(Bytes::from(buffer.into_inner()))
}
