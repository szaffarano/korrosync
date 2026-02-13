//! Redb-based implementation of KOReader synchronization service.
//!
//! This module provides a [`KorrosyncService`] implementation using the embedded
//! redb database for persistent storage of user authentication and reading progress.
//!
//! # Database Schema
//!
//! The implementation maintains two tables:
//!
//! - **users-v2**: Stores user credentials with username as key and [`User`] as value
//! - **progress-v2**: Stores reading progress with composite key (document, user) and [`Progress`] as value
//!
//! # Example
//!
//! ```no_run
//! use korrosync::service::db::{KorrosyncServiceRedb, KorrosyncService};
//! use korrosync::model::{User, Progress};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize the service with a database file
//! let service = KorrosyncServiceRedb::new("korrosync.db")?;
//!
//! // Add a user
//! let user = User::new("alice", "password")?;
//! service.create_or_update_user(user)?;
//!
//! // Update reading progress
//! let progress = Progress {
//!     device_id: "device-123".to_string(),
//!     device: "Kindle".to_string(),
//!     percentage: 45.5,
//!     progress: "Chapter 5".to_string(),
//!     timestamp: 1609459200000,
//! };
//! service.update_progress("alice".into(), "book.epub".into(), progress)?;
//! # Ok(())
//! # }
//! ```

use rkyv::{Archive, Deserialize, Serialize};
use std::{fs::create_dir_all, path::Path};

use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

use crate::{
    model::{Progress, User},
    service::{db::KorrosyncService, error::ServiceError, serialization::Rkyv},
};

// Table definitions with versioning for future migration support
// TODO: implement migrations for table definitions. So far we don't need it but it could be useful in the future
const USERS_TABLE: TableDefinition<&str, Rkyv<User>> = TableDefinition::new("users-v2");
const PROGRESS_TABLE: TableDefinition<Rkyv<ProgressKey>, Rkyv<Progress>> =
    TableDefinition::new("progress-v2");

/// Redb-based implementation of KoReader synchronization service.
///
/// This service provides a high-level API for user authentication and reading progress
/// synchronization using an embedded redb database with transactional guarantees.
///
pub struct KorrosyncServiceRedb {
    db: Database,
}

/// Composite key for the progress table.
///
/// Combines document identifier and username to uniquely identify
/// a user's progress in a specific document.
#[derive(Debug, Archive, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
struct ProgressKey {
    document: String,
    user: String,
}

impl KorrosyncServiceRedb {
    /// Creates a new KorrosyncServiceRedb with a database at the specified path.
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
    /// Returns a new `KorrosyncServiceRedb` instance ready for use.
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
    /// use korrosync::service::db::KorrosyncServiceRedb;
    ///
    /// // Create a service with a simple database file
    /// let service = KorrosyncServiceRedb::new("korrosync.db")?;
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(path: impl AsRef<Path>) -> Result<KorrosyncServiceRedb, ServiceError> {
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

        Ok(Self { db })
    }
}

impl KorrosyncService for KorrosyncServiceRedb {
    /// Retrieves a user by username from the database.
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
    /// # Example
    ///
    /// ```no_run
    /// use korrosync::service::db::{KorrosyncService, KorrosyncServiceRedb};
    ///
    /// let service = KorrosyncServiceRedb::new("korrosync.db")?;
    ///
    /// match service.get_user("alice".into())? {
    ///     Some(user) => println!("Found user: {}", user.username()),
    ///     None => println!("User not found"),
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn get_user(&self, name: String) -> Result<Option<User>, ServiceError> {
        let read_txn = self.db.begin_read().map_err(ServiceError::db)?;
        let table = read_txn.open_table(USERS_TABLE).map_err(ServiceError::db)?;

        let user = table
            .get(&*name)
            .map_err(ServiceError::db)?
            .map(|hash| hash.value());

        Ok(user)
    }

    /// Adds a new user or updates an existing user in the database.
    ///
    /// # Arguments
    ///
    /// * `user` - Reference to the user to add or update
    ///
    /// # Example
    ///
    /// ```no_run
    /// use korrosync::service::db::{KorrosyncService, KorrosyncServiceRedb};
    /// use korrosync::model::User;
    ///
    /// let service = KorrosyncServiceRedb::new("korrosync.db")?;
    /// let user = User::new("alice", "secure_password")?;
    ///
    /// service.create_or_update_user(user)?;
    /// println!("User added successfully");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn create_or_update_user(&self, user: User) -> Result<User, ServiceError> {
        let write_txn = self.db.begin_write().map_err(ServiceError::db)?;
        {
            let mut table = write_txn
                .open_table(USERS_TABLE)
                .map_err(ServiceError::db)?;
            table
                .insert(user.username(), &user)
                .map_err(ServiceError::db)?;
        }
        write_txn.commit().map_err(ServiceError::db)?;

