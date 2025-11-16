//! Service layer for business logic and data persistence.
//!
//! This module provides the business logic layer that sits between the domain models
//! and the API layer. It handles data persistence, validation, and business rules.
//!
//! # Architecture
//!
//! The service layer follows a trait-based design pattern to allow for multiple
//! storage implementations while maintaining a consistent interface.
//!
//! ## Submodules
//!
//! ### [`db`]
//!
//! Database abstraction layer providing:
//! - [`db::KorrosyncService`] - Trait defining core database operations
//! - [`db::KorrosyncServiceRedb`] - Default implementation using embedded redb database
//!
//! The database module uses trait objects (`Arc<dyn KorrosyncService>`) to enable
//! runtime polymorphism and future support for alternative storage backends
//! (e.g., PostgreSQL, SQLite, or cloud storage).
//!
//! # Usage Example
//!
//! ```no_run
//! use std::sync::Arc;
//! use korrosync::service::db::{KorrosyncService, KorrosyncServiceRedb};
//! use korrosync::model::{User, Progress};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create service instance
//! let service: Arc<dyn KorrosyncService + Send + Sync> =
//!     Arc::new(KorrosyncServiceRedb::new("korrosync.db")?);
//!
//! // Use the service through the trait interface
//! let user = User::new("alice", "password")?;
//! service.create_or_update_user(user)?;
//!
//! // The service can be cloned (Arc) and shared across threads
//! let service_clone = service.clone();
//! // ... use in async handlers or other threads
//! # Ok(())
//! # }
//! ```

pub mod db;
pub mod error;
pub mod serialization;
