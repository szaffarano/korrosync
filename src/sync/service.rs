//! Database service layer for KoReader synchronization.
//!
//! This module provides the [`KorrosyncService`] which manages persistent storage
//! for user authentication and reading progress synchronization using an embedded
//! redb database.
//!
//! # Database Schema
//!
//! The service maintains two tables:
//!
//! - **users-v1**: Stores user credentials with username as key and [`User`] as value
//! - **progress-v1**: Stores reading progress with composite key (document, user) and [`Progress`] as value
//!
//! # Thread Safety
//!
//! The service is thread-safe and can be cloned cheaply (uses `Arc<Database>` internally).
//! Multiple clones can safely operate on the same database concurrently.
//!
//! # Example
//!
//! ```no_run
//! use korrosync::sync::service::{KorrosyncService, Progress};
//! use korrosync::model::User;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize the service with a database file
//! let service = KorrosyncService::new("korrosync.db")?;
//!
//! // Add a user
//! let user = User::new("alice", "password")?;
//! service.add_user(&user)?;
//!
//! // Update reading progress
//! let progress = Progress {
//!     device_id: "device-123".to_string(),
//!     device: "Kindle".to_string(),
//!     percentage: 45.5,
//!     progress: "Chapter 5".to_string(),
//!     timestamp: 1609459200000,
//! };
//! service.update_progress("alice", "book.epub", progress)?;
//! # Ok(())
//! # }
//! ```

use bincode::{Decode, Encode};
use std::{fs::create_dir_all, path::Path, sync::Arc};

use redb::{Database, ReadableDatabase, TableDefinition};

use crate::{
    model::User,
    sync::{error::ServiceError, serialization::Bincode},
};

// Table definitions with versioning for future migration support
// TODO: implement migrations for table definitions. So far we don't need it but it could be useful in the future
const USERS_TABLE: TableDefinition<&str, Bincode<User>> = TableDefinition::new("users-v1");
const PROGRESS_TABLE: TableDefinition<Bincode<ProgressKey>, Bincode<Progress>> =
    TableDefinition::new("progress-v1");

/// Main service for managing KOReader synchronization data.
///
/// This service provides a high-level API for user authentication and reading progress
/// synchronization. It wraps an embedded redb database and provides transactional
/// guarantees for all operations.
///
/// # Cloning
///
/// The service uses `Arc<Database>` internally, making clones cheap and safe.
/// All clones share the same underlying database.
///
/// # Thread Safety
///
/// This struct is `Clone` and can be safely shared across threads. The underlying
/// redb database handles concurrent access with MVCC (Multi-Version Concurrency Control).
#[derive(Clone)]
pub struct KorrosyncService {
    db: Arc<Database>,
}

/// Composite key for the progress table.
///
/// Combines document identifier and username to uniquely identify
/// a user's progress in a specific document.
#[derive(Debug, Decode, Encode, PartialEq, Eq, PartialOrd, Ord, Default)]
struct ProgressKey {
    document: String,
    user: String,
}

/// Reading progress information for a document.
///
/// This struct contains all the information about a user's reading progress
/// in a specific document, including device information and the current position.
///
/// # Fields
///
/// * `device_id` - Unique identifier for the device reporting progress
/// * `device` - Human-readable device name (e.g., "Kindle", "Kobo")
/// * `percentage` - Reading progress as a percentage (0.0 - 100.0)
/// * `progress` - Textual representation of progress (e.g., page number, chapter)
/// * `timestamp` - Unix timestamp in milliseconds when progress was last updated
///
/// # Example
///
/// ```
/// use korrosync::sync::service::Progress;
///
/// let progress = Progress {
///     device_id: "device-123".to_string(),
///     device: "Kindle Paperwhite".to_string(),
///     percentage: 67.5,
///     progress: "Page 135 of 200".to_string(),
///     timestamp: 1609459200000,
/// };
/// ```
#[derive(Debug, Encode, Decode, Default, Clone)]
pub struct Progress {
    pub device_id: String,
    pub device: String,
    pub percentage: f32,
    pub progress: String,
    pub timestamp: u64,
}

