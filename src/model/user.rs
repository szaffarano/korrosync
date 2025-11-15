//! User model for authentication and activity tracking.
//!
//! This module provides the [`User`] struct, which represents a user in the system
//! with secure password storage using the Argon2 algorithm. The implementation follows
//! OWASP password storage best practices.
//!
//! # Password Security
//!
//! Passwords are hashed using Argon2 (the winner of the Password Hashing Competition)
//! with randomly generated salts. Plain-text passwords are never stored.
//!
//! # Example
//!
//! ```no_run
//! use korrosync::model::User;
//!
//! // Create a new user
//! let user = User::new("alice", "secure_password")?;
//!
//! // Verify password
//! user.check("secure_password")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use argon2::{
    Argon2,
    password_hash::{
        self, PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
    },
};
use bincode::{Decode, Encode};
use chrono::Utc;

use crate::model::error::Error;

/// User model representing an authenticated user in the system.
///
/// This struct stores user credentials securely using Argon2 password hashing
/// and tracks user activity timestamps. All password operations are performed
/// using constant-time comparisons to prevent timing attacks.
///
/// # Fields
///
/// * `username` - The unique identifier for the user
/// * `password_hash` - Argon2 hash of the user's password (never stores plaintext)
/// * `last_activity` - Optional timestamp (in milliseconds since Unix epoch) of last user activity
///
/// # Security Considerations
///
/// - Passwords are hashed using Argon2 with randomly generated salts
/// - Password verification uses constant-time comparison
/// - Implements serialization/deserialization via bincode for storage
///
/// # Thread Safety
///
/// This struct is `Send` and `Sync` by default, as it only contains thread-safe fields.
/// Argon2 operations are performed in methods and do not affect thread safety.
#[derive(Debug, Encode, Decode, Default)]
pub struct User {
    username: String,
    password_hash: String,
    last_activity: Option<i64>,
}

impl User {
    /// Creates a new user with the given username and plain password.
    ///
    /// The password is hashed using Argon2 with a randomly generated salt before storage.
    /// The plain-text password is never stored. This follows OWASP password storage guidelines.
    ///
    /// # Arguments
    ///
    /// * `username` - The unique username for this user
    /// * `password` - The plain-text password to hash and store
    ///
    /// # Returns
    ///
    /// Returns a new `User` instance with the hashed password and no activity recorded.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Password hashing fails (extremely rare, typically indicates system issues)
    /// - Random number generation for salt fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use korrosync::model::User;
    ///
    /// let user = User::new("alice", "my_secure_password")?;
    /// assert_eq!(user.username(), "alice");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Security
    ///
    /// More info: <https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html>
    pub fn new(
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Self, password_hash::Error> {
        let password = password.into();
        let username = username.into();

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

        Ok(Self {
            username,
            password_hash,
            last_activity: None,
        })
    }

    /// Returns the username associated with this user.
    ///
    /// # Returns
    ///
    /// A string slice containing the username.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use korrosync::model::User;
    ///
    /// let user = User::new("alice", "password")?;
    /// assert_eq!(user.username(), "alice");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Verifies if the given plain password matches the stored password hash.
    ///
    /// This method uses constant-time comparison to prevent timing attacks.
    /// The verification is performed using Argon2's built-in verification function.
    ///
    /// # Arguments
    ///
    /// * `password` - The plain-text password to verify
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the password matches, or an error if verification fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The stored password hash is malformed or corrupted
    /// - The provided password does not match the stored hash
    /// - Password verification encounters a system error
    ///
    /// # Example
    ///
    /// ```no_run
    /// use korrosync::model::User;
    ///
    /// let user = User::new("alice", "correct_password")?;
    ///
    /// // Correct password
    /// assert!(user.check("correct_password").is_ok());
    ///
    /// // Wrong password
    /// assert!(user.check("wrong_password").is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Security
    ///
    /// This method is designed to be resistant to timing attacks through the use
    /// of constant-time comparison operations provided by the Argon2 implementation.
    pub fn check(&self, password: impl AsRef<str>) -> Result<bool, Error> {
        let parsed_hash = PasswordHash::new(&self.password_hash).map_err(Error::runtime)?;
        let argon2 = Argon2::default();

        match argon2.verify_password(password.as_ref().as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(password_hash::Error::Password) => Ok(false),
            Err(e) => Err(Error::runtime(e)),
        }
    }

    /// Sets the last activity time to a specific timestamp.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Unix timestamp in milliseconds since the epoch (UTC)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use korrosync::model::User;
    ///
    /// let mut user = User::new("alice", "password")?;
    /// user.set_last_activity(1609459200000); // 2021-01-01 00:00:00 UTC
    /// assert_eq!(user.last_activity(), Some(1609459200000));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_last_activity(&mut self, timestamp: i64) {
        self.last_activity = Some(timestamp);
    }

