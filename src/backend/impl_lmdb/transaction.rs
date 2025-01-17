// Copyright 2018-2019 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use lmdb::Transaction;

use super::{DatabaseImpl, ErrorImpl, RoCursorImpl, RwCursorImpl, StatImpl, WriteFlagsImpl};
use crate::backend::traits::{
    BackendRoCursorTransaction, BackendRoTransaction, BackendRwCursorTransaction,
    BackendRwCursorType, BackendRwDupPrevCursorTransaction, BackendRwTransaction,
};

#[derive(Debug)]
pub struct RoTransactionImpl<'t>(pub(crate) lmdb::RoTransaction<'t>);

impl<'t> BackendRoTransaction for RoTransactionImpl<'t> {
    type Database = DatabaseImpl;
    type Error = ErrorImpl;
    type Stat = StatImpl;

    fn get(&self, db: &Self::Database, key: &[u8]) -> Result<&[u8], Self::Error> {
        self.0.get(db.0, &key).map_err(ErrorImpl::LmdbError)
    }

    fn abort(self) {
        self.0.abort()
    }

    fn stat(&self, db: &Self::Database) -> Result<Self::Stat, Self::Error> {
        self.0
            .stat(db.0)
            .map(StatImpl)
            .map_err(ErrorImpl::LmdbError)
    }
}

impl<'t> BackendRoCursorTransaction<'t> for RoTransactionImpl<'t> {
    type RoCursor = RoCursorImpl<'t>;
    type RwCursor = RwCursorImpl<'t>;

    fn open_ro_cursor(&'t self, db: &Self::Database) -> Result<Self::RoCursor, Self::Error> {
        self.0
            .open_ro_cursor(db.0)
            .map(RoCursorImpl)
            .map_err(ErrorImpl::LmdbError)
    }

    fn open_ro_dup_cursor(&'t self, db: &Self::Database) -> Result<Self::RwCursor, Self::Error> {
        self.0
            .open_ro_cursor(db.0)
            .map(RwCursorImpl)
            .map_err(ErrorImpl::LmdbError)
    }
}

impl<'t> BackendRwCursorTransaction<'t> for RwTransactionImpl<'t> {
    type RoCursor = RoCursorImpl<'t>;
    type RwCursor = RwCursorImpl<'t>;

    fn open_ro_cursor(&'t self, db: &Self::Database) -> Result<Self::RoCursor, Self::Error> {
        self.0
            .open_ro_cursor(db.0)
            .map(RoCursorImpl)
            .map_err(ErrorImpl::LmdbError)
    }

    fn open_ro_dup_cursor(&'t self, db: &Self::Database) -> Result<Self::RwCursor, Self::Error> {
        self.0
            .open_ro_cursor(db.0)
            .map(RwCursorImpl)
            .map_err(ErrorImpl::LmdbError)
    }
}

#[derive(Debug)]
pub struct RwTransactionImpl<'t>(pub(crate) lmdb::RwTransaction<'t>);

impl<'t> BackendRwTransaction for RwTransactionImpl<'t> {
    type Database = DatabaseImpl;
    type Error = ErrorImpl;
    type Flags = WriteFlagsImpl;
    type Stat = StatImpl;

    fn get(&self, db: &Self::Database, key: &[u8]) -> Result<&[u8], Self::Error> {
        self.0.get(db.0, &key).map_err(ErrorImpl::LmdbError)
    }

    fn stat(&self, db: &Self::Database) -> Result<Self::Stat, Self::Error> {
        //Err(ErrorImpl::LmdbError(lmdb::Error::Invalid))
        self.0
            .stat(db.0)
            .map(StatImpl)
            .map_err(ErrorImpl::LmdbError)
    }

    fn put(
        &mut self,
        db: &Self::Database,
        key: &[u8],
        value: &[u8],
        flags: Self::Flags,
    ) -> Result<(), Self::Error> {
        self.0
            .put(db.0, &key, &value, flags.0)
            .map_err(ErrorImpl::LmdbError)
    }

    #[cfg(not(feature = "db-dup-sort"))]
    fn del(&mut self, db: &Self::Database, key: &[u8]) -> Result<(), Self::Error> {
        self.0.del(db.0, &key, None).map_err(ErrorImpl::LmdbError)
    }

    #[cfg(feature = "db-dup-sort")]
    fn del(
        &mut self,
        db: &Self::Database,
        key: &[u8],
        value: Option<&[u8]>,
    ) -> Result<(), Self::Error> {
        self.0.del(db.0, &key, value).map_err(ErrorImpl::LmdbError)
    }

    fn clear_db(&mut self, db: &Self::Database) -> Result<(), Self::Error> {
        self.0.clear_db(db.0).map_err(ErrorImpl::LmdbError)
    }

    fn commit(self) -> Result<(), Self::Error> {
        self.0.commit().map_err(ErrorImpl::LmdbError)
    }

    fn abort(self) {
        self.0.abort()
    }
}

pub enum BackendRwCursorFamily {}

impl<'t> BackendRwCursorType<'t> for BackendRwCursorFamily {
    type Type = RwCursorImpl<'t>;
}

impl<'t> BackendRwDupPrevCursorTransaction for RwTransactionImpl<'t> {
    type RwCursor = BackendRwCursorFamily;

    // fn open_rw_dup_prev_cursor<'a>(
    //     &mut self,
    //     db: &Self::Database,
    // ) -> Result<RwCursorImpl<'a>, Self::Error> {
    //     self.0
    //         .open_rw_cursor(db.0)
    //         .map(RwCursorImpl)
    //         .map_err(ErrorImpl::LmdbError)
    // }
}