impl KorrosyncService {
    /// Creates a new KorrosyncService with a database at the specified path.
    ///
    /// This method initializes the embedded redb database and creates the required
    /// tables if they don't already exist. If the database file already exists,
    /// it will be opened and reused.
    ///
    /// **Parent directories are created automatically** if they don't exist, so you can
    /// safely provide paths like `"data/db/korrosync.db"` without pre-creating the folders.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the database file (will be created if it doesn't exist)
    ///
    /// # Returns
    ///
    /// Returns a new `KorrosyncService` instance ready for use.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Parent directories cannot be created (permission issues, invalid path)
    /// - The database file cannot be created or opened
    /// - There are permission issues accessing the file or directories
    /// - The database is corrupted or incompatible
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use korrosync::sync::service::KorrosyncService;
    ///
    /// // Create a service with a simple database file
    /// let service = KorrosyncService::new("korrosync.db")?;
    ///
    /// // Create a service with nested directories (will be created automatically)
    /// let service = KorrosyncService::new("data/databases/korrosync.db")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(path: impl AsRef<Path>) -> Result<Self, ServiceError> {
        let path = path.as_ref();

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            create_dir_all(parent)?;
        }

        let db = Database::create(path).map_err(ServiceError::db)?;

        // create tables if not exist
        let write_txn = db.begin_write().map_err(ServiceError::db)?;
        write_txn
            .open_table(USERS_TABLE)
            .map_err(ServiceError::db)?;
        write_txn
            .open_table(PROGRESS_TABLE)
            .map_err(ServiceError::db)?;
        write_txn.commit().map_err(ServiceError::db)?;

        Ok(Self { db: Arc::new(db) })
    }
}

impl KorrosyncService {
    /// Retrieves a user by username from the database.
    ///
    /// This method performs a read-only transaction to fetch the user.
    /// Multiple concurrent reads are allowed and won't block each other.
    ///
    /// # Arguments
    ///
    /// * `name` - The username to look up
    ///
    /// # Returns
    ///
    /// - `Ok(Some(user))` - User found with the given username
    /// - `Ok(None)` - No user exists with the given username
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The database cannot be accessed
    /// - The table cannot be opened
    /// - Data corruption is detected
    ///
    /// # Example
    ///
    /// ```no_run
    /// use korrosync::sync::service::KorrosyncService;
    ///
    /// let service = KorrosyncService::new("korrosync.db")?;
    ///
    /// match service.get_user("alice")? {
    ///     Some(user) => println!("Found user: {}", user.username()),
    ///     None => println!("User not found"),
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn get_user(&self, name: impl Into<String>) -> Result<Option<User>, ServiceError> {
        let username = name.into();
        let read_txn = self.db.begin_read().map_err(ServiceError::db)?;
        let table = read_txn.open_table(USERS_TABLE).map_err(ServiceError::db)?;

        let user = table
            .get(&*username)
            .map_err(ServiceError::db)?
            .map(|hash| hash.value());

        Ok(user)
    }

    /// Adds a new user or updates an existing user in the database.
    ///
    /// This method performs a write transaction to insert or update the user.
    /// If a user with the same username already exists, they will be overwritten.
    /// The transaction is atomic and will be rolled back if any error occurs.
    ///
    /// # Arguments
    ///
    /// * `user` - Reference to the user to add or update
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The database cannot be accessed
    /// - The write transaction fails
    /// - There's insufficient disk space
    /// - Serialization of the user fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use korrosync::sync::service::KorrosyncService;
    /// use korrosync::model::User;
    ///
    /// let service = KorrosyncService::new("korrosync.db")?;
    /// let user = User::new("alice", "secure_password")?;
    ///
    /// service.add_user(&user)?;
    /// println!("User added successfully");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Transaction Behavior
    ///
    /// This operation is transactional. If an error occurs after the insert
    /// but before commit, the entire transaction will be rolled back.
    pub fn add_user(&self, user: &User) -> Result<(), ServiceError> {
        let write_txn = self.db.begin_write().map_err(ServiceError::db)?;
        {
            let mut table = write_txn
                .open_table(USERS_TABLE)
                .map_err(ServiceError::db)?;
            table
                .insert(user.username(), user)
                .map_err(ServiceError::db)?;
        }
        write_txn.commit().map_err(ServiceError::db)?;

        Ok(())
    }

