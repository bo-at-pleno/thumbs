use warp::Filter;
use std::convert::Infallible;
use std::fs::File;
use std::io::Cursor;
use image::{DynamicImage, GenericImageView, ImageOutputFormat};
use serde::Deserialize;

#[tokio::main]
async fn main() {
    // Route: /thumbnail/{image_path}?width=100&height=100
    let thumbnail_route = warp::path!("thumbnail" / String)
        .and(warp::query::<ThumbnailParams>())
        .and_then(handle_thumbnail);

    warp::serve(thumbnail_route).run(([127, 0, 0, 1], 3030)).await;
}

// Struct to parse query parameters (width and height).
#[derive(Debug, Deserialize)]
struct ThumbnailParams {
    width: u32,
    height: u32,
}

// Handler function for creating the thumbnail.
async fn handle_thumbnail(image_path: String, params: ThumbnailParams) -> Result<impl warp::Reply, Infallible> {
    match create_thumbnail(&image_path, params.width, params.height) {
        Ok(buffer) => Ok(warp::reply::with_header(
            buffer,
            "Content-Type",
            "image/png",
        )),
        Err(_) => Ok(warp::reply::with_status(
            "Failed to generate thumbnail".to_string(),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

// Function to create a thumbnail.
fn create_thumbnail(image_path: &str, width: u32, height: u32) -> Result<warp::hyper::body::Bytes, std::io::Error> {
    // Open and load the image from the file path.
    let img = image::open(&image_path).expect("Failed to open image file");

    // Resize the image to the given dimensions.
    let thumbnail = img.thumbnail(width, height);

    // Encode the resized image to PNG format.
    let mut buffer = Cursor::new(Vec::new());
    thumbnail
        .write_to(&mut buffer, ImageOutputFormat::Png)
        .expect("Failed to write image to buffer");

    // Convert the buffer to a warp-compatible response.
    Ok(warp::hyper::body::Bytes::from(buffer.into_inner()))
}
