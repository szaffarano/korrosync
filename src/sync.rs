use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use bincode::{Decode, Encode, decode_from_slice, encode_to_vec};
use std::{any::type_name, cmp::Ordering, path::Path, sync::Arc};

use redb::{Database, Key, ReadableDatabase, TableDefinition, TypeName, Value};

use crate::error::{Error, Result};

#[derive(Clone)]
pub struct KorrosyncService {
    db: Arc<Database>,
}

const USERS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("users");
const PROGRESS_TABLE: TableDefinition<Bincode<ProgressKey>, Bincode<ProgressValue>> =
    TableDefinition::new("progress");

impl KorrosyncService {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let db = Database::create(path)?;

        // create tables if not exist
        let write_txn = db.begin_write()?;
        write_txn.open_table(USERS_TABLE)?;
        write_txn.open_table(PROGRESS_TABLE)?;
        write_txn.commit()?;

        Ok(Self { db: Arc::new(db) })
    }
}

impl KorrosyncService {
    pub fn get_user(&self, name: impl Into<String>) -> Result<Option<User>> {
        let username = name.into();
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(USERS_TABLE)?;

        let user = table.get(&*username)?.map(|hash| User {
            username,
            password_hash: hash.value().to_string(),
        });

        Ok(user)
    }

    pub fn add_user(&self, user: User) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(USERS_TABLE)?;
            table.insert(&*user.username, &*user.password_hash)?;
        }
        write_txn.commit()?;

        Ok(())
    }

    pub fn update_progress(
        &self,
        user: impl Into<String>,
        document: impl Into<String>,
        progress: ProgressValue,
    ) -> Result<(String, u64)> {
        let user = user.into();
        let document = document.into();
        let key = ProgressKey { document, user };

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(PROGRESS_TABLE)?;
            table.insert(&key, &progress)?;
        }
        write_txn.commit()?;

        Ok((key.document, progress.timestamp))
    }

    pub fn get_progress(&self, user: String, document: String) -> Result<ProgressValue> {
        let key = ProgressKey { document, user };

        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(PROGRESS_TABLE)?;

        if let Some(progress) = table.get(&key)? {
            Ok(progress.value())
        } else {
            Err(Error::NotFound(
                "Progress not found for document".to_string(),
            ))
        }
    }
}

pub struct User {
    username: String,
    password_hash: String,
}

#[derive(Debug, Decode, Encode, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProgressKey {
    pub document: String,
    pub user: String,
}

#[derive(Debug, Encode, Decode)]
pub struct ProgressValue {
    pub device_id: String,
    pub device: String,
    pub percentage: f32,
    pub progress: String,
    pub timestamp: u64,
}

impl User {
    /// Creates a new user with the given username and plain password.
    /// The password is hashed before storing.
    /// More info: https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Result<Self> {
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
        })
    }

    pub fn check(&self, password: impl AsRef<str>) -> Result<()> {
        let parsed_hash = PasswordHash::new(&self.password_hash)?;
        let argon2 = Argon2::default();

        argon2.verify_password(password.as_ref().as_bytes(), &parsed_hash)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Bincode<T>(pub T);

impl<T> Value for Bincode<T>
where
    T: std::fmt::Debug + Encode + Decode<()>,
{
    type SelfType<'a>
        = T
    where
        Self: 'a;

    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        decode_from_slice(data, bincode::config::standard())
            .expect("Failed to decode bincode value")
            .0
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        encode_to_vec(value, bincode::config::standard()).expect("Failed to encode bincode value")
    }

    fn type_name() -> TypeName {
        TypeName::new(&format!("Bincode<{}>", type_name::<T>()))
    }
}

impl<T> Key for Bincode<T>
where
    T: std::fmt::Debug + Decode<()> + Encode + Ord,
{
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        Self::from_bytes(data1).cmp(&Self::from_bytes(data2))
    }
}
