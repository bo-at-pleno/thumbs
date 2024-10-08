use log::info;
use std::convert::Infallible;
use tokio::task;
use warp::http::{Response, StatusCode};
use warp::hyper::Body;
use warp::{Filter, Rejection, Reply};

use crate::img::{create_thumbnail, ThumbnailParams};

pub fn thumbnail_route() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("thumbnail")
        .and(warp::path::tail())
        .and(warp::query::<ThumbnailParams>())
        .and_then(handle_thumbnail)
        .with(warp::log("thumbs::requests"))
}

pub fn preview_route() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("preview")
        .and(warp::path::tail())
        .and(warp::query::<ThumbnailParams>())
        .and_then(handle_preview)
        .with(warp::log("thumbs::requests"))
}

async fn handle_thumbnail(
    tail: warp::path::Tail,
    params: ThumbnailParams,
) -> Result<impl warp::Reply, Infallible> {
    let image_path = tail.as_str().to_string();
    info!("Serving image_path: {:?}, params: {:?}", image_path, params);

    let result = task::spawn_blocking(move || create_thumbnail(&image_path, params)).await;

    match result {
        Ok(Ok(buffer)) => {
            let response = Response::builder()
                .header("Content-Type", "image/png")
                .body(Body::from(buffer))
                .unwrap();
            Ok(response)
        }
        Ok(Err(_)) | Err(_) => {
            let response = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Failed to generate thumbnail".to_string()))
                .unwrap();
            Ok(response)
        }
    }
}

async fn handle_preview(
    tail: warp::path::Tail,
    params: ThumbnailParams,
) -> Result<impl warp::Reply, Infallible> {
    let image_path = tail.as_str().to_string();
    info!(
        "Serving preview for image_path: {:?}, params: {:?}",
        image_path, params
    );

    // Implement the logic for generating a preview here
    // For now, we'll just return a placeholder response
    let response = Response::builder()
        .header("Content-Type", "text/plain")
        .body(Body::from("Preview not implemented".to_string()))
        .unwrap();
    Ok(response)
}
