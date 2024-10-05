use std::convert::Infallible;
use tokio::signal;
use warp::http::Response;
use warp::hyper::Body;
use warp::Filter;

use clap::Parser;
use std::net::SocketAddr;
mod img;
use img::{create_thumbnail, ThumbnailParams};

/// Simple thumbnail server that generates a thumbnail from an image file.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Host to bind the server to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to bind the server to
    #[arg(long, default_value_t = 3030)]
    port: u16,

    /// Size of the in-memory cache
    #[arg(short, long, default_value_t = 100)]
    cache_size: usize,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    img::initialize_cache(args.cache_size);
    let addr: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .expect("Invalid address");
    println!("Server started at {}", addr);

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
        _ = warp::serve(thumbnail_route).run(addr) => {},
        _ = shutdown_signal => {},
    }

    println!("Bye!");
}

async fn handle_thumbnail(
    tail: warp::path::Tail,
    params: ThumbnailParams,
) -> Result<impl warp::Reply, Infallible> {
    let image_path = tail.as_str().to_string();
    println!("Serving image_path: {:?}, params: {:?}", image_path, params);
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
