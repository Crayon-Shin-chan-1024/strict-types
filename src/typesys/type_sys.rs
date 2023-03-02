// Strict encoding schema library, implementing validation and parsing
// strict encoded data against a schema.
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2023 by
//     Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2023 UBIDECO Institute
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

//! Embedded lib is a set of compiled type libraries having no external
//! dependencies

use std::fmt::{self, Display, Formatter};

use amplify::confinement;
use amplify::confinement::MediumOrdMap;
use amplify::num::u24;
use encoding::{LibName, StrictDeserialize, StrictSerialize, TypeName};
use strict_encoding::STRICT_TYPES_LIB;

use crate::{SemId, Ty};

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[display("{lib}.{name}")]
pub struct TypeFqn {
    pub lib: LibName,
    pub name: TypeName,
}

impl TypeFqn {
    pub fn with(lib: LibName, name: TypeName) -> TypeFqn { TypeFqn { lib, name } }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
pub struct TypeFqid {
    pub id: SemId,
    pub fqn: Option<TypeFqn>,
}

impl TypeFqid {
    pub fn unnamed(id: SemId) -> TypeFqid { TypeFqid { id, fqn: None } }

    pub fn named(id: SemId, lib: LibName, name: TypeName) -> TypeFqid {
        TypeFqid {
            id,
            fqn: Some(TypeFqn::with(lib, name)),
        }
    }
}

impl Display for TypeFqid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.fqn {
            Some(fqn) => Display::fmt(fqn, f),
            None => Display::fmt(&self.id, f),
        }
    }
}

/// Type system represents a set of strict types assembled from multiple
/// libraries. It is designed to provide all necessary type information to
/// analyze a type with all types it depends onto.
///
/// # Type guarantees
///
/// - Total number of types do not exceed 2^24-1;
/// - Strict-serialized size is less than 2^24 bytes;
/// - Type system is complete (i.e. no type references a type which is not a part of the system).
#[derive(Wrapper, Clone, Eq, PartialEq, Debug, Default, From)]
#[wrapper(Deref)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
pub struct TypeSystem(MediumOrdMap<TypeFqid, Ty<SemId>>);

impl StrictSerialize for TypeSystem {}
impl StrictDeserialize for TypeSystem {}

impl TypeSystem {
    pub fn new() -> Self { Self::default() }

    pub fn count_types(&self) -> u24 { self.0.len_u24() }

    pub(super) fn insert_unchecked(
        &mut self,
        fqid: TypeFqid,
        ty: Ty<SemId>,
    ) -> Result<bool, confinement::Error> {
        self.0.insert(fqid, ty).map(|res| res.is_some())
    }
}

impl Display for TypeSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "typesys -- {:+}", self.id())?;
        writeln!(f)?;
        for (id, ty) in &self.0 {
            writeln!(f, "data {id:0} :: {ty:0}")?;
        }
        Ok(())
    }
}

#[cfg(feature = "base64")]
impl fmt::UpperHex for TypeSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use baid58::ToBaid58;
        use base64::Engine;

        let id = self.id();

        writeln!(f, "----- BEGIN STRICT TYPE SYSTEM -----")?;
        writeln!(f, "Id: {}", id)?;
        writeln!(f, "Checksum: {}", id.to_baid58().mnemonic())?;
        writeln!(f)?;

        let data = self.to_strict_serialized::<0xFFFFFF>().expect("in-memory");
        let engine = base64::engine::general_purpose::STANDARD;
        let data = engine.encode(data);
        let mut data = data.as_str();
        while data.len() >= 76 {
            let (line, rest) = data.split_at(76);
            writeln!(f, "{}", line)?;
            data = rest;
        }
        writeln!(f, "{}", data)?;

        writeln!(f, "\n----- END STRICT TYPE SYSTEM -----")?;
        Ok(())
    }
}