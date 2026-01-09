use rkyv::{
    Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize, access_unchecked,
    api::high::{HighDeserializer, HighSerializer},
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
    T::Archived: RkyvDeserialize<T, HighDeserializer<Error>>,
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

        // Use unsafe access since redb doesn't guarantee alignment
        // We trust the data since we control serialization
        unsafe {
            let archived = access_unchecked::<T::Archived>(data);
            deserialize::<T, Error>(archived).unwrap_or_default()
        }
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        rkyv::to_bytes::<Error>(value).unwrap_or_else(|_| AlignedVec::new())
    }

    fn type_name() -> TypeName {
        TypeName::new(&format!("Rkyv<{}>", type_name::<T>()))
    }
}

impl<T> Key for Rkyv<T>
where
    T: std::fmt::Debug + Default + Archive + Ord,
    T::Archived: RkyvDeserialize<T, HighDeserializer<Error>>,
    for<'a> T: RkyvSerialize<HighSerializer<AlignedVec, ArenaHandle<'a>, Error>>,
{
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        Self::from_bytes(data1).cmp(&Self::from_bytes(data2))
    }
}
