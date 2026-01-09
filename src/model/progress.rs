//! Reading progress tracking for documents.
//!
//! This module defines the [`Progress`] struct which represents a user's reading position
//! and metadata for a specific document. Progress is synchronized across devices and
//! includes information about the device, position, and timestamp.

use rkyv::{Archive, Deserialize, Serialize};

/// Reading progress information for a document.
///
/// This struct contains all the information about a user's reading progress
/// in a specific document, including device information and the current position.
///
/// # Example
///
/// ```
/// use korrosync::model::Progress;
///
/// let progress = Progress {
///     device_id: "device-123".to_string(),
///     device: "Kindle Paperwhite".to_string(),
///     percentage: 67.5,
///     progress: "Page 135 of 200".to_string(),
///     timestamp: 1609459200000,
/// };
/// ```
#[derive(Debug, Archive, Serialize, Deserialize, Default, Clone)]
pub struct Progress {
    /// Unique identifier for the device reporting progress
    pub device_id: String,
    /// Human-readable device name
    pub device: String,
    /// Reading progress as a percentage
    pub percentage: f32,
    /// Textual representation of progress
    pub progress: String,
    /// Unix timestamp in milliseconds when progress was last updated
    pub timestamp: u64,
}
