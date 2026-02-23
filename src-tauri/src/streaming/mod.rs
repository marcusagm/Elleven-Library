//! HLS On-the-Fly Streaming Module
//!
//! Provides real-time video transcoding via HLS protocol.
//! Segments are generated on-demand and cached to disk.

pub mod linear;
pub mod playlist;
pub mod probe;
pub mod process_manager;
pub mod segment;
pub mod server;