    /// Updates or creates reading progress for a user's document.
    ///
    /// This method stores the reading progress for a specific user and document combination.
    /// If progress already exists for this combination, it will be overwritten with the new data.
    /// The operation is atomic and transactional.
    ///
    /// # Arguments
    ///
    /// * `user` - The username of the user
    /// * `document` - The document identifier (typically filename or path)
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
    /// Returns an error if:
    /// - The database cannot be accessed
    /// - The write transaction fails
    /// - Serialization of the progress data fails
    /// - There's insufficient disk space
    ///
    /// # Example
    ///
    /// ```no_run
    /// use korrosync::sync::service::{KorrosyncService, Progress};
    ///
    /// let service = KorrosyncService::new("korrosync.db")?;
    ///
    /// let progress = Progress {
    ///     device_id: "device-123".to_string(),
    ///     device: "Kindle".to_string(),
    ///     percentage: 45.5,
    ///     progress: "Page 91 of 200".to_string(),
    ///     timestamp: 1609459200000,
    /// };
    ///
    /// let (doc, ts) = service.update_progress("alice", "book.epub", progress)?;
    /// println!("Updated progress for {} at timestamp {}", doc, ts);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Transaction Behavior
    ///
    /// This operation is transactional. The progress will only be committed if
    /// the entire operation succeeds. If an error occurs, no changes will be made.
    pub fn update_progress(
        &self,
        user: impl Into<String>,
        document: impl Into<String>,
        progress: Progress,
    ) -> Result<(String, u64), ServiceError> {
        let user = user.into();
        let document = document.into();
        let key = ProgressKey { document, user };

        let write_txn = self.db.begin_write().map_err(ServiceError::db)?;
        {
            let mut table = write_txn
                .open_table(PROGRESS_TABLE)
                .map_err(ServiceError::db)?;
            table.insert(&key, &progress).map_err(ServiceError::db)?;
        }
        write_txn.commit().map_err(ServiceError::db)?;

        Ok((key.document, progress.timestamp))
    }

