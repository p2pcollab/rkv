// Copyright 2018-2019 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use super::ErrorImpl;
use crate::backend::traits::{BackendDupIter, BackendIter, BackendRoCursor};
use lmdb::Cursor;

pub struct IterImpl<'i, C> {
    // LMDB semantics dictate that a cursor must be valid for the entire lifetime
    // of an iterator. In other words, cursors must not be dropped while an
    // iterator built from it is alive. Unfortunately, the LMDB crate API does
    // not express this through the type system, so we must enforce it somehow.
    #[allow(dead_code)]
    cursor: C,
    iter: lmdb::Iter<'i>,
}

impl<'i, C> IterImpl<'i, C> {
    pub(crate) fn new(
        mut cursor: C,
        to_iter: impl FnOnce(&mut C) -> lmdb::Iter<'i>,
    ) -> IterImpl<'i, C> {
        let iter = to_iter(&mut cursor);
        IterImpl { cursor, iter }
    }
}

impl<'i, C> BackendIter<'i> for IterImpl<'i, C> {
    type Error = ErrorImpl;

    #[allow(clippy::type_complexity)]
    fn next(&mut self) -> Option<Result<(&'i [u8], &'i [u8]), Self::Error>> {
        self.iter.next().map(|e| e.map_err(ErrorImpl::LmdbError))
    }
}

pub struct ProxyIterImpl<'i> {
    // Here we do not keep the cursor, because we are in a sub-iterator (the iterator of values on a duplicate key)
    // and the lmdb cursor is kept by the higher level iterator.
    // becareful to keep the higher level iterator (the iterator of keys) alive as long as you are iterating on this iterator.
    #[allow(dead_code)]
    iter: lmdb::Iter<'i>,
}

impl<'i> ProxyIterImpl<'i> {
    pub(crate) fn new_from_lmdb_iter(iter: lmdb::Iter<'i>) -> ProxyIterImpl<'i> {
        ProxyIterImpl { iter }
    }
}

impl<'i> BackendIter<'i> for ProxyIterImpl<'i> {
    type Error = ErrorImpl;

    #[allow(clippy::type_complexity)]
    fn next(&mut self) -> Option<Result<(&'i [u8], &'i [u8]), Self::Error>> {
        self.iter.next().map(|e| e.map_err(ErrorImpl::LmdbError))
    }
}

pub struct IterDupImpl<'i, C> {
    // LMDB semantics dictate that a cursor must be valid for the entire lifetime
    // of an iterator. In other words, cursors must not be dropped while an
    // iterator built from it is alive. Unfortunately, the LMDB crate API does
    // not express this through the type system, so we must enforce it somehow.
    #[allow(dead_code)]
    cursor: C,
    iter: lmdb::IterPrevDup<'i>,
}

impl<'i, C> IterDupImpl<'i, C> {
    pub(crate) fn new(
        mut cursor: C,
        to_iter: impl FnOnce(&mut C) -> lmdb::IterPrevDup<'i>,
    ) -> IterDupImpl<'i, C> {
        let iter = to_iter(&mut cursor);
        IterDupImpl { cursor, iter }
    }
}

impl<'i, C> BackendDupIter<'i> for IterDupImpl<'i, C> {
    type Error = ErrorImpl;
    type Iter = ProxyIterImpl<'i>;

    #[allow(clippy::type_complexity)]
    fn next(&mut self) -> Option<Result<Self::Iter, Self::Error>> {
        let next = self.iter.next();
        match next {
            None => None,
            Some(n) => Some(Ok(ProxyIterImpl::new_from_lmdb_iter(n))),
        }
    }
}
