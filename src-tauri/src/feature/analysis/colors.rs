//! Color palette extraction using k-means clustering in the CIE-LAB color space.
//!
//! This module analyzes image thumbnails to extract a representative color palette.
//! It uses the `kmeans-colors` crate for efficient clustering and the `palette` crate
//! for accurate RGB↔LAB color space conversion.

use crate::core::error::{AppError, AppResult};
use crate::core::models::asset::AssetColor;
use image::GenericImageView;
use kmeans_colors::{get_kmeans_hamerly, Kmeans};
use palette::color_difference::EuclideanDistance;
use palette::{IntoColor, Lab, Srgb};
use std::path::Path;
use tracing::debug;

/// Default number of color clusters to extract from an image.
const DEFAULT_CLUSTER_COUNT: usize = 16;

/// Maximum number of k-means iterations before convergence is forced.
const MAX_KMEANS_ITERATIONS: usize = 20;

/// Convergence threshold for k-means (stop when centroids move less than this).
const CONVERGENCE_FACTOR: f32 = 5.0;

/// Fixed seed for reproducible k-means results across runs.
const RANDOM_SEED: u64 = 42;

/// Minimum alpha value (0-255) for a pixel to be included in analysis.
/// Pixels with alpha below this are considered transparent and ignored.
const MINIMUM_ALPHA_THRESHOLD: u8 = 128;

/// Extracts a color palette from a thumbnail image using k-means clustering
/// in the CIE-LAB color space.
///
/// # Arguments
///
/// * `thumbnail_path` - Path to the thumbnail image (WebP, PNG, JPEG, etc.).
/// * `cluster_count` - Number of color clusters to extract. Defaults to 16.
///
/// # Errors
///
/// Returns `AppError::ExtractionError` if the image cannot be loaded or has no valid pixels.
pub fn extract_color_palette(
    thumbnail_path: &Path,
    cluster_count: Option<usize>,
) -> AppResult<Vec<AssetColor>> {
    let effective_cluster_count = cluster_count.unwrap_or(DEFAULT_CLUSTER_COUNT);

    let image = image::open(thumbnail_path).map_err(|e| {
        AppError::Internal(format!(
            "Failed to open thumbnail for color analysis: {}",
            e
        ))
    })?;

    let (image_width, image_height) = image.dimensions();

    debug!(
        "COLOR: Starting extraction for {:?} ({}x{}, k={})",
        thumbnail_path.file_name().unwrap_or_default(),
        image_width,
        image_height,
        effective_cluster_count
    );

    let lab_pixels = collect_opaque_lab_pixels(&image);

    if lab_pixels.is_empty() {
        return Err(AppError::Internal(
            "No opaque pixels found in image for color analysis".to_string(),
        ));
    }

    let actual_cluster_count = effective_cluster_count.min(lab_pixels.len());

    let kmeans_result = run_kmeans_clustering(&lab_pixels, actual_cluster_count);

    let extracted_colors = build_color_palette(&kmeans_result, &lab_pixels, actual_cluster_count);

    debug!(
        "COLOR: Extracted {} colors from {:?}",
        extracted_colors.len(),
        thumbnail_path.file_name().unwrap_or_default()
    );

    Ok(extracted_colors)
}

/// Collects opaque pixels and converts to CIELAB.
///
/// # Arguments
///
/// * `image` - The image to extract pixels from.
///
/// # Returns
///
/// A vector of CIELAB colors.
fn collect_opaque_lab_pixels(image: &image::DynamicImage) -> Vec<Lab> {
    let rgba_image = image.to_rgba8();
    let mut lab_pixels = Vec::with_capacity((rgba_image.width() * rgba_image.height()) as usize);

    for pixel in rgba_image.pixels() {
        if pixel[3] < MINIMUM_ALPHA_THRESHOLD {
            continue;
        }

        let srgb_color = Srgb::new(
            pixel[0] as f32 / 255.0,
            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0,
        );
        let lab_color: Lab = srgb_color.into_color();
        lab_pixels.push(lab_color);
    }

    lab_pixels
}

/// Runs the k-means algorithm.
///
/// # Arguments
///
/// * `lab_pixels` - The CIELAB pixels to cluster.
/// * `cluster_count` - The number of clusters to create.
///
/// # Returns
///
/// A `Kmeans` struct containing the clustering results.
fn run_kmeans_clustering(lab_pixels: &[Lab], cluster_count: usize) -> Kmeans<Lab> {
    get_kmeans_hamerly(
        cluster_count,
        MAX_KMEANS_ITERATIONS,
        CONVERGENCE_FACTOR,
        false,
        lab_pixels,
        RANDOM_SEED,
    )
}

/// Builds the formal AssetColor vector from k-means results.
///
/// # Arguments
///
/// * `kmeans_result` - The k-means clustering results.
/// * `lab_pixels` - The CIELAB pixels used for clustering.
/// * `cluster_count` - The number of clusters.
///
/// # Returns
///
/// A vector of `AssetColor` structs.
fn build_color_palette(
    kmeans_result: &Kmeans<Lab>,
    lab_pixels: &[Lab],
    cluster_count: usize,
) -> Vec<AssetColor> {
    let centroids = &kmeans_result.centroids;
    let total_pixel_count = lab_pixels.len() as f64;

    let mut cluster_pixel_counts = vec![0usize; cluster_count];
    for pixel in lab_pixels {
        let mut nearest_index = 0;
        let mut min_dist = f32::MAX;
        for (idx, centroid) in centroids.iter().enumerate() {
            let dist = pixel.distance_squared(*centroid);
            if dist < min_dist {
                min_dist = dist;
                nearest_index = idx;
            }
        }
        cluster_pixel_counts[nearest_index] += 1;
    }

    let mut palette: Vec<AssetColor> = centroids
        .iter()
        .enumerate()
        .map(|(index, centroid)| {
            let percentage = cluster_pixel_counts[index] as f64 / total_pixel_count;
            let hex_value = lab_to_hex(centroid);

            AssetColor {
                id: None,
                hex_color: hex_value,
                lab_lightness: centroid.l as f64,
                lab_green_red: centroid.a as f64,
                lab_blue_yellow: centroid.b as f64,
                percentage,
                rank: 0, // Will be set after sorting
            }
        })
        .filter(|color| color.percentage > 0.001)
        .collect();

    palette.sort_by(|a, b| b.percentage.partial_cmp(&a.percentage).unwrap());

    // Set ranks
    for (idx, color) in palette.iter_mut().enumerate() {
        color.rank = (idx + 1) as i32;
    }

    palette
}

/// Converts LAB to Hex.
///
/// # Arguments
///
/// * `lab_color` - The CIELAB color to convert.
///
/// # Returns
///
/// A string containing the hexadecimal representation of the color.
fn lab_to_hex(lab_color: &Lab) -> String {
    let srgb_color: Srgb = (*lab_color).into_color();
    let red = (srgb_color.red.clamp(0.0, 1.0) * 255.0).round() as u8;
    let green = (srgb_color.green.clamp(0.0, 1.0) * 255.0).round() as u8;
    let blue = (srgb_color.blue.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02X}{:02X}{:02X}", red, green, blue)
}