    /// Retrieves reading progress for a specific user and document.
    ///
    /// This method performs a read-only transaction to fetch the progress information.
    /// Multiple concurrent reads are allowed and won't block each other.
    ///
    /// # Arguments
    ///
    /// * `user` - The username of the user
    /// * `document` - The document identifier to look up
    ///
    /// # Returns
    ///
    /// Returns the `Progress` information if found.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The database cannot be accessed
    /// - No progress exists for the given user/document combination ([`Error::NotFound`])
    /// - Data corruption is detected
    /// - Deserialization fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use korrosync::sync::service::KorrosyncService;
    ///
    /// let service = KorrosyncService::new("korrosync.db")?;
    ///
    /// match service.get_progress("alice".to_string(), "book.epub".to_string()) {
    ///     Ok(progress) => {
    ///         println!("Progress: {}% on device {}",
    ///                  progress.percentage, progress.device);
    ///     }
    ///     Err(e) => println!("No progress found: {}", e),
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn get_progress(&self, user: String, document: String) -> Result<Progress, ServiceError> {
        let key = ProgressKey { document, user };

        let read_txn = self.db.begin_read().map_err(ServiceError::db)?;
        let table = read_txn
            .open_table(PROGRESS_TABLE)
            .map_err(ServiceError::db)?;

        if let Some(progress) = table.get(&key).map_err(ServiceError::db)? {
            Ok(progress.value())
        } else {
            Err(ServiceError::NotFound(
                "Progress not found for document".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    // === Test Helper Functions ===

    fn create_test_service() -> KorrosyncService {
        let db = NamedTempFile::new().expect("Failed to create temp file");
        KorrosyncService::new(db.path()).expect("Failed to create service")
    }

    fn create_test_user(username: &str) -> User {
        User::new(username, "test_password").expect("Failed to create user")
    }

    fn create_test_progress() -> Progress {
        Progress {
            device_id: "device-123".to_string(),
            device: "Kindle".to_string(),
            percentage: 45.5,
            progress: "Page 91 of 200".to_string(),
            timestamp: 1609459200000,
        }
    }

    // === Service Initialization Tests ===

    #[test]
    fn test_new_creates_service_with_simple_path() {
        let db = NamedTempFile::new().expect("Failed to create temp file");
        let service = KorrosyncService::new(db.path());
        assert!(service.is_ok(), "Service creation should succeed");
    }

    #[test]
    fn test_new_creates_parent_directories() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("nested/dirs/korrosync.db");

        let service = KorrosyncService::new(&db_path);
        assert!(
            service.is_ok(),
            "Service should create parent directories automatically"
        );
        assert!(
            db_path.parent().unwrap().exists(),
            "Parent directories should exist"
        );
    }

    #[test]
    fn test_new_opens_existing_database() {
        let db = NamedTempFile::new().expect("Failed to create temp file");
        let db_path = db.path().to_path_buf();

        // Create first service and add a user
        {
            let service = KorrosyncService::new(&db_path).expect("Failed to create service");
            let user = create_test_user("alice");
            service.add_user(&user).expect("Failed to add user");
        }

        // Reopen the same database
        let service = KorrosyncService::new(&db_path).expect("Failed to reopen database");
        let retrieved = service
            .get_user("alice")
            .expect("Failed to get user")
            .expect("User should exist");

        assert_eq!(retrieved.username(), "alice");
    }

    // === User CRUD Operation Tests ===

    #[test]
    fn test_add_and_get_user() {
        let service = create_test_service();
        let user = create_test_user("alice");

        service.add_user(&user).expect("Failed to add user");

        let retrieved = service
            .get_user("alice")
            .expect("Failed to get user")
            .expect("User not found");

        assert_eq!(retrieved.username(), "alice");
    }

    #[test]
    fn test_get_user_returns_none_when_not_exists() {
        let service = create_test_service();

        let result = service
            .get_user("nonexistent")
            .expect("Query should not fail");

        assert!(result.is_none(), "Should return None for non-existent user");
    }

    #[test]
    fn test_add_user_overwrites_existing() {
        let service = create_test_service();
        let user1 = User::new("alice", "password1").expect("Failed to create user1");
        let user2 = User::new("alice", "password2").expect("Failed to create user2");

        service.add_user(&user1).expect("Failed to add user1");
        service.add_user(&user2).expect("Failed to add user2");

        let retrieved = service
            .get_user("alice")
            .expect("Failed to get user")
            .expect("User not found");

        // Verify the second password works (overwrote the first)
        assert!(
            retrieved.check("password2").is_ok(),
            "Should verify with second password"
        );
        assert!(
            retrieved.check("password1").is_err(),
            "Should not verify with first password"
        );
    }

    #[test]
    fn test_username_verification() {
        let service = create_test_service();
        let user = create_test_user("alice");

        service.add_user(&user).expect("Failed to add user");

        let retrieved = service
            .get_user("alice")
            .expect("Failed to get user")
            .expect("User not found");

        assert_eq!(
            retrieved.username(),
            "alice",
            "Username should match exactly"
        );
    }

    #[test]
    fn test_username_case_sensitive() {
        let service = create_test_service();
        let user = create_test_user("Alice");

        service.add_user(&user).expect("Failed to add user");

        let result = service.get_user("alice").expect("Query should not fail");
        assert!(result.is_none(), "Username lookup should be case-sensitive");

        let result = service.get_user("Alice").expect("Query should not fail");
        assert!(result.is_some(), "Exact case should match");
    }

    // === Progress CRUD Operation Tests ===

    #[test]
    fn test_update_and_get_progress() {
        let service = create_test_service();
        let progress = create_test_progress();

        service
            .update_progress("alice", "book.epub", progress)
            .expect("Failed to update progress");

        let retrieved = service
            .get_progress("alice".to_string(), "book.epub".to_string())
            .expect("Failed to get progress");

        assert_eq!(retrieved.device_id, "device-123");
        assert_eq!(retrieved.device, "Kindle");
        assert_eq!(retrieved.percentage, 45.5);
        assert_eq!(retrieved.progress, "Page 91 of 200");
        assert_eq!(retrieved.timestamp, 1609459200000);
    }

    #[test]
    fn test_update_progress_returns_document_and_timestamp() {
        let service = create_test_service();
        let progress = create_test_progress();

        let (doc, ts) = service
            .update_progress("alice", "book.epub", progress)
            .expect("Failed to update progress");

        assert_eq!(doc, "book.epub");
        assert_eq!(ts, 1609459200000);
    }

    #[test]
    fn test_update_progress_overwrites_existing() {
        let service = create_test_service();

        let progress1 = Progress {
            device_id: "device-1".to_string(),
            device: "Kindle".to_string(),
            percentage: 30.0,
            progress: "Page 60".to_string(),
            timestamp: 1000000,
        };

        let progress2 = Progress {
            device_id: "device-2".to_string(),
            device: "Kobo".to_string(),
            percentage: 70.0,
            progress: "Page 140".to_string(),
            timestamp: 2000000,
        };

        service
            .update_progress("alice", "book.epub", progress1)
            .expect("Failed to update progress first time");

        service
            .update_progress("alice", "book.epub", progress2)
            .expect("Failed to update progress second time");

        let retrieved = service
            .get_progress("alice".to_string(), "book.epub".to_string())
            .expect("Failed to get progress");

        assert_eq!(retrieved.device_id, "device-2");
        assert_eq!(retrieved.percentage, 70.0);
        assert_eq!(retrieved.timestamp, 2000000);
    }

    #[test]
    fn test_get_progress_not_found_error() {
        let service = create_test_service();

        let result = service.get_progress("alice".to_string(), "nonexistent.epub".to_string());

        assert!(
            result.is_err(),
            "Should return error for non-existent progress"
        );
        match result {
            Err(ServiceError::NotFound(_)) => {} // Expected
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_progress_is_user_specific() {
        let service = create_test_service();
        let progress = create_test_progress();

        // Same document, different users
        service
            .update_progress("alice", "book.epub", progress.clone())
            .expect("Failed to update alice's progress");

        let mut bob_progress = progress;
        bob_progress.percentage = 80.0;
        service
            .update_progress("bob", "book.epub", bob_progress)
            .expect("Failed to update bob's progress");

        // Verify each user has their own progress
        let alice_retrieved = service
            .get_progress("alice".to_string(), "book.epub".to_string())
            .expect("Failed to get alice's progress");

        let bob_retrieved = service
            .get_progress("bob".to_string(), "book.epub".to_string())
            .expect("Failed to get bob's progress");

        assert_eq!(alice_retrieved.percentage, 45.5);
        assert_eq!(bob_retrieved.percentage, 80.0);
    }

    #[test]
    fn test_progress_is_document_specific() {
        let service = create_test_service();

        let progress1 = Progress {
            device_id: "device-1".to_string(),
            device: "Kindle".to_string(),
            percentage: 30.0,
            progress: "Page 60".to_string(),
            timestamp: 1000000,
        };

        let progress2 = Progress {
            device_id: "device-1".to_string(),
            device: "Kindle".to_string(),
            percentage: 70.0,
            progress: "Page 140".to_string(),
            timestamp: 2000000,
        };

        // Same user, different documents
        service
            .update_progress("alice", "book1.epub", progress1)
            .expect("Failed to update progress for book1");

        service
            .update_progress("alice", "book2.epub", progress2)
            .expect("Failed to update progress for book2");

        // Verify each document has separate progress
        let book1_retrieved = service
            .get_progress("alice".to_string(), "book1.epub".to_string())
            .expect("Failed to get book1 progress");

        let book2_retrieved = service
            .get_progress("alice".to_string(), "book2.epub".to_string())
            .expect("Failed to get book2 progress");

        assert_eq!(book1_retrieved.percentage, 30.0);
        assert_eq!(book2_retrieved.percentage, 70.0);
    }

    #[test]
    fn test_progress_all_fields_stored_correctly() {
        let service = create_test_service();

        let progress = Progress {
            device_id: "unique-device-id-123".to_string(),
            device: "Kindle Paperwhite 11th Gen".to_string(),
            percentage: 67.89,
            progress: "Chapter 12, Page 345 of 512".to_string(),
            timestamp: 1704067200000,
        };

        service
            .update_progress("testuser", "detailed-book.pdf", progress)
            .expect("Failed to update progress");

        let retrieved = service
            .get_progress("testuser".to_string(), "detailed-book.pdf".to_string())
            .expect("Failed to get progress");

        assert_eq!(retrieved.device_id, "unique-device-id-123");
        assert_eq!(retrieved.device, "Kindle Paperwhite 11th Gen");
        assert_eq!(retrieved.percentage, 67.89);
        assert_eq!(retrieved.progress, "Chapter 12, Page 345 of 512");
        assert_eq!(retrieved.timestamp, 1704067200000);
    }

    // === Thread Safety and Concurrency Tests ===

    #[tokio::test]
    async fn test_service_clone_is_thread_safe() {
        let service = create_test_service();
        let user = create_test_user("alice");
        service.add_user(&user).expect("Failed to add user");

        let service_clone = service.clone();

        let handle = tokio::spawn(async move {
            service_clone
                .get_user("alice")
                .expect("Failed to get user")
                .expect("User not found")
                .username()
                .to_string()
        });

        let username = handle.await.expect("Task failed");
        assert_eq!(username, "alice");
    }

    #[tokio::test]
    async fn test_concurrent_reads() {
        let service = create_test_service();
        let user = create_test_user("alice");
        service.add_user(&user).expect("Failed to add user");

        let mut handles = vec![];

        // Spawn 10 concurrent read tasks
        for _ in 0..10 {
            let service_clone = service.clone();
            let handle = tokio::spawn(async move {
                service_clone
                    .get_user("alice")
                    .expect("Failed to get user")
                    .expect("User not found")
                    .username()
                    .to_string()
            });
            handles.push(handle);
        }

        // All reads should succeed
        for handle in handles {
            let username = handle.await.expect("Task failed");
            assert_eq!(username, "alice");
        }
    }

    #[tokio::test]
    async fn test_concurrent_writes() {
        let service = create_test_service();

        let mut handles = vec![];

        // Spawn 10 concurrent write tasks for different users
        for i in 0..10 {
            let service_clone = service.clone();
            let username = format!("user{}", i);
            let handle = tokio::spawn(async move {
                let user = User::new(&username, "password").expect("Failed to create user");
                service_clone.add_user(&user).expect("Failed to add user");
                username
            });
            handles.push(handle);
        }

        // Wait for all writes to complete
        for handle in handles {
            handle.await.expect("Task failed");
        }

        // Verify all users were created
        for i in 0..10 {
            let username = format!("user{}", i);
            let result = service.get_user(&username).expect("Failed to get user");
            assert!(result.is_some(), "User {} should exist", username);
        }
    }

    // === Edge Case Tests ===

    #[test]
    fn test_empty_username_or_document() {
        let service = create_test_service();
        let progress = create_test_progress();

        let result = service.update_progress("", "book.epub", progress.clone());
        assert!(result.is_ok(), "Empty username should be allowed");

        let result = service.update_progress("alice", "", progress);
        assert!(result.is_ok(), "Empty document should be allowed");
    }

    #[test]
    fn test_special_characters_in_identifiers() {
        let service = create_test_service();
        let progress = create_test_progress();

        let special_user = "user@example.com";
        let special_doc = "book-title_v2.0 [final] (2024).epub";

        service
            .update_progress(special_user, special_doc, progress)
            .expect("Should handle special characters");

        let retrieved = service
            .get_progress(special_user.to_string(), special_doc.to_string())
            .expect("Should retrieve with special characters");

        assert_eq!(retrieved.device_id, "device-123");
    }

    #[test]
    fn test_boundary_values() {
        let service = create_test_service();

        let progress_0 = Progress {
            device_id: "device-1".to_string(),
            device: "Test".to_string(),
            percentage: 0.0,
            progress: "Start".to_string(),
            timestamp: 0,
        };

        let progress_100 = Progress {
            device_id: "device-1".to_string(),
            device: "Test".to_string(),
            percentage: 100.0,
            progress: "End".to_string(),
            timestamp: u64::MAX,
        };

        service
            .update_progress("alice", "doc1", progress_0)
            .expect("Should handle 0% and timestamp 0");

        service
            .update_progress("alice", "doc2", progress_100)
            .expect("Should handle 100% and max timestamp");

        let retrieved_0 = service
            .get_progress("alice".to_string(), "doc1".to_string())
            .expect("Should retrieve 0%");
        assert_eq!(retrieved_0.percentage, 0.0);
        assert_eq!(retrieved_0.timestamp, 0);

        let retrieved_100 = service
            .get_progress("alice".to_string(), "doc2".to_string())
            .expect("Should retrieve 100%");
        assert_eq!(retrieved_100.percentage, 100.0);
        assert_eq!(retrieved_100.timestamp, u64::MAX);
    }

    #[test]
    fn test_very_long_identifiers() {
        let service = create_test_service();
        let progress = create_test_progress();

        let long_username = "a".repeat(1000);
        let long_document = "b".repeat(1000);

        service
            .update_progress(&long_username, &long_document, progress)
            .expect("Should handle very long identifiers");

        let retrieved = service
            .get_progress(long_username.clone(), long_document.clone())
            .expect("Should retrieve with long identifiers");

        assert_eq!(retrieved.device_id, "device-123");
    }

    #[test]
    fn test_empty_progress_string() {
        let service = create_test_service();

        let progress = Progress {
            device_id: "device-1".to_string(),
            device: "Test".to_string(),
            percentage: 50.0,
            progress: "".to_string(),
            timestamp: 1000000,
        };

        service
            .update_progress("alice", "book.epub", progress)
            .expect("Should handle empty progress string");

        let retrieved = service
            .get_progress("alice".to_string(), "book.epub".to_string())
            .expect("Should retrieve progress");

        assert_eq!(retrieved.progress, "");
    }
}
