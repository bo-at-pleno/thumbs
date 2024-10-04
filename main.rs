use image::{DynamicImage, GenericImageView, ImageOutputFormat};
use serde::Deserialize;
use std::convert::Infallible;
use std::fs::File;
use std::io::Cursor;
use tokio::signal;
use warp::http::Response;
use warp::hyper::Body;
use warp::Filter;

#[tokio::main]
async fn main() {
    println!("Server started!");

    // Route: /thumbnail/{image_path}?width=100&height=100
    let thumbnail_route = warp::path!("thumbnail" / String)
        .and(warp::query::<ThumbnailParams>())
        .and_then(handle_thumbnail);

    // Create a future that listens for the termination signal.
    let shutdown_signal = async {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        println!("Received Ctrl+C, shutting down...");
    };

    // Start the server and wait for either the server to complete or the shutdown signal.
    tokio::select! {
        _ = warp::serve(thumbnail_route).run(([127, 0, 0, 1], 3030)) => {},
        _ = shutdown_signal => {},
    }

    println!("Bye!");
}

// Struct to parse query parameters (width and height).
#[derive(Debug, Deserialize)]
struct ThumbnailParams {
    width: u32,
    height: u32,
}

async fn handle_thumbnail(
    image_path: String,
    params: ThumbnailParams,
) -> Result<impl warp::Reply, Infallible> {
    match create_thumbnail(&image_path, params.width, params.height) {
        Ok(buffer) => {
            let response = Response::builder()
                .header("Content-Type", "image/png")
                .body(Body::from(buffer))
                .unwrap();
            Ok(response)
        }
        Err(_) => {
            let response = Response::builder()
                .status(warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Failed to generate thumbnail".to_string()))
                .unwrap();
            Ok(response)
        }
    }
}

// Function to create a thumbnail.
fn create_thumbnail(
    image_path: &str,
    width: u32,
    height: u32,
) -> Result<warp::hyper::body::Bytes, std::io::Error> {
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
