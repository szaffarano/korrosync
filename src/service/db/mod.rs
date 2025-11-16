//! Database service layer for KoReader synchronization.
//!
//! This module defines the [`KorrosyncService`] trait which provides an abstract interface
//! for managing persistent storage of user authentication and reading progress synchronization.
//!
//! # Implementations
//!
//! Currently available implementations:
//!
//! - [`KorrosyncServiceRedb`] - Embedded redb database implementation (default)
//!

use crate::{
    model::{Progress, User},
    service::error::ServiceError,
};

pub mod redb;
pub use self::redb::KorrosyncServiceRedb;

/// Trait defining the core database operations for KoReader synchronization.
///
/// This trait provides a database-agnostic interface for managing users and reading progress.
/// Implementations can use any storage backend (embedded databases, SQL databases, etc.)
/// as long as they provide these operations.
///
/// # Design
///
/// The trait uses owned `String` parameters for flexibility and to avoid lifetime issues
/// when used with trait objects (`Arc<dyn KorrosyncService>`).
///
/// # Thread Safety
///
/// Implementations must be thread-safe when wrapped in `Arc`. All methods take `&self`
/// to allow concurrent access from multiple threads.
pub trait KorrosyncService {
    /// Retrieves a user by username.
    ///
    /// # Arguments
    ///
    /// * `name` - The username to look up
    ///
    /// # Returns
    ///
    /// - `Ok(Some(user))` - User found with the given username
    /// - `Ok(None)` - No user exists with the given username
    /// - `Err(...)` - Unexpected database error occurred
    fn get_user(&self, name: String) -> Result<Option<User>, ServiceError>;

    /// Creates a new user or updates an existing one.
    ///
    /// If a user with the same username already exists, they will be overwritten.
    ///
    /// # Arguments
    ///
    /// * `user` - The user to add or update
    ///
    /// # Returns
    ///
    /// - `Ok(User)` - User was successfully created or updated
    /// - `Err(...)` - unexpected database error occurred
    fn create_or_update_user(&self, user: User) -> Result<User, ServiceError>;

    /// Updates or creates reading progress for a user's document.
    ///
    /// If progress already exists for this user/document combination, it will be overwritten.
    ///
    /// # Arguments
    ///
    /// * `user` - The username of the user
    /// * `document` - The document identifier
    /// * `progress` - The progress information to store
    ///
    /// # Returns
    ///
    /// Returns a tuple containing:
    /// - The document identifier (echoed back)
    /// - The timestamp from the progress record
    ///
    /// # Errors
    ///
    /// Returns an error if an unexpected database error occurs.
    fn update_progress(
        &self,
        user: String,
        document: String,
        progress: Progress,
    ) -> Result<(String, u64), ServiceError>;

    /// Retrieves reading progress for a specific user and document.
    ///
    /// # Arguments
    ///
    /// * `user` - The username of the user
    /// * `document` - The document identifier to look up
    ///
    /// # Returns
    ///
    /// - `Ok(Some(progress))` - Progress found for the user/document combination
    /// - `Ok(None)` - No progress exists for this combination
    /// - `Err(...)` - Unexpected error occurred
    fn get_progress(
        &self,
        user: String,
        document: String,
    ) -> Result<Option<Progress>, ServiceError>;
}
