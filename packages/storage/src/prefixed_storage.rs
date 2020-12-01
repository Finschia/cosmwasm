use cosmwasm_std::Storage;
#[cfg(feature = "iterator")]
use cosmwasm_std::{Order, KV};

use crate::length_prefixed::{to_length_prefixed, to_length_prefixed_nested};
#[cfg(feature = "iterator")]
use crate::namespace_helpers::range_with_prefix;
use crate::namespace_helpers::{get_with_prefix, remove_with_prefix, set_with_prefix};

/// An alias of PrefixedStorage::new for less verbose usage
pub fn prefixed<'a>(storage: &'a mut dyn Storage, namespace: &[u8]) -> PrefixedStorage<'a> {
    PrefixedStorage::new(storage, namespace)
}

/// An alias of ReadonlyPrefixedStorage::new for less verbose usage
pub fn prefixed_read<'a>(
    storage: &'a dyn Storage,
    namespace: &[u8],
) -> ReadonlyPrefixedStorage<'a> {
    ReadonlyPrefixedStorage::new(storage, namespace)
}

pub struct PrefixedStorage<'a> {
    storage: &'a mut dyn Storage,
    prefix: Vec<u8>,
}

impl<'a> PrefixedStorage<'a> {
    pub fn new(storage: &'a mut dyn Storage, namespace: &[u8]) -> Self {
        PrefixedStorage {
            storage,
            prefix: to_length_prefixed(namespace),
        }
    }

    // Nested namespaces as documented in
    // https://github.com/webmaster128/key-namespacing#nesting
    pub fn multilevel(storage: &'a mut dyn Storage, namespaces: &[&[u8]]) -> Self {
        PrefixedStorage {
            storage,
            prefix: to_length_prefixed_nested(namespaces),
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        get_with_prefix(self.storage, &self.prefix, key)
    }

    pub fn set(&mut self, key: &[u8], value: &[u8]) {
        set_with_prefix(self.storage, &self.prefix, key, value);
    }

    pub fn remove(&mut self, key: &[u8]) {
        remove_with_prefix(self.storage, &self.prefix, key);
    }

    #[cfg(feature = "iterator")]
    /// range allows iteration over a set of keys, either forwards or backwards
    /// uses standard rust range notation, and eg db.range(b"foo"..b"bar") also works reverse
    pub fn range<'b>(
        &'b self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = KV> + 'b> {
        range_with_prefix(self.storage, &self.prefix, start, end, order)
    }
}

pub struct ReadonlyPrefixedStorage<'a> {
    storage: &'a dyn Storage,
    prefix: Vec<u8>,
}

impl<'a> ReadonlyPrefixedStorage<'a> {
    pub fn new(storage: &'a dyn Storage, namespace: &[u8]) -> Self {
        ReadonlyPrefixedStorage {
            storage,
            prefix: to_length_prefixed(namespace),
        }
    }

    // Nested namespaces as documented in
    // https://github.com/webmaster128/key-namespacing#nesting
    pub fn multilevel(storage: &'a dyn Storage, namespaces: &[&[u8]]) -> Self {
        ReadonlyPrefixedStorage {
            storage,
            prefix: to_length_prefixed_nested(namespaces),
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        get_with_prefix(self.storage, &self.prefix, key)
    }

    #[cfg(feature = "iterator")]
    /// range allows iteration over a set of keys, either forwards or backwards
    pub fn range<'b>(
        &'b self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = KV> + 'b> {
        range_with_prefix(self.storage, &self.prefix, start, end, order)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::MockStorage;

    #[test]
    fn prefixed_storage_set_and_get() {
        let mut storage = MockStorage::new();

        // set
        let mut foo = PrefixedStorage::new(&mut storage, b"foo");
        foo.set(b"bar", b"gotcha");
        assert_eq!(storage.get(b"\x00\x03foobar").unwrap(), b"gotcha".to_vec());

        // get
        let foo = PrefixedStorage::new(&mut storage, b"foo");
        assert_eq!(foo.get(b"bar"), Some(b"gotcha".to_vec()));
        assert_eq!(foo.get(b"elsewhere"), None);
    }

    #[test]
    fn prefixed_storage_multilevel_set_and_get() {
        let mut storage = MockStorage::new();

        // set
        let mut bar = PrefixedStorage::multilevel(&mut storage, &[b"foo", b"bar"]);
        bar.set(b"baz", b"winner");
        assert_eq!(
            storage.get(b"\x00\x03foo\x00\x03barbaz").unwrap(),
            b"winner".to_vec()
        );

        // get
        let bar = PrefixedStorage::multilevel(&mut storage, &[b"foo", b"bar"]);
        assert_eq!(bar.get(b"baz"), Some(b"winner".to_vec()));
        assert_eq!(bar.get(b"elsewhere"), None);
    }

    #[test]
    fn readonly_prefixed_storage_get() {
        let mut storage = MockStorage::new();
        storage.set(b"\x00\x03foobar", b"gotcha");

        // try readonly correctly
        let foo = ReadonlyPrefixedStorage::new(&storage, b"foo");
        assert_eq!(foo.get(b"bar"), Some(b"gotcha".to_vec()));
        assert_eq!(foo.get(b"elsewhere"), None);

        // no collisions with other prefixes
        let fo = ReadonlyPrefixedStorage::new(&storage, b"fo");
        assert_eq!(fo.get(b"obar"), None);
    }

    #[test]
    fn readonly_prefixed_storage_multilevel_get() {
        let mut storage = MockStorage::new();
        storage.set(b"\x00\x03foo\x00\x03barbaz", b"winner");

        let bar = ReadonlyPrefixedStorage::multilevel(&storage, &[b"foo", b"bar"]);
        assert_eq!(bar.get(b"baz"), Some(b"winner".to_vec()));
        assert_eq!(bar.get(b"elsewhere"), None);
    }
}
