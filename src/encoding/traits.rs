// Strict encoding schema library, implementing validation and parsing
// strict encoded data against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2023 Ubideco Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::io::{BufRead, Seek};
use std::{fs, io};

use amplify::confinement::Confined;

use super::DecodeError;
use crate::encoding::{DeserializeError, SerializeError, StrictReader, StrictWriter};

pub trait ToIdent: ToOwned<Owned = String> {}
impl<T> ToIdent for T where T: ToOwned<Owned = String> {}

pub trait TypedWrite: Sized {
    type PrimitiveWriter: WritePrimitive<Self>;
    type TupleWriter: WriteTuple<Self>;
    type StructWriter: WriteStruct<Self>;
    type UnionWriter: WriteUnion<Self>;
    type EnumWriter: WriteEnum<Self>;

    fn write_primitive(self) -> Self::PrimitiveWriter;
    fn write_tuple(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::TupleWriter;
    fn write_struct(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::StructWriter;
    fn write_union(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::UnionWriter;
    fn write_enum(self, ns: impl ToIdent, name: Option<impl ToIdent>) -> Self::EnumWriter;
}

pub trait WritePrimitive<P: Sized>: Sized {
    fn write__(self, value: &impl StrictEncode) -> io::Result<Self>;
    fn complete(self) -> P;
}

pub trait WriteTuple<P: Sized>: Sized {
    fn write_field(self, value: &impl StrictEncode) -> io::Result<Self>;
    fn write_field_ord(self, ord: u8, value: &impl StrictEncode) -> io::Result<Self>;
    fn complete(self) -> P;
}

pub trait WriteStruct<P: Sized>: Sized {
    fn write_field(self, name: impl ToIdent, value: &impl StrictEncode) -> io::Result<Self>;
    fn write_field_ord(
        self,
        name: impl ToIdent,
        ord: u8,
        value: &impl StrictEncode,
    ) -> io::Result<Self>;
    fn complete(self) -> P;
}

pub trait WriteEnum<P: Sized>: Sized {
    fn write_variant(self, name: impl ToIdent, value: u8) -> io::Result<Self>;
    fn complete(self) -> P;
}

pub trait WriteUnion<P: Sized>: Sized {
    type TupleWriter: WriteTuple<Self>;
    type StructWriter: WriteStruct<Self>;

    fn write_unit(self, name: impl ToIdent) -> io::Result<Self>;
    fn write_type(self, name: impl ToIdent, value: &impl StrictEncode) -> io::Result<Self> {
        Ok(self.write_tuple(name).write_field(value)?.complete())
    }
    fn write_tuple(self, name: impl ToIdent) -> Self::TupleWriter;
    fn write_struct(self, name: impl ToIdent) -> Self::StructWriter;
    fn complete(self) -> P;
}

pub trait TypedRead: Sized {}

pub trait StrictEncode {
    fn strict_encode_dumb() -> Self;
    fn strict_encode(&self, writer: &impl TypedWrite) -> io::Result<()>;
}

pub trait StrictDecode: Sized {
    fn strict_decode(reader: &impl TypedRead) -> Result<Self, DecodeError>;
}

pub trait Serialize: StrictEncode {
    fn strict_serialized_len(&self) -> io::Result<usize> {
        let mut counter = StrictWriter::counter();
        self.strict_encode(&mut counter)?;
        Ok(counter.unbox().count)
    }

    fn to_strict_serialized<const MAX: usize>(
        &self,
    ) -> Result<Confined<Vec<u8>, 0, MAX>, SerializeError> {
        let mut ast_data = StrictWriter::in_memory(MAX);
        self.strict_encode(&mut ast_data)?;
        Confined::<Vec<u8>, 0, MAX>::try_from(ast_data.unbox()).map_err(SerializeError::from)
    }

    fn strict_serialize_to_file<const MAX: usize>(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), SerializeError> {
        let mut file = StrictWriter::with(MAX, fs::File::create(path)?);
        self.strict_encode(&mut file).map_err(SerializeError::from)
    }
}

pub trait Deserialize: StrictDecode {
    fn from_strict_serialized<const MAX: usize>(
        ast_data: Confined<Vec<u8>, 0, MAX>,
    ) -> Result<Self, DeserializeError> {
        let cursor = io::Cursor::new(ast_data.into_inner());
        let mut reader = StrictReader::with(MAX, cursor);
        let me = Self::strict_decode(&mut reader)?;
        let mut cursor = reader.unbox();
        if !cursor.fill_buf()?.is_empty() {
            return Err(DeserializeError::DataNotEntirelyConsumed);
        }
        Ok(me)
    }

    fn strict_deserialize_from_file<const MAX: usize>(
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, DeserializeError> {
        let file = fs::File::open(path)?;
        let mut reader = StrictReader::with(MAX, file);
        let me = Self::strict_decode(&mut reader)?;
        let mut file = reader.unbox();
        if file.stream_position()? != file.seek(io::SeekFrom::End(0))? {
            return Err(DeserializeError::DataNotEntirelyConsumed);
        }
        Ok(me)
    }
}
