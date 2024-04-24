// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

mod force;
mod nft;
mod rarity_tier;
mod tournament;
mod versions;

pub(crate) use versions::*;
pub use {force::*, rarity_tier::*, tournament::*};

use {
    frame_support::pallet_prelude::*,
    sp_std::{ops::Range, prelude::*},
};

pub type IpfsUrl = BoundedVec<u8, MaxIpfsUrl>;
pub struct MaxIpfsUrl;
impl Get<u32> for MaxIpfsUrl {
    fn get() -> u32 {
        80
    }
}

pub type SeasonId = u16;
pub type Dna = BoundedVec<u8, ConstU32<100>>;
pub type SoulCount = u32;

/// Used to indicate which version of the forging and/or mint logic should be used.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub enum LogicGeneration {
    #[default]
    First,
    Second,
    Third,
}

/// Used to indicate the layout of an avatars DNA byte sequence.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq, Eq)]
pub enum DnaEncoding {
    #[default]
    V1,
    V2,
    V3,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq, Eq)]
pub struct Avatar<BlockNumber> {
    pub season_id: SeasonId,
    pub encoding: DnaEncoding,
    pub dna: Dna,
    pub souls: SoulCount,
    pub minted_at: BlockNumber,
}

impl<BlockNumber> Avatar<BlockNumber>
where
    BlockNumber: sp_runtime::traits::BlockNumber,
{
    pub(crate) fn rarity(&self) -> u8 {
        match self.encoding {
            DnaEncoding::V1 => AttributeMapperV1::rarity(self),
            DnaEncoding::V2 => AttributeMapperV2::rarity(self),
            DnaEncoding::V3 => AttributeMapperV3::rarity(self),
        }
    }

    pub(crate) fn force(&self) -> u8 {
        match self.encoding {
            DnaEncoding::V1 => AttributeMapperV1::force(self),
            DnaEncoding::V2 => AttributeMapperV2::force(self),
            DnaEncoding::V3 => AttributeMapperV3::force(self),
        }
    }
}

pub(crate) trait ByteConvertible: Clone {
    fn from_byte(byte: u8) -> Self;
    fn as_byte(&self) -> u8;
}

impl ByteConvertible for u8 {
    fn from_byte(byte: u8) -> Self {
        byte
    }

    fn as_byte(&self) -> u8 {
        *self
    }
}

pub(crate) trait Ranged {
    fn range() -> Range<usize>;
}