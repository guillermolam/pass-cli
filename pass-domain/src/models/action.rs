/*
 *  Copyright (c) 2026 Proton AG
 *  This file is part of Proton AG and Proton Pass.
 *
 *  Proton Pass is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  Proton Pass is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with Proton Pass.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

#[derive(Clone, Copy, Debug, serde::Serialize, Default, strum::FromRepr)]
#[repr(u64)]
pub enum EventAction {
    #[default]
    Unknown = 0,
    VaultUpdate = 2,
    VaultSoftDelete = 3,
    ItemCreate = 20,
    ItemUpdate = 21,
    ItemTrash = 22,
    ItemUntrash = 23,
    ItemSoftDelete = 24,
    ItemRead = 31,
    PublicLinkCreated = 60,
    PublicLinkDeleted = 61,
}

impl EventAction {
    pub fn value(self) -> u64 {
        self as u64
    }

    pub fn from(value: u64) -> Option<Self> {
        Self::from_repr(value)
    }
}
