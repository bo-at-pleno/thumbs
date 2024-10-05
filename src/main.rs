use std::convert::Infallible;
use tokio::signal;
use warp::http::Response;
use warp::hyper::Body;
use warp::Filter;

mod img;
use img::{create_thumbnail, ThumbnailParams};

#[tokio::main]
async fn main() {
    println!("Server started!");

    // Route: /thumbnail/{image_path}?width=100&height=100
    let thumbnail_route = warp::path("thumbnail")
        .and(warp::path::tail())
        .and(warp::query::<ThumbnailParams>())
        .and_then(handle_thumbnail)
        .with(warp::log("thumbs::requests")); // Add logging

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

async fn handle_thumbnail(
    tail: warp::path::Tail,
    params: ThumbnailParams,
) -> Result<impl warp::Reply, Infallible> {
    println!("thumbnailing!");
    let image_path = tail.as_str().to_string();
    println!("image_path: {:?}, params: {:?}", image_path, params);
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
