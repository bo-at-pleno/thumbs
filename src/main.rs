use clap::Parser;
use log::{error, info, warn};
use simplelog::{ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};
use std::net::SocketAddr;
use tokio::signal;
use warp::Filter;

mod img;
mod routes;
use routes::thumbnail_route;

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
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    let args = Args::parse();
    img::initialize_cache(args.cache_size);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .map_err(|e| {
            error!("Failed to parse address: {}", e);
            std::process::exit(1);
        })
        .unwrap();

    info!("Server started at {}", addr);

    // Use the thumbnail_route from the routes module
    let routes = thumbnail_route();

    // Create a future that listens for the termination signal.
    let shutdown_signal = async {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        warn!("Received Ctrl+C, shutting down...");
    };

    // Start the server and wait for either the server to complete or the shutdown signal.
    tokio::select! {
        _ = warp::serve(routes).run(addr) => {},
        _ = shutdown_signal => {},
    }

    info!("Bye!");
}
