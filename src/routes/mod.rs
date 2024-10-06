use log::{info, warn};
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
