//! Domain models for KOReader synchronization.
//!
//! This module contains the core business entities used throughout the application.
//!
//! # Models
//!
//! ## [`User`]
//!
//! Represents a user account.
//!
//! ## [`Progress`]
//!
//! Represents reading progress for a specific document on a specific device.
//! Tracks the current position, percentage complete, device information, and timestamp.
//!
//! ## [`Error`]
//!
//! Model-specific errors that can occur during user or progress operations.
//!
//! # Usage Example
//!
//! ```
//! use korrosync::model::{User, Progress};
//!
//! // Create a new user
//! let user = User::new("alice", "secure_password")
//!     .expect("Failed to create user");
//!
//! // Create progress information
//! let progress = Progress {
//!     device_id: "kindle-123".to_string(),
//!     device: "Kindle Paperwhite".to_string(),
//!     percentage: 42.5,
//!     progress: "Page 85 of 200".to_string(),
//!     timestamp: 1704067200000,
//! };
//! ```

mod error;
mod progress;
mod user;

pub use error::Error;
pub use progress::Progress;
pub use user::User;
