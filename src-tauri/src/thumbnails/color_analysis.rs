//! Color palette extraction using k-means clustering in the CIE-LAB color space.
//!
//! This module analyzes image thumbnails to extract a representative color palette.
//! It uses the `kmeans-colors` crate for efficient clustering and the `palette` crate
//! for accurate RGB↔LAB color space conversion.
//!
//! The algorithm:
//! 1. Loads the thumbnail image (already ~256px, ideal for performance)
//! 2. Converts RGBA pixels to CIE-LAB, ignoring transparent pixels (alpha < 128)
//! 3. Runs k-means clustering with k=16 centroids
//! 4. Sorts clusters by percentage (weight) descending
//! 5. Converts LAB centroids back to hex for storage and display

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

/// A single color extracted from an image's palette.
#[derive(Debug, Clone)]
pub struct ExtractedColor {
    /// Hexadecimal representation (e.g., "#FF5733").
    pub hex_value: String,
    /// CIE-LAB L* component (lightness, 0–100).
    pub lab_lightness: f64,
    /// CIE-LAB a* component (green-red axis, roughly -128 to 127).
    pub lab_green_red: f64,
    /// CIE-LAB b* component (blue-yellow axis, roughly -128 to 127).
    pub lab_blue_yellow: f64,
    /// Proportion of the image this color represents (0.0–1.0).
    pub percentage: f64,
}

/// Extracts a color palette from a thumbnail image using k-means clustering
/// in the CIE-LAB color space.
///
/// # Arguments
///
/// * `thumbnail_path` - Path to the thumbnail image (WebP, PNG, JPEG, etc.).
/// * `cluster_count` - Number of color clusters to extract. Pass `None` for the default (16).
///
/// # Returns
///
/// A vector of `ExtractedColor` sorted by percentage descending (most dominant first).
///
/// # Errors
///
/// Returns an error if the image cannot be loaded or has no valid (non-transparent) pixels.
pub fn extract_color_palette(
    thumbnail_path: &Path,
    cluster_count: Option<usize>,
) -> Result<Vec<ExtractedColor>, Box<dyn std::error::Error + Send + Sync>> {
    let effective_cluster_count = cluster_count.unwrap_or(DEFAULT_CLUSTER_COUNT);

    let image = image::open(thumbnail_path)?;
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
        return Err("No opaque pixels found in image".into());
    }

    let actual_cluster_count = effective_cluster_count.min(lab_pixels.len());

    let kmeans_result = run_kmeans_clustering(&lab_pixels, actual_cluster_count)?;

    let extracted_colors = build_color_palette(&kmeans_result, &lab_pixels, actual_cluster_count);

    debug!(
        "COLOR: Extracted {} colors from {:?}",
        extracted_colors.len(),
        thumbnail_path.file_name().unwrap_or_default()
    );

    Ok(extracted_colors)
}

/// Collects all opaque pixels from the image and converts them to CIE-LAB.
///
/// Pixels with alpha below `MINIMUM_ALPHA_THRESHOLD` are filtered out.
fn collect_opaque_lab_pixels(image: &image::DynamicImage) -> Vec<Lab> {
    let rgba_image = image.to_rgba8();
    let mut lab_pixels = Vec::with_capacity((rgba_image.width() * rgba_image.height()) as usize);

    for pixel in rgba_image.pixels() {
        if pixel[3] < MINIMUM_ALPHA_THRESHOLD {
            continue;
        }

        let red_normalized = pixel[0] as f32 / 255.0;
        let green_normalized = pixel[1] as f32 / 255.0;
        let blue_normalized = pixel[2] as f32 / 255.0;

        let srgb_color = Srgb::new(red_normalized, green_normalized, blue_normalized);
        let lab_color: Lab = srgb_color.into_color();
        lab_pixels.push(lab_color);
    }

    lab_pixels
}

