use rkyv::{
    Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize,
    api::high::{HighDeserializer, HighSerializer, access},
    deserialize,
    rancor::Error,
    ser::allocator::ArenaHandle,
    util::AlignedVec,
};

use redb::{Key, TypeName, Value};
use std::{any::type_name, cmp::Ordering};

#[derive(Debug)]
pub(crate) struct Rkyv<T>(T);

impl<T> Value for Rkyv<T>
where
    T: std::fmt::Debug + Default + Archive,
    T::Archived: RkyvDeserialize<T, HighDeserializer<Error>>
        + rkyv::Portable
        + for<'a> rkyv::bytecheck::CheckBytes<rkyv::api::high::HighValidator<'a, Error>>,
    for<'a> T: RkyvSerialize<HighSerializer<AlignedVec, ArenaHandle<'a>, Error>>,
{
    type SelfType<'a>
        = T
    where
        Self: 'a;

    type AsBytes<'a>
        = AlignedVec
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        if data.is_empty() {
            return T::default();
        }

        match access::<T::Archived, Error>(data) {
            Ok(archived) => deserialize::<T, Error>(archived).unwrap_or_else(|e| {
                tracing::warn!("Failed to deserialize data: {}, using default value", e);
                T::default()
            }),
            Err(e) => {
                tracing::warn!(
                    "Bytecheck validation failed: {}. Data may be corrupted, using default value",
                    e
                );
                T::default()
            }
        }
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        rkyv::to_bytes::<Error>(value).unwrap_or_else(|e| {
            tracing::warn!(
                "Failed to serialize value of type {}: {}. Returning empty AlignedVec",
                type_name::<T>(),
                e
            );
            AlignedVec::new()
        })
    }

    fn type_name() -> TypeName {
        TypeName::new(&format!("Rkyv<{}>", type_name::<T>()))
    }
}

impl<T> Key for Rkyv<T>
where
    T: std::fmt::Debug + Default + Archive + Ord,
    T::Archived: RkyvDeserialize<T, HighDeserializer<Error>>
        + rkyv::Portable
        + for<'a> rkyv::bytecheck::CheckBytes<rkyv::api::high::HighValidator<'a, Error>>,
    for<'a> T: RkyvSerialize<HighSerializer<AlignedVec, ArenaHandle<'a>, Error>>,
{
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        Self::from_bytes(data1).cmp(&Self::from_bytes(data2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Progress, User};

    #[test]
    fn from_bytes_empty_returns_default() {
        let user = <Rkyv<User> as Value>::from_bytes(&[]);
        assert_eq!(user.username(), "");
        assert_eq!(user.last_activity(), None);

        let progress = <Rkyv<Progress> as Value>::from_bytes(&[]);
        assert_eq!(progress.device_id, "");
        assert_eq!(progress.percentage, 0.0);
    }

    #[test]
    fn from_bytes_corrupt_data_returns_default() {
        let corrupt = [0u8, 1, 2, 3, 4, 5, 6, 7];
        let user = <Rkyv<User> as Value>::from_bytes(&corrupt);
        assert_eq!(user.username(), "");
    }

    #[test]
    fn round_trip_user_and_progress() {
        let user = User::new("alice", "password").expect("user");
        let bytes = <Rkyv<User> as Value>::as_bytes(&user);
        let decoded = <Rkyv<User> as Value>::from_bytes(bytes.as_slice());
        assert_eq!(decoded.username(), "alice");
        assert!(decoded.check("password").unwrap());

        let progress = Progress {
            device_id: "d1".into(),
            device: "Kindle".into(),
            percentage: 12.5,
            progress: "p".into(),
            timestamp: 42,
        };
        let bytes = <Rkyv<Progress> as Value>::as_bytes(&progress);
        let decoded = <Rkyv<Progress> as Value>::from_bytes(bytes.as_slice());
        assert_eq!(decoded.device_id, "d1");
        assert_eq!(decoded.percentage, 12.5);
    }

    #[test]
    fn compare_orders_by_deserialized_value() {
        #[derive(
            Debug, Default, PartialEq, Eq, PartialOrd, Ord, Archive, RkyvSerialize, RkyvDeserialize,
        )]
        struct SortKey(u32);

        let a = <Rkyv<SortKey> as Value>::as_bytes(&SortKey(1));
        let b = <Rkyv<SortKey> as Value>::as_bytes(&SortKey(2));
        assert_eq!(
            <Rkyv<SortKey> as Key>::compare(a.as_slice(), b.as_slice()),
            Ordering::Less
        );
        assert_eq!(
            <Rkyv<SortKey> as Key>::compare(b.as_slice(), a.as_slice()),
            Ordering::Greater
        );
        assert_eq!(
            <Rkyv<SortKey> as Key>::compare(a.as_slice(), a.as_slice()),
            Ordering::Equal
        );
    }

    #[test]
    fn fixed_width_is_none_and_type_name_includes_rkyv() {
        assert_eq!(<Rkyv<User> as Value>::fixed_width(), None);
        let name = format!("{:?}", <Rkyv<User> as Value>::type_name());
        assert!(name.contains("Rkyv"));
    }
}
