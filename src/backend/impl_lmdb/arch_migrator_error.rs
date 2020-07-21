// Copyright 2018-2019 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use std::{
    io,
    num,
    str,
};

use failure::Fail;

#[derive(Debug, Fail)]
pub enum MigrateError {
    #[fail(display = "database not found: {:?}", _0)]
    DatabaseNotFound(String),

    #[fail(display = "{}", _0)]
    FromString(String),

    #[fail(display = "couldn't determine bit depth")]
    IndeterminateBitDepth,

    #[fail(display = "I/O error: {:?}", _0)]
    IoError(io::Error),

    #[fail(display = "invalid DatabaseFlags bits")]
    InvalidDatabaseBits,

    #[fail(display = "invalid data version")]
    InvalidDataVersion,

    #[fail(display = "invalid magic number")]
    InvalidMagicNum,

    #[fail(display = "invalid NodeFlags bits")]
    InvalidNodeBits,

    #[fail(display = "invalid PageFlags bits")]
    InvalidPageBits,

    #[fail(display = "invalid page number")]
    InvalidPageNum,

    #[fail(display = "lmdb backend error: {}", _0)]
    LmdbError(lmdb::Error),

    #[fail(display = "string conversion error")]
    StringConversionError,

    #[fail(display = "TryFromInt error: {:?}", _0)]
    TryFromIntError(num::TryFromIntError),

    #[fail(display = "unexpected Page variant")]
    UnexpectedPageVariant,

    #[fail(display = "unexpected PageHeader variant")]
    UnexpectedPageHeaderVariant,

    #[fail(display = "unsupported PageHeader variant")]
    UnsupportedPageHeaderVariant,

    #[fail(display = "UTF8 error: {:?}", _0)]
    Utf8Error(str::Utf8Error),
}

impl From<io::Error> for MigrateError {
    fn from(e: io::Error) -> MigrateError {
        MigrateError::IoError(e)
    }
}

impl From<str::Utf8Error> for MigrateError {
    fn from(e: str::Utf8Error) -> MigrateError {
        MigrateError::Utf8Error(e)
    }
}

impl From<num::TryFromIntError> for MigrateError {
    fn from(e: num::TryFromIntError) -> MigrateError {
        MigrateError::TryFromIntError(e)
    }
}

impl From<&str> for MigrateError {
    fn from(e: &str) -> MigrateError {
        MigrateError::FromString(e.to_string())
    }
}

impl From<String> for MigrateError {
    fn from(e: String) -> MigrateError {
        MigrateError::FromString(e)
    }
}

impl From<lmdb::Error> for MigrateError {
    fn from(e: lmdb::Error) -> MigrateError {
        MigrateError::LmdbError(e)
    }
}
