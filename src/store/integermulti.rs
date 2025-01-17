// Copyright 2018 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use std::marker::PhantomData;

use crate::{
    backend::{
        BackendDatabase, BackendDupIter, BackendIter, BackendRoCursor, BackendRwCursor,
        BackendRwTransaction,
    },
    error::StoreError,
    readwrite::{Readable, Writer},
    store::{
        keys::{Key, PrimitiveInt},
        multi::{DIter, Iter, MultiStore},
    },
    value::Value,
};

type EmptyResult = Result<(), StoreError>;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct MultiIntegerStore<D, K> {
    inner: MultiStore<D>,
    phantom: PhantomData<K>,
}

impl<D, K> MultiIntegerStore<D, K>
where
    D: BackendDatabase,
    K: PrimitiveInt,
{
    pub(crate) fn new(db: D) -> MultiIntegerStore<D, K> {
        MultiIntegerStore {
            inner: MultiStore::new(db),
            phantom: PhantomData,
        }
    }

    pub fn iter_prev_dup_from<'r, I, C, R>(
        &self,
        reader: &'r R,
        k: K,
    ) -> Result<DIter<'r, I>, StoreError>
    where
        R: Readable<'r, Database = D, RwCursor = C>,
        I: BackendDupIter<'r>,
        C: BackendRwCursor<'r, Iter = I>,
        K: 'r,
    {
        self.inner.iter_prev_dup_from(reader, Key::new(&k)?)
    }

    pub fn get<'r, R, I, C>(&self, reader: &'r R, k: K) -> Result<Iter<'r, I>, StoreError>
    where
        R: Readable<'r, Database = D, RoCursor = C>,
        I: BackendIter<'r>,
        C: BackendRoCursor<'r, Iter = I>,
        K: 'r,
    {
        self.inner.get(reader, Key::new(&k)?)
    }

    pub fn get_first<'r, R>(&self, reader: &'r R, k: K) -> Result<Option<Value<'r>>, StoreError>
    where
        R: Readable<'r, Database = D>,
    {
        self.inner.get_first(reader, Key::new(&k)?)
    }

    pub fn iter_start<'r, R, I, C>(&self, reader: &'r R) -> Result<Iter<'r, I>, StoreError>
    where
        R: Readable<'r, Database = D, RoCursor = C>,
        I: BackendIter<'r>,
        C: BackendRoCursor<'r, Iter = I>,
    {
        self.inner.iter_start(reader)
    }

    pub fn put<T>(&self, writer: &mut Writer<T>, k: K, v: &Value) -> EmptyResult
    where
        T: BackendRwTransaction<Database = D>,
    {
        self.inner.put(writer, Key::new(&k)?, v)
    }

    pub fn put_with_flags<T>(
        &self,
        writer: &mut Writer<T>,
        k: K,
        v: &Value,
        flags: T::Flags,
    ) -> EmptyResult
    where
        T: BackendRwTransaction<Database = D>,
    {
        self.inner.put_with_flags(writer, Key::new(&k)?, v, flags)
    }

    pub fn delete_all<T>(&self, writer: &mut Writer<T>, k: K) -> EmptyResult
    where
        T: BackendRwTransaction<Database = D>,
    {
        self.inner.delete_all(writer, Key::new(&k)?)
    }

    pub fn delete<T>(&self, writer: &mut Writer<T>, k: K, v: &Value) -> EmptyResult
    where
        T: BackendRwTransaction<Database = D>,
    {
        self.inner.delete(writer, Key::new(&k)?, v)
    }

    pub fn clear<T>(&self, writer: &mut Writer<T>) -> EmptyResult
    where
        T: BackendRwTransaction<Database = D>,
    {
        self.inner.clear(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::BackendDatabaseFlags;
    use crate::*;
    use std::fs;

    use tempfile::Builder;

    #[test]
    fn test_integer_keys() {
        let root = Builder::new()
            .prefix("test_integer_keys")
            .tempdir()
            .expect("tempdir");
        fs::create_dir_all(root.path()).expect("dir created");

        let k = Rkv::new::<backend::SafeMode>(root.path()).expect("new succeeded");
        let s = k
            .open_multi_integer("s", StoreOptions::create())
            .expect("open");

        macro_rules! test_integer_keys {
            ($type:ty, $key:expr) => {{
                let mut writer = k.write().expect("writer");

                s.put(&mut writer, $key, &Value::Str("hello!"))
                    .expect("write");
                assert_eq!(
                    s.get_first(&writer, $key).expect("read"),
                    Some(Value::Str("hello!"))
                );
                writer.commit().expect("committed");

                let reader = k.read().expect("reader");
                assert_eq!(
                    s.get_first(&reader, $key).expect("read"),
                    Some(Value::Str("hello!"))
                );
            }};
        }

        test_integer_keys!(u32, std::u32::MIN);
        test_integer_keys!(u32, std::u32::MAX);
    }

    #[test]
    fn test_clear() {
        let root = Builder::new()
            .prefix("test_multi_integer_clear")
            .tempdir()
            .expect("tempdir");
        fs::create_dir_all(root.path()).expect("dir created");

        let k = Rkv::new::<backend::SafeMode>(root.path()).expect("new succeeded");
        let s = k
            .open_multi_integer("s", StoreOptions::create())
            .expect("open");

        {
            let mut writer = k.write().expect("writer");
            s.put(&mut writer, 1, &Value::Str("hello!")).expect("write");
            s.put(&mut writer, 1, &Value::Str("hello1!"))
                .expect("write");
            s.put(&mut writer, 2, &Value::Str("hello!")).expect("write");
            assert_eq!(
                s.get_first(&writer, 1).expect("read"),
                Some(Value::Str("hello!"))
            );
            assert_eq!(
                s.get_first(&writer, 2).expect("read"),
                Some(Value::Str("hello!"))
            );
            assert_eq!(s.get_first(&writer, 3).expect("read"), None);
            writer.commit().expect("committed");
        }

        {
            let mut writer = k.write().expect("writer");
            s.clear(&mut writer).expect("cleared");
            writer.commit().expect("committed");

            let reader = k.read().expect("reader");
            assert_eq!(s.get_first(&reader, 1).expect("read"), None);
            assert_eq!(s.get_first(&reader, 2).expect("read"), None);
            assert_eq!(s.get_first(&reader, 3).expect("read"), None);
        }
    }

    #[test]
    fn test_dup() {
        let root = Builder::new()
            .prefix("test_multi_integer_dup")
            .tempdir()
            .expect("tempdir");
        fs::create_dir_all(root.path()).expect("dir created");

        let k = Rkv::new::<backend::SafeMode>(root.path()).expect("new succeeded");
        let s = k
            .open_multi_integer("s", StoreOptions::create())
            .expect("open");

        {
            let mut writer = k.write().expect("writer");
            s.put(&mut writer, 1, &Value::Str("hello!")).expect("write");
            s.put(&mut writer, 1, &Value::Str("hello!")).expect("write");
            s.put(&mut writer, 1, &Value::Str("hello1!"))
                .expect("write");
            assert_eq!(
                s.get_first(&writer, 1).expect("read"),
                Some(Value::Str("hello!"))
            );
            assert_eq!(s.get_first(&writer, 2).expect("read"), None);
            assert_eq!(s.get_first(&writer, 3).expect("read"), None);
            writer.commit().expect("committed");
        }

        {
            let mut writer = k.write().expect("writer");
            s.clear(&mut writer).expect("cleared");
            writer.commit().expect("committed");

            let reader = k.read().expect("reader");
            assert_eq!(s.get_first(&reader, 1).expect("read"), None);
            assert_eq!(s.get_first(&reader, 2).expect("read"), None);
            assert_eq!(s.get_first(&reader, 3).expect("read"), None);
        }
    }

    #[test]
    fn test_dup_2() {
        let root = Builder::new()
            .prefix("test_multi_integer_dup")
            .tempdir()
            .expect("tempdir");
        fs::create_dir_all(root.path()).expect("dir created");

        let k = Rkv::new::<backend::SafeMode>(root.path()).expect("new succeeded");
        let s = k
            .open_multi_integer("s", StoreOptions::create())
            .expect("open");

        {
            let mut writer = k.write().expect("writer");
            s.put(&mut writer, 1, &Value::Str("hello!")).expect("write");
            s.put(&mut writer, 1, &Value::Str("hello!")).expect("write");
            s.put(&mut writer, 1, &Value::Str("hello1!"))
                .expect("write");

            let mut iter = s.get(&writer, 1).expect("read");
            assert_eq!(
                iter.next().expect("first").expect("ok").1,
                Value::Str("hello!")
            );
            assert_eq!(
                iter.next().expect("second").expect("ok").1,
                Value::Str("hello1!")
            );
            assert!(iter.next().is_none());
        }
    }

    #[cfg(feature = "lmdb")]
    #[test]
    pub fn test_dup_iter_integer() {
        let path_str = "test-env";
        let root = Builder::new().prefix(path_str).tempdir().unwrap();

        // we set an encryption key with all zeros... for test purpose only ;)
        let key: [u8; 32] = [0; 32];
        {
            fs::create_dir_all(root.path()).unwrap();

            println!("{}", root.path().to_str().unwrap());

            let mut manager = Manager::<backend::LmdbEnvironment>::singleton()
                .write()
                .unwrap();
            let shared_rkv = manager
                .get_or_create(root.path(), |path| {
                    // Rkv::new::<Lmdb>(path) // use this instead to disable encryption
                    Rkv::with_encryption_key_and_mapsize::<backend::Lmdb>(
                        path,
                        key,
                        2 * 1024 * 1024 * 1024,
                    )
                })
                .unwrap();
            let env = shared_rkv.read().unwrap();

            println!("LMDB Version: {}", env.version());

            let mut opts = StoreOptions::<backend::LmdbDatabaseFlags>::create();
            opts.flags.set(DatabaseFlags::DUP_FIXED, true);
            let expiry_store = env.open_multi_integer("expiry", opts).unwrap();

            let mut writer = env.write().expect("writer");
            expiry_store
                .put(&mut writer, 1, &Value::Blob(&[104u8, 101u8, 108u8, 108u8]))
                .expect("write");
            expiry_store
                .put(&mut writer, 1, &Value::Blob(&[104u8, 101u8, 108u8, 108u8]))
                .expect("write");
            expiry_store
                .put(&mut writer, 1, &Value::Blob(&[104u8, 101u8, 108u8, 107u8]))
                .expect("write");
            expiry_store
                .put(&mut writer, 2, &Value::Blob(&[2u8, 2u8, 2u8, 2u8]))
                .expect("write");
            expiry_store
                .put(&mut writer, 2, &Value::Blob(&[2u8, 2u8, 2u8, 3u8]))
                .expect("write");
            expiry_store
                .put(&mut writer, 4, &Value::Blob(&[4u8, 2u8, 2u8, 4u8]))
                .expect("write");
            expiry_store
                .put(&mut writer, 4, &Value::Blob(&[4u8, 2u8, 2u8, 3u8]))
                .expect("write");

            let mut iter = expiry_store.get(&writer, 1).expect("firstkey");

            assert_eq!(
                iter.next().expect("firstvalue").expect("ok").1,
                Value::Blob(&[104u8, 101u8, 108u8, 107u8])
            );
            assert_eq!(
                iter.next().expect("secondvalue").expect("ok").1,
                Value::Blob(&[104u8, 101u8, 108u8, 108u8])
            );
            assert!(iter.next().is_none());

            let mut iter = expiry_store.iter_prev_dup_from(&writer, 4).unwrap();

            let mut sub_iter = iter.next().unwrap().unwrap();
            assert_eq!(sub_iter.next().expect("first").expect("ok").1, [2, 2, 2, 3]);
            assert_eq!(
                sub_iter.next().expect("second").expect("ok").1,
                [2, 2, 2, 2]
            );
            assert!(sub_iter.next().is_none());

            let mut sub_iter2 = iter.next().unwrap().unwrap();
            assert_eq!(
                sub_iter2.next().expect("first").expect("ok").1,
                [104, 101, 108, 108]
            );
            assert_eq!(
                sub_iter2.next().expect("second").expect("ok").1,
                [104, 101, 108, 107]
            );
            assert!(sub_iter2.next().is_none());

            //writer.commit().unwrap();
        }
    }

    #[test]
    fn test_del() {
        let root = Builder::new()
            .prefix("test_multi_integer_dup")
            .tempdir()
            .expect("tempdir");
        fs::create_dir_all(root.path()).expect("dir created");

        let k = Rkv::new::<backend::SafeMode>(root.path()).expect("new succeeded");
        let s = k
            .open_multi_integer("s", StoreOptions::create())
            .expect("open");

        {
            let mut writer = k.write().expect("writer");
            s.put(&mut writer, 1, &Value::Str("hello!")).expect("write");
            s.put(&mut writer, 1, &Value::Str("hello!")).expect("write");
            s.put(&mut writer, 1, &Value::Str("hello1!"))
                .expect("write");
            {
                let mut iter = s.get(&writer, 1).expect("read");
                assert_eq!(
                    iter.next().expect("first").expect("ok").1,
                    Value::Str("hello!")
                );
                assert_eq!(
                    iter.next().expect("second").expect("ok").1,
                    Value::Str("hello1!")
                );
                assert!(iter.next().is_none());
            }
            writer.commit().expect("committed");
        }

        {
            let mut writer = k.write().expect("writer");
            s.delete(&mut writer, 1, &Value::Str("hello!"))
                .expect("deleted");
            writer.commit().expect("committed");

            let reader = k.read().expect("reader");
            let mut iter = s.get(&reader, 1).expect("read");
            assert_eq!(
                iter.next().expect("first").expect("ok").1,
                Value::Str("hello1!")
            );
            assert!(iter.next().is_none());
        }

        {
            let mut writer = k.write().expect("writer");
            s.delete(&mut writer, 1, &Value::Str("hello!"))
                .expect_err("deleted");
            writer.commit().expect("committed");

            let reader = k.read().expect("reader");
            let mut iter = s.get(&reader, 1).expect("read");
            assert_eq!(
                iter.next().expect("first").expect("ok").1,
                Value::Str("hello1!")
            );
            assert!(iter.next().is_none());
        }

        {
            let mut writer = k.write().expect("writer");
            s.delete(&mut writer, 1, &Value::Str("hello1!"))
                .expect("deleted");
            writer.commit().expect("committed");

            let reader = k.read().expect("reader");
            let mut iter = s.get(&reader, 1).expect("read");
            assert!(iter.next().is_none());
        }

        {
            let mut writer = k.write().expect("writer");
            s.delete(&mut writer, 1, &Value::Str("hello1!"))
                .expect_err("deleted");
            writer.commit().expect("committed");

            let reader = k.read().expect("reader");
            let mut iter = s.get(&reader, 1).expect("read");
            assert!(iter.next().is_none());
        }
    }

    #[test]
    fn test_persist() {
        let root = Builder::new()
            .prefix("test_multi_integer_persist")
            .tempdir()
            .expect("tempdir");
        fs::create_dir_all(root.path()).expect("dir created");

        {
            let k = Rkv::new::<backend::SafeMode>(root.path()).expect("new succeeded");
            let s = k
                .open_multi_integer("s", StoreOptions::create())
                .expect("open");

            let mut writer = k.write().expect("writer");
            s.put(&mut writer, 1, &Value::Str("hello!")).expect("write");
            s.put(&mut writer, 1, &Value::Str("hello1!"))
                .expect("write");
            s.put(&mut writer, 2, &Value::Str("hello!")).expect("write");
            {
                let mut iter = s.get(&writer, 1).expect("read");
                assert_eq!(
                    iter.next().expect("first").expect("ok").1,
                    Value::Str("hello!")
                );
                assert_eq!(
                    iter.next().expect("second").expect("ok").1,
                    Value::Str("hello1!")
                );
                assert!(iter.next().is_none());
            }
            writer.commit().expect("committed");
        }

        {
            let k = Rkv::new::<backend::SafeMode>(root.path()).expect("new succeeded");
            let s = k
                .open_multi_integer("s", StoreOptions::create())
                .expect("open");

            let reader = k.read().expect("reader");
            let mut iter = s.get(&reader, 1).expect("read");
            assert_eq!(
                iter.next().expect("first").expect("ok").1,
                Value::Str("hello!")
            );
            assert_eq!(
                iter.next().expect("second").expect("ok").1,
                Value::Str("hello1!")
            );
            assert!(iter.next().is_none());
        }
    }
}

#[cfg(test)]
mod tests_safe {
    use super::*;
    use crate::*;

    use std::fs;

    use tempfile::Builder;

    #[test]
    fn test_integer_keys() {
        let root = Builder::new()
            .prefix("test_integer_keys")
            .tempdir()
            .expect("tempdir");
        fs::create_dir_all(root.path()).expect("dir created");

        let k = Rkv::new::<backend::SafeMode>(root.path()).expect("new succeeded");
        let s = k
            .open_multi_integer("s", StoreOptions::create())
            .expect("open");

        macro_rules! test_integer_keys {
            ($type:ty, $key:expr) => {{
                let mut writer = k.write().expect("writer");

                s.put(&mut writer, $key, &Value::Str("hello!"))
                    .expect("write");
                assert_eq!(
                    s.get_first(&writer, $key).expect("read"),
                    Some(Value::Str("hello!"))
                );
                writer.commit().expect("committed");

                let reader = k.read().expect("reader");
                assert_eq!(
                    s.get_first(&reader, $key).expect("read"),
                    Some(Value::Str("hello!"))
                );
            }};
        }

        test_integer_keys!(u32, std::u32::MIN);
        test_integer_keys!(u32, std::u32::MAX);
    }

    #[test]
    fn test_clear() {
        let root = Builder::new()
            .prefix("test_multi_integer_clear")
            .tempdir()
            .expect("tempdir");
        fs::create_dir_all(root.path()).expect("dir created");

        let k = Rkv::new::<backend::SafeMode>(root.path()).expect("new succeeded");
        let s = k
            .open_multi_integer("s", StoreOptions::create())
            .expect("open");

        {
            let mut writer = k.write().expect("writer");
            s.put(&mut writer, 1, &Value::Str("hello!")).expect("write");
            s.put(&mut writer, 1, &Value::Str("hello1!"))
                .expect("write");
            s.put(&mut writer, 2, &Value::Str("hello!")).expect("write");
            assert_eq!(
                s.get_first(&writer, 1).expect("read"),
                Some(Value::Str("hello!"))
            );
            assert_eq!(
                s.get_first(&writer, 2).expect("read"),
                Some(Value::Str("hello!"))
            );
            assert_eq!(s.get_first(&writer, 3).expect("read"), None);
            writer.commit().expect("committed");
        }

        {
            let mut writer = k.write().expect("writer");
            s.clear(&mut writer).expect("cleared");
            writer.commit().expect("committed");

            let reader = k.read().expect("reader");
            assert_eq!(s.get_first(&reader, 1).expect("read"), None);
            assert_eq!(s.get_first(&reader, 2).expect("read"), None);
            assert_eq!(s.get_first(&reader, 3).expect("read"), None);
        }
    }

    #[test]
    fn test_dup() {
        let root = Builder::new()
            .prefix("test_multi_integer_dup")
            .tempdir()
            .expect("tempdir");
        fs::create_dir_all(root.path()).expect("dir created");

        let k = Rkv::new::<backend::SafeMode>(root.path()).expect("new succeeded");
        let s = k
            .open_multi_integer("s", StoreOptions::create())
            .expect("open");

        {
            let mut writer = k.write().expect("writer");
            s.put(&mut writer, 1, &Value::Str("hello!")).expect("write");
            s.put(&mut writer, 1, &Value::Str("hello!")).expect("write");
            s.put(&mut writer, 1, &Value::Str("hello1!"))
                .expect("write");
            assert_eq!(
                s.get_first(&writer, 1).expect("read"),
                Some(Value::Str("hello!"))
            );
            assert_eq!(s.get_first(&writer, 2).expect("read"), None);
            assert_eq!(s.get_first(&writer, 3).expect("read"), None);
            writer.commit().expect("committed");
        }

        {
            let mut writer = k.write().expect("writer");
            s.clear(&mut writer).expect("cleared");
            writer.commit().expect("committed");

            let reader = k.read().expect("reader");
            assert_eq!(s.get_first(&reader, 1).expect("read"), None);
            assert_eq!(s.get_first(&reader, 2).expect("read"), None);
            assert_eq!(s.get_first(&reader, 3).expect("read"), None);
        }
    }

    #[test]
    fn test_dup_2() {
        let root = Builder::new()
            .prefix("test_multi_integer_dup")
            .tempdir()
            .expect("tempdir");
        fs::create_dir_all(root.path()).expect("dir created");

        let k = Rkv::new::<backend::SafeMode>(root.path()).expect("new succeeded");
        let s = k
            .open_multi_integer("s", StoreOptions::create())
            .expect("open");

        {
            let mut writer = k.write().expect("writer");
            s.put(&mut writer, 1, &Value::Str("hello!")).expect("write");
            s.put(&mut writer, 1, &Value::Str("hello!")).expect("write");
            s.put(&mut writer, 1, &Value::Str("hello1!"))
                .expect("write");

            let mut iter = s.get(&writer, 1).expect("read");
            assert_eq!(
                iter.next().expect("first").expect("ok").1,
                Value::Str("hello!")
            );
            assert_eq!(
                iter.next().expect("second").expect("ok").1,
                Value::Str("hello1!")
            );
            assert!(iter.next().is_none());
        }
    }

    #[test]
    fn test_del() {
        let root = Builder::new()
            .prefix("test_multi_integer_dup")
            .tempdir()
            .expect("tempdir");
        fs::create_dir_all(root.path()).expect("dir created");

        let k = Rkv::new::<backend::SafeMode>(root.path()).expect("new succeeded");
        let s = k
            .open_multi_integer("s", StoreOptions::create())
            .expect("open");

        {
            let mut writer = k.write().expect("writer");
            s.put(&mut writer, 1, &Value::Str("hello!")).expect("write");
            s.put(&mut writer, 1, &Value::Str("hello!")).expect("write");
            s.put(&mut writer, 1, &Value::Str("hello1!"))
                .expect("write");
            {
                let mut iter = s.get(&writer, 1).expect("read");
                assert_eq!(
                    iter.next().expect("first").expect("ok").1,
                    Value::Str("hello!")
                );
                assert_eq!(
                    iter.next().expect("second").expect("ok").1,
                    Value::Str("hello1!")
                );
                assert!(iter.next().is_none());
            }
            writer.commit().expect("committed");
        }

        {
            let mut writer = k.write().expect("writer");
            s.delete(&mut writer, 1, &Value::Str("hello!"))
                .expect("deleted");
            writer.commit().expect("committed");

            let reader = k.read().expect("reader");
            let mut iter = s.get(&reader, 1).expect("read");
            assert_eq!(
                iter.next().expect("first").expect("ok").1,
                Value::Str("hello1!")
            );
            assert!(iter.next().is_none());
        }

        {
            let mut writer = k.write().expect("writer");
            s.delete(&mut writer, 1, &Value::Str("hello!"))
                .expect_err("deleted");
            writer.commit().expect("committed");

            let reader = k.read().expect("reader");
            let mut iter = s.get(&reader, 1).expect("read");
            assert_eq!(
                iter.next().expect("first").expect("ok").1,
                Value::Str("hello1!")
            );
            assert!(iter.next().is_none());
        }

        {
            let mut writer = k.write().expect("writer");
            s.delete(&mut writer, 1, &Value::Str("hello1!"))
                .expect("deleted");
            writer.commit().expect("committed");

            let reader = k.read().expect("reader");
            let mut iter = s.get(&reader, 1).expect("read");
            assert!(iter.next().is_none());
        }

        {
            let mut writer = k.write().expect("writer");
            s.delete(&mut writer, 1, &Value::Str("hello1!"))
                .expect_err("deleted");
            writer.commit().expect("committed");

            let reader = k.read().expect("reader");
            let mut iter = s.get(&reader, 1).expect("read");
            assert!(iter.next().is_none());
        }
    }

    #[test]
    fn test_persist() {
        let root = Builder::new()
            .prefix("test_multi_integer_persist")
            .tempdir()
            .expect("tempdir");
        fs::create_dir_all(root.path()).expect("dir created");

        {
            let k = Rkv::new::<backend::SafeMode>(root.path()).expect("new succeeded");
            let s = k
                .open_multi_integer("s", StoreOptions::create())
                .expect("open");

            let mut writer = k.write().expect("writer");
            s.put(&mut writer, 1, &Value::Str("hello!")).expect("write");
            s.put(&mut writer, 1, &Value::Str("hello1!"))
                .expect("write");
            s.put(&mut writer, 2, &Value::Str("hello!")).expect("write");
            {
                let mut iter = s.get(&writer, 1).expect("read");
                assert_eq!(
                    iter.next().expect("first").expect("ok").1,
                    Value::Str("hello!")
                );
                assert_eq!(
                    iter.next().expect("second").expect("ok").1,
                    Value::Str("hello1!")
                );
                assert!(iter.next().is_none());
            }
            writer.commit().expect("committed");
        }

        {
            let k = Rkv::new::<backend::SafeMode>(root.path()).expect("new succeeded");
            let s = k
                .open_multi_integer("s", StoreOptions::create())
                .expect("open");

            let reader = k.read().expect("reader");
            let mut iter = s.get(&reader, 1).expect("read");
            assert_eq!(
                iter.next().expect("first").expect("ok").1,
                Value::Str("hello!")
            );
            assert_eq!(
                iter.next().expect("second").expect("ok").1,
                Value::Str("hello1!")
            );
            assert!(iter.next().is_none());
        }
    }
}