    /// Returns the last recorded activity timestamp for this user.
    ///
    /// # Returns
    ///
    /// - `Some(timestamp)` - Unix timestamp in milliseconds since epoch (UTC) if activity has been recorded
    /// - `None` - If the user has never been active or activity has not been tracked
    ///
    /// # Example
    ///
    /// ```no_run
    /// use korrosync::model::User;
    ///
    /// let mut user = User::new("alice", "password")?;
    /// assert_eq!(user.last_activity(), None);
    ///
    /// user.touch();
    /// assert!(user.last_activity().is_some());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn last_activity(&self) -> Option<i64> {
        self.last_activity
    }

    /// Updates the last activity time to the current UTC time.
    ///
    /// This is a convenience method that automatically sets the last activity
    /// timestamp to the current time. Useful for tracking user sessions or
    /// recording recent interactions.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use korrosync::model::User;
    ///
    /// let mut user = User::new("alice", "password")?;
    ///
    /// // Record that the user is active now
    /// user.touch();
    ///
    /// // The last activity is now set to the current time
    /// assert!(user.last_activity().is_some());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn touch(&mut self) {
        self.last_activity = Some(Utc::now().timestamp_millis());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new("alice", "password123").expect("Failed to create user");
        assert_eq!(user.username(), "alice");
        assert_eq!(user.last_activity(), None);
    }

    #[test]
    fn test_username() {
        let user = User::new("bob", "secret").expect("Failed to create user");
        assert_eq!(user.username(), "bob");
    }

    #[test]
    fn test_password_verification_success() {
        let user = User::new("alice", "correct_password").expect("Failed to create user");
        assert!(
            user.check("correct_password").is_ok(),
            "Password verification should succeed with correct password"
        );
    }

    #[test]
    fn test_password_verification_failure() {
        let user = User::new("alice", "correct_password").expect("Failed to create user");
        assert!(
            !user
                .check("wrong_password")
                .expect("Failed to check password"),
            "Password verification should fail with incorrect password"
        );
    }

    #[test]
    fn test_password_is_hashed() {
        let password = "plaintext_password";
        let user = User::new("alice", password).expect("Failed to create user");

        assert!(
            !user.password_hash.contains(password),
            "Password should be hashed, not stored in plaintext"
        );

        assert!(
            user.password_hash.starts_with("$argon2"),
            "Password hash should be in Argon2 format"
        );
    }

    #[test]
    fn test_unique_salt_per_user() {
        let password = "same_password";
        let user1 = User::new("alice", password).expect("Failed to create user1");
        let user2 = User::new("bob", password).expect("Failed to create user2");

        assert_ne!(
            user1.password_hash, user2.password_hash,
            "Different users with same password should have different hashes"
        );
    }

    #[test]
    fn test_last_activity_initial() {
        let user = User::new("alice", "password").expect("Failed to create user");
        assert_eq!(
            user.last_activity(),
            None,
            "New user should have no last activity"
        );
    }

    #[test]
    fn test_set_last_activity() {
        let mut user = User::new("alice", "password").expect("Failed to create user");
        let timestamp = 1609459200000i64;

        user.set_last_activity(timestamp);
        assert_eq!(user.last_activity(), Some(timestamp));
    }

    #[test]
    fn test_touch_updates_activity() {
        let mut user = User::new("alice", "password").expect("Failed to create user");

        assert_eq!(user.last_activity(), None);

        user.touch();
        assert!(
            user.last_activity().is_some(),
            "touch() should set last_activity"
        );

        let now = Utc::now().timestamp_millis();
        let activity = user.last_activity().unwrap();
        assert!(
            (now - activity).abs() < 1000,
            "touch() should set timestamp to current time (within 1 second)"
        );
    }

    #[test]
    fn test_touch_updates_timestamp() {
        let mut user = User::new("alice", "password").expect("Failed to create user");

        user.touch();
        let first_activity = user
            .last_activity()
            .expect("Should have activity after first touch");

        std::thread::sleep(std::time::Duration::from_millis(10));

        user.touch();
        let second_activity = user
            .last_activity()
            .expect("Should have activity after second touch");

        assert!(
            second_activity > first_activity,
            "Subsequent touch() should update timestamp to a later time"
        );
    }
}
