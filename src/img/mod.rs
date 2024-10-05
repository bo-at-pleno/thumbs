use blake3::Hasher;
use image::ImageOutputFormat;
use lazy_static::lazy_static;
use log::info;
use lru::LruCache;
use serde::Deserialize;
use std::io::{Cursor, Read};
use std::num::NonZero;
use std::sync::Mutex;
use warp::hyper::body::Bytes;

#[derive(Debug, Deserialize)]
pub struct ThumbnailParams {
    pub width: u32,
    pub height: u32,
}

lazy_static! {
    static ref CACHE: Mutex<LruCache<String, Bytes>> =
        Mutex::new(LruCache::new(NonZero::new(100).unwrap()));
}

pub fn initialize_cache(size: usize) {
    let non_zero_size = std::num::NonZeroUsize::new(size).unwrap();
    let mut cache = CACHE.lock().unwrap();
    *cache = LruCache::new(non_zero_size);
}

fn generate_cache_key(image_path: &str, width: u32, height: u32) -> String {
    let mut hasher = Hasher::new();
    hasher.update(image_path.as_bytes());
    hasher.update(&width.to_le_bytes());
    hasher.update(&height.to_le_bytes());
    let hash = hasher.finalize();
    hash.to_hex().to_string()
}

pub fn create_thumbnail(
    image_path: &str,
    width: u32,
    height: u32,
) -> Result<Bytes, std::io::Error> {
    let cache_key = generate_cache_key(image_path, width, height);
    info!("Cache key: {}", cache_key);

    // Check if the thumbnail is already cached in memory
    if let Some(cached_thumbnail) = CACHE.lock().unwrap().get(&cache_key) {
        info!("Thumbnail found in cache: {}", cache_key);
        return Ok(cached_thumbnail.clone());
    }

    // Check if the image file exists and is accessible.
    if !std::fs::metadata(image_path).is_ok() {
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

    let bytes = Bytes::from(buffer.into_inner());

    // Save the thumbnail to the in-memory cache
    CACHE.lock().unwrap().put(cache_key.clone(), bytes.clone());

    Ok(bytes)
}