/// Runs the k-means Hamerly algorithm on LAB-space pixels.
///
/// # Errors
///
/// Returns an error if clustering fails to converge or produces invalid results.
fn run_kmeans_clustering(
    lab_pixels: &[Lab],
    cluster_count: usize,
) -> Result<Kmeans<Lab>, Box<dyn std::error::Error + Send + Sync>> {
    let result = get_kmeans_hamerly(
        cluster_count,
        MAX_KMEANS_ITERATIONS,
        CONVERGENCE_FACTOR,
        false, // verbose
        lab_pixels,
        RANDOM_SEED,
    );

    Ok(result)
}

/// Converts k-means centroids into the final `ExtractedColor` palette.
///
/// Calculates the percentage of pixels closest to each centroid and
/// converts LAB colors back to hex for display.
fn build_color_palette(
    kmeans_result: &Kmeans<Lab>,
    lab_pixels: &[Lab],
    cluster_count: usize,
) -> Vec<ExtractedColor> {
    let centroids = &kmeans_result.centroids;
    let total_pixel_count = lab_pixels.len() as f64;

    // Count pixels assigned to each centroid
    let mut cluster_pixel_counts = vec![0usize; cluster_count];
    for pixel in lab_pixels {
        let nearest_centroid_index = find_nearest_centroid(pixel, centroids);
        cluster_pixel_counts[nearest_centroid_index] += 1;
    }

    let mut palette: Vec<ExtractedColor> = centroids
        .iter()
        .enumerate()
        .map(|(index, centroid)| {
            let percentage = cluster_pixel_counts[index] as f64 / total_pixel_count;
            let hex_value = lab_to_hex(centroid);

            ExtractedColor {
                hex_value,
                lab_lightness: centroid.l as f64,
                lab_green_red: centroid.a as f64,
                lab_blue_yellow: centroid.b as f64,
                percentage,
            }
        })
        .filter(|color| color.percentage > 0.001) // Skip negligible clusters
        .collect();

    palette.sort_by(|color_a, color_b| {
        color_b
            .percentage
            .partial_cmp(&color_a.percentage)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    palette
}

/// Finds the index of the nearest centroid to a given LAB pixel.
fn find_nearest_centroid(pixel: &Lab, centroids: &[Lab]) -> usize {
    let mut nearest_index = 0;
    let mut minimum_distance = f32::MAX;

    for (index, centroid) in centroids.iter().enumerate() {
        let distance = pixel.distance_squared(*centroid);
        if distance < minimum_distance {
            minimum_distance = distance;
            nearest_index = index;
        }
    }

    nearest_index
}

/// Converts a CIE-LAB color to a hexadecimal RGB string (e.g., "#FF5733").
fn lab_to_hex(lab_color: &Lab) -> String {
    let srgb_color: Srgb = (*lab_color).into_color();

    // Clamp to valid sRGB range before converting to u8
    let red = (srgb_color.red.clamp(0.0, 1.0) * 255.0).round() as u8;
    let green = (srgb_color.green.clamp(0.0, 1.0) * 255.0).round() as u8;
    let blue = (srgb_color.blue.clamp(0.0, 1.0) * 255.0).round() as u8;

    format!("#{:02X}{:02X}{:02X}", red, green, blue)
}

/// Converts a hexadecimal color string to CIE-LAB components.
///
/// Supports formats: "#RRGGBB" and "RRGGBB".
///
/// # Errors
///
/// Returns an error if the hex string is invalid.
pub fn hex_to_lab(
    hex_color: &str,
) -> Result<(f64, f64, f64), Box<dyn std::error::Error + Send + Sync>> {
    let hex_trimmed = hex_color.trim_start_matches('#');

    if hex_trimmed.len() != 6 {
        return Err(format!("Invalid hex color length: {}", hex_color).into());
    }

    let red = u8::from_str_radix(&hex_trimmed[0..2], 16)?;
    let green = u8::from_str_radix(&hex_trimmed[2..4], 16)?;
    let blue = u8::from_str_radix(&hex_trimmed[4..6], 16)?;

    let srgb_color = Srgb::new(
        red as f32 / 255.0,
        green as f32 / 255.0,
        blue as f32 / 255.0,
    );
    let lab_color: Lab = srgb_color.into_color();

    Ok((lab_color.l as f64, lab_color.a as f64, lab_color.b as f64))
}