        Ok(user)
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
    /// # Example
    ///
    /// ```no_run
    /// use korrosync::service::db::{KorrosyncService, KorrosyncServiceRedb};
    /// use korrosync::model::Progress;
    ///
    /// let service = KorrosyncServiceRedb::new("korrosync.db")?;
    ///
    /// let progress = Progress {
    ///     device_id: "device-123".to_string(),
    ///     device: "Kindle".to_string(),
    ///     percentage: 45.5,
    ///     progress: "Page 91 of 200".to_string(),
    ///     timestamp: 1609459200000,
    /// };
    ///
    /// let (doc, ts) = service.update_progress("alice".into(), "book.epub".into(), progress)?;
    /// println!("Updated progress for {} at timestamp {}", doc, ts);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn update_progress(
        &self,
        user: String,
        document: String,
        progress: Progress,
    ) -> Result<(String, u64), ServiceError> {
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
    /// # Arguments
    ///
    /// * `user` - The username of the user
    /// * `document` - The document identifier to look up
    ///
    /// # Returns
    ///
    /// Returns the `Progress` information if found.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use korrosync::service::db::{KorrosyncService, KorrosyncServiceRedb};
    ///
    /// let service = KorrosyncServiceRedb::new("korrosync.db")?;
    ///
    /// match service.get_progress("alice".to_string(), "book.epub".to_string()) {
    ///     Ok(Some(progress)) => {
    ///         println!("Progress: {}% on device {}",
    ///                  progress.percentage, progress.device);
    ///     }
    ///     Ok(None) => println!("No progress found"),
    ///     Err(e) => println!("Unexpected error: {}", e),
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn get_progress(
        &self,
        user: String,
        document: String,
    ) -> Result<Option<Progress>, ServiceError> {
        let key = ProgressKey { document, user };

        let read_txn = self.db.begin_read().map_err(ServiceError::db)?;
        let table = read_txn
            .open_table(PROGRESS_TABLE)
            .map_err(ServiceError::db)?;

        if let Some(progress) = table.get(&key).map_err(ServiceError::db)? {
            Ok(Some(progress.value()))
        } else {
            Ok(None)
        }
    }

    fn list_users(&self) -> Result<Vec<User>, ServiceError> {
        let read_txn = self.db.begin_read().map_err(ServiceError::db)?;
        let table = read_txn.open_table(USERS_TABLE).map_err(ServiceError::db)?;

        let mut users = Vec::new();
        for entry in table.iter().map_err(ServiceError::db)? {
            let (_key, value) = entry.map_err(ServiceError::db)?;
            users.push(value.value());
        }
        Ok(users)
    }

    fn delete_user(&self, name: String) -> Result<bool, ServiceError> {
        let write_txn = self.db.begin_write().map_err(ServiceError::db)?;
        let existed = {
            let mut table = write_txn
                .open_table(USERS_TABLE)
                .map_err(ServiceError::db)?;
            table.remove(&*name).map_err(ServiceError::db)?.is_some()
        };
        write_txn.commit().map_err(ServiceError::db)?;
        Ok(existed)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    // === Test Helper Functions ===

    fn create_test_service() -> (TempDir, impl KorrosyncService) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let service = KorrosyncServiceRedb::new(db_path).expect("Failed to create service");
        (temp_dir, service)
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
        let service = KorrosyncServiceRedb::new(db.path());
        assert!(service.is_ok(), "Service creation should succeed");
    }

    #[test]
    fn test_new_creates_parent_directories() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("nested/dirs/korrosync.db");

        let service = KorrosyncServiceRedb::new(&db_path);
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
            let service = KorrosyncServiceRedb::new(&db_path).expect("Failed to create service");
            let user = create_test_user("alice");
            service
                .create_or_update_user(user)
                .expect("Failed to add user");
        }

        // Reopen the same database
        let service = KorrosyncServiceRedb::new(&db_path).expect("Failed to reopen database");
        let retrieved = service
            .get_user("alice".into())
            .expect("Failed to get user")
            .expect("User should exist");

        assert_eq!(retrieved.username(), "alice");
    }

    // === User CRUD Operation Tests ===

    #[test]
    fn test_add_and_get_user() {
        let (_temp, service) = create_test_service();
        let user = create_test_user("alice");

        service
            .create_or_update_user(user)
            .expect("Failed to add user");

        let retrieved = service
            .get_user("alice".into())
            .expect("Failed to get user")
            .expect("User not found");

        assert_eq!(retrieved.username(), "alice");
    }

    #[test]
    fn test_get_user_returns_none_when_not_exists() {
        let (_temp, service) = create_test_service();

        let result = service
            .get_user("nonexistent".into())
            .expect("Query should not fail");

        assert!(result.is_none(), "Should return None for non-existent user");
    }

    #[test]
    fn test_add_user_overwrites_existing() {
        let (_temp, service) = create_test_service();
        let user1 = User::new("alice", "password1").expect("Failed to create user1");
        let user2 = User::new("alice", "password2").expect("Failed to create user2");

        service
            .create_or_update_user(user1)
            .expect("Failed to add user1");
        service
            .create_or_update_user(user2)
            .expect("Failed to add user2");

        let retrieved = service
            .get_user("alice".into())
            .expect("Failed to get user")
            .expect("User not found");

        // Verify the second password works (overwrote the first)
        assert!(
            retrieved
                .check("password2")
                .expect("Error checking password"),
            "Should verify with second password"
        );
        assert!(
            !retrieved
                .check("password1")
                .expect("Error checking password"),
            "Should not verify with first password"
        );
    }

    #[test]
    fn test_username_verification() {
        let (_temp, service) = create_test_service();
        let user = create_test_user("alice");

        service
            .create_or_update_user(user)
            .expect("Failed to add user");

        let retrieved = service
            .get_user("alice".into())
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
        let (_temp, service) = create_test_service();
        let user = create_test_user("Alice");

        service
            .create_or_update_user(user)
            .expect("Failed to add user");

        let result = service
            .get_user("alice".into())
            .expect("Query should not fail");
        assert!(result.is_none(), "Username lookup should be case-sensitive");

        let result = service
            .get_user("Alice".into())
            .expect("Query should not fail");
        assert!(result.is_some(), "Exact case should match");
    }

    // === Progress CRUD Operation Tests ===

    #[test]
    fn test_update_and_get_progress() {
        let (_temp, service) = create_test_service();
        let progress = create_test_progress();

        service
            .update_progress("alice".into(), "book.epub".into(), progress)
            .expect("Failed to update progress");

        let retrieved = service
            .get_progress("alice".to_string(), "book.epub".to_string())
            .expect("Failed to get progress");

        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.device_id, "device-123");
        assert_eq!(retrieved.device, "Kindle");
        assert_eq!(retrieved.percentage, 45.5);
        assert_eq!(retrieved.progress, "Page 91 of 200");
        assert_eq!(retrieved.timestamp, 1609459200000);
    }

    #[test]
    fn test_update_progress_returns_document_and_timestamp() {
        let (_temp, service) = create_test_service();
        let progress = create_test_progress();

        let (doc, ts) = service
            .update_progress("alice".into(), "book.epub".into(), progress)
            .expect("Failed to update progress");

        assert_eq!(doc, "book.epub");
        assert_eq!(ts, 1609459200000);
    }

    #[test]
    fn test_update_progress_overwrites_existing() {
        let (_temp, service) = create_test_service();

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
            .update_progress("alice".into(), "book.epub".into(), progress1)
            .expect("Failed to update progress first time");

        service
            .update_progress("alice".into(), "book.epub".into(), progress2)
            .expect("Failed to update progress second time");

        let retrieved = service
            .get_progress("alice".to_string(), "book.epub".to_string())
            .expect("Failed to get progress");

        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.device_id, "device-2");
        assert_eq!(retrieved.percentage, 70.0);
        assert_eq!(retrieved.timestamp, 2000000);
    }

    #[test]
    fn test_get_progress_not_found_error() {
        let (_temp, service) = create_test_service();

        let result = service.get_progress("alice".to_string(), "nonexistent.epub".to_string());

        assert!(
            result.is_ok(),
            "Should return Ok(None) for non-existent progress"
        );
        match result {
            Ok(None) => {} // Expected
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_progress_is_user_specific() {
        let (_temp, service) = create_test_service();
        let progress = create_test_progress();

        // Same document, different users
        service
            .update_progress("alice".into(), "book.epub".into(), progress.clone())
            .expect("Failed to update alice's progress");

        let mut bob_progress = progress;
        bob_progress.percentage = 80.0;
        service
            .update_progress("bob".into(), "book.epub".into(), bob_progress)
            .expect("Failed to update bob's progress");

        // Verify each user has their own progress
        let alice_retrieved = service
            .get_progress("alice".to_string(), "book.epub".to_string())
            .expect("Failed to get alice's progress");

        let bob_retrieved = service
            .get_progress("bob".to_string(), "book.epub".to_string())
            .expect("Failed to get bob's progress");

        assert!(alice_retrieved.is_some());
        assert!(bob_retrieved.is_some());
        let alice_retrieved = alice_retrieved.unwrap();
        let bob_retrieved = bob_retrieved.unwrap();
        assert_eq!(alice_retrieved.percentage, 45.5);
        assert_eq!(bob_retrieved.percentage, 80.0);
    }

    #[test]
    fn test_progress_is_document_specific() {
        let (_temp, service) = create_test_service();

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
            .update_progress("alice".into(), "book1.epub".into(), progress1)
            .expect("Failed to update progress for book1");

        service
            .update_progress("alice".into(), "book2.epub".into(), progress2)
            .expect("Failed to update progress for book2");

        // Verify each document has separate progress
        let book1_retrieved = service
            .get_progress("alice".to_string(), "book1.epub".to_string())
            .expect("Failed to get book1 progress");

        let book2_retrieved = service
            .get_progress("alice".to_string(), "book2.epub".to_string())
            .expect("Failed to get book2 progress");

        assert!(book1_retrieved.is_some());
        assert!(book2_retrieved.is_some());
        assert_eq!(book1_retrieved.unwrap().percentage, 30.0);
        assert_eq!(book2_retrieved.unwrap().percentage, 70.0);
    }

    #[test]
    fn test_progress_all_fields_stored_correctly() {
        let (_temp, service) = create_test_service();

        let progress = Progress {
            device_id: "unique-device-id-123".to_string(),
            device: "Kindle Paperwhite 11th Gen".to_string(),
            percentage: 67.89,
            progress: "Chapter 12, Page 345 of 512".to_string(),
            timestamp: 1704067200000,
        };

        service
            .update_progress("testuser".into(), "detailed-book.pdf".into(), progress)
            .expect("Failed to update progress");

        let retrieved = service
            .get_progress("testuser".to_string(), "detailed-book.pdf".to_string())
            .expect("Failed to get progress");

        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.device_id, "unique-device-id-123");
        assert_eq!(retrieved.device, "Kindle Paperwhite 11th Gen");
        assert_eq!(retrieved.percentage, 67.89);
        assert_eq!(retrieved.progress, "Chapter 12, Page 345 of 512");
        assert_eq!(retrieved.timestamp, 1704067200000);
    }

    // === Thread Safety and Concurrency Tests ===

    #[tokio::test]
    async fn test_service_clone_is_thread_safe() {
        let (_temp, svc) = create_test_service();
        let service = Arc::new(svc);
        let user = create_test_user("alice");
        service
            .create_or_update_user(user)
            .expect("Failed to add user");

        let service_clone = service.clone();

        let handle = tokio::spawn(async move {
            service_clone
                .get_user("alice".into())
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
        let (_temp, svc) = create_test_service();
        let service = Arc::new(svc);
        let user = create_test_user("alice");
        service
            .create_or_update_user(user)
            .expect("Failed to add user");

        let mut handles = vec![];

        // Spawn 10 concurrent read tasks
        for _ in 0..10 {
            let service_clone = service.clone();
            let handle = tokio::spawn(async move {
                service_clone
                    .get_user("alice".into())
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
        let (_temp, svc) = create_test_service();
        let service = Arc::new(svc);

        let mut handles = vec![];

        // Spawn 10 concurrent write tasks for different users
        for i in 0..10 {
            let username = format!("user{}", i);
            let service = service.clone();
            let handle = tokio::spawn(async move {
                let user = User::new(&username, "password").expect("Failed to create user");
                service
                    .create_or_update_user(user)
                    .expect("Failed to add user");
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
            let result = service
                .get_user(username.clone())
                .expect("Failed to get user");
            assert!(result.is_some(), "User {} should exist", username);
        }
    }

    // === Edge Case Tests ===

    #[test]
    fn test_empty_username_or_document() {
        let (_temp, service) = create_test_service();
        let progress = create_test_progress();

        let result = service.update_progress("".into(), "book.epub".into(), progress.clone());
        assert!(result.is_ok(), "Empty username should be allowed");

        let result = service.update_progress("alice".into(), "".into(), progress);
        assert!(result.is_ok(), "Empty document should be allowed");
    }

    #[test]
    fn test_special_characters_in_identifiers() {
        let (_temp, service) = create_test_service();
        let progress = create_test_progress();

        let special_user = "user@example.com";
        let special_doc = "book-title_v2.0 [final] (2024).epub";

        service
            .update_progress(special_user.into(), special_doc.into(), progress)
            .expect("Should handle special characters");

        let retrieved = service
            .get_progress(special_user.into(), special_doc.into())
            .expect("Should retrieve with special characters");

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().device_id, "device-123");
    }

    #[test]
    fn test_boundary_values() {
        let (_temp, service) = create_test_service();

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
            .update_progress("alice".into(), "doc1".into(), progress_0)
            .expect("Should handle 0% and timestamp 0");

        service
            .update_progress("alice".into(), "doc2".into(), progress_100)
            .expect("Should handle 100% and max timestamp");

        let retrieved_0 = service
            .get_progress("alice".to_string(), "doc1".to_string())
            .expect("Should retrieve 0%");

        assert!(retrieved_0.is_some());
        let retrieved_0 = retrieved_0.unwrap();

        assert_eq!(retrieved_0.percentage, 0.0);
        assert_eq!(retrieved_0.timestamp, 0);

        let retrieved_100 = service
            .get_progress("alice".to_string(), "doc2".to_string())
            .expect("Should retrieve 100%");

        assert!(retrieved_100.is_some());
        let retrieved_100 = retrieved_100.unwrap();
        assert_eq!(retrieved_100.percentage, 100.0);
        assert_eq!(retrieved_100.timestamp, u64::MAX);
    }

    #[test]
    fn test_very_long_identifiers() {
        let (_temp, service) = create_test_service();
        let progress = create_test_progress();

        let long_username = "a".repeat(1000);
        let long_document = "b".repeat(1000);

        service
            .update_progress(long_username.clone(), long_document.clone(), progress)
            .expect("Should handle very long identifiers");

        let retrieved = service
            .get_progress(long_username.clone(), long_document.clone())
            .expect("Should retrieve with long identifiers");

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().device_id, "device-123");
    }

    // === List Users Tests ===

    #[test]
    fn test_list_users_empty() {
        let (_temp, service) = create_test_service();
        let users = service.list_users().expect("Failed to list users");
        assert!(users.is_empty());
    }

    #[test]
    fn test_list_users_returns_all() {
        let (_temp, service) = create_test_service();

        service
            .create_or_update_user(create_test_user("alice"))
            .expect("Failed to add alice");
        service
            .create_or_update_user(create_test_user("bob"))
            .expect("Failed to add bob");
        service
            .create_or_update_user(create_test_user("charlie"))
            .expect("Failed to add charlie");

        let users = service.list_users().expect("Failed to list users");
        assert_eq!(users.len(), 3);

        let mut usernames: Vec<&str> = users.iter().map(|u| u.username()).collect();
        usernames.sort();
        assert_eq!(usernames, vec!["alice", "bob", "charlie"]);
    }

    // === Delete User Tests ===

    #[test]
    fn test_delete_user_existing() {
        let (_temp, service) = create_test_service();
        service
            .create_or_update_user(create_test_user("alice"))
            .expect("Failed to add user");

        let deleted = service
            .delete_user("alice".into())
            .expect("Failed to delete user");
        assert!(deleted, "Should return true for existing user");

        let user = service
            .get_user("alice".into())
            .expect("Failed to get user");
        assert!(user.is_none(), "User should no longer exist");
    }

    #[test]
    fn test_delete_user_nonexistent() {
        let (_temp, service) = create_test_service();

        let deleted = service
            .delete_user("nonexistent".into())
            .expect("Failed to delete user");
        assert!(!deleted, "Should return false for non-existent user");
    }

    #[test]
    fn test_empty_progress_string() {
        let (_temp, service) = create_test_service();

        let progress = Progress {
            device_id: "device-1".to_string(),
            device: "Test".to_string(),
            percentage: 50.0,
            progress: "".to_string(),
            timestamp: 1000000,
        };

        service
            .update_progress("alice".into(), "book.epub".into(), progress)
            .expect("Should handle empty progress string");

        let retrieved = service
            .get_progress("alice".to_string(), "book.epub".to_string())
            .expect("Should retrieve progress");

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().progress, "");
    }
}
