// Copyright 2018-2019 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use lmdb::Cursor;

use super::{IterDupImpl, IterImpl};
use crate::backend::traits::{BackendRoCursor, BackendRwCursor};

#[derive(Debug)]
pub struct RoCursorImpl<'c>(pub(crate) lmdb::RoCursor<'c>);

impl<'c> BackendRoCursor<'c> for RoCursorImpl<'c> {
    type Iter = IterImpl<'c, lmdb::RoCursor<'c>>;

    fn get_key_value<K>(self, key: K, value: &crate::value::Value) -> bool
    where
        K: AsRef<[u8]> + 'c,
    {
        match self.0.get(
            Some(key.as_ref()),
            Some(&value.to_bytes().unwrap()),
            lmdb::MDB_GET_BOTH,
        ) {
            Err(e) => false,
            Ok((Some(k), _)) => true,
            Ok((None, _)) => true,
        }
    }

    fn into_iter(self) -> Self::Iter {
        // We call RoCursor.iter() instead of RoCursor.iter_start() because
        // the latter panics when there are no items in the store, whereas the
        // former returns an iterator that yields no items. And since we create
        // the Cursor and don't change its position, we can be sure that a call
        // to Cursor.iter() will start at the beginning.
        IterImpl::new(self.0, lmdb::RoCursor::iter)
    }

    fn into_iter_from<K>(self, key: K) -> Self::Iter
    where
        K: AsRef<[u8]> + 'c,
    {
        IterImpl::new(self.0, |cursor| cursor.iter_from(key))
    }

    fn into_iter_dup_of<K>(self, key: K) -> Self::Iter
    where
        K: AsRef<[u8]> + 'c,
    {
        IterImpl::new(self.0, |cursor| cursor.iter_dup_of(key))
    }

    fn into_iter_prev(self) -> Self::Iter {
        IterImpl::new(self.0, lmdb::RoCursor::iter_prev)
    }
}

#[derive(Debug)]
pub struct RwCursorImpl<'c>(pub(crate) lmdb::RoCursor<'c>);

impl<'c> BackendRoCursor<'c> for RwCursorImpl<'c> {
    type Iter = IterImpl<'c, lmdb::RoCursor<'c>>;

    fn get_key_value<K>(self, key: K, value: &crate::value::Value) -> bool
    where
        K: AsRef<[u8]> + 'c,
    {
        match self.0.get(
            Some(key.as_ref()),
            Some(&value.to_bytes().unwrap()),
            lmdb::MDB_GET_BOTH,
        ) {
            Err(e) => false,
            Ok((Some(k), _)) => true,
            Ok((None, _)) => false,
        }
    }

    fn into_iter(self) -> Self::Iter {
        IterImpl::new(self.0, lmdb::RoCursor::iter)
    }

    fn into_iter_from<K>(self, key: K) -> Self::Iter
    where
        K: AsRef<[u8]> + 'c,
    {
        IterImpl::new(self.0, |cursor| cursor.iter_from(key))
    }

    fn into_iter_dup_of<K>(self, key: K) -> Self::Iter
    where
        K: AsRef<[u8]> + 'c,
    {
        IterImpl::new(self.0, |cursor| cursor.iter_dup_of(key))
    }

    fn into_iter_prev(self) -> Self::Iter {
        IterImpl::new(self.0, lmdb::RoCursor::iter_prev)
    }
}

impl<'c> BackendRwCursor<'c> for RwCursorImpl<'c> {
    type Iter = IterDupImpl<'c, lmdb::RoCursor<'c>>;

    fn into_iter_prev_dup_from<K>(self, key: K) -> Self::Iter
    where
        K: AsRef<[u8]> + 'c,
    {
        IterDupImpl::new(self.0, |cursor| cursor.iter_prev_dup_from(key))
    }
}
