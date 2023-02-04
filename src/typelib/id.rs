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

use amplify::{Bytes32, RawArray};
use baid58::ToBaid58;
use encoding::{StrictDeserialize, StrictSerialize};
use strict_encoding::{StrictDumb, STRICT_TYPES_LIB};

use crate::typelib::TypeLib;

pub const LIB_ID_TAG: [u8; 32] = *b"urn:ubideco:strict-types:lib:v01";

/// Semantic type id, which commits to the type memory layout, name and field/variant names.
#[derive(Wrapper, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, From)]
#[wrapper(Deref, BorrowSlice, FromStr, Hex, Index, RangeOps)]
#[display(Self::to_baid58)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = STRICT_TYPES_LIB)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct TypeLibId(
    #[from]
    #[from([u8; 32])]
    Bytes32,
);

impl StrictSerialize for TypeLib {}
impl StrictDeserialize for TypeLib {}

impl ToBaid58<32> for TypeLibId {
    const HRP: &'static str = "stl";
    fn to_baid58_payload(&self) -> [u8; 32] { self.to_raw_array() }
}

impl TypeLib {
    pub fn id(&self) -> TypeLibId {
        let mut hasher = blake3::Hasher::new_keyed(&LIB_ID_TAG);
        for ty in self.types.values() {
            hasher.update(ty.id().as_slice());
        }
        TypeLibId::from_raw_array(hasher.finalize())
    }
}
