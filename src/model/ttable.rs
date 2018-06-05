/*
 * Copyright (C) 2017-2018 Ryan Huang
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use model::zobrist::ZobristHash;

const TABLE_SIZE: usize = 1 << 20;
const TABLE_MASK: u64 = TABLE_SIZE as u64 - 1;

// This could just by an array, but because arrays are allocated on the stack (even when
// doing Box::new(array)), we need to use a Vec
pub struct TTable(Vec<Option<Entry>>);

impl TTable {
    pub fn new() -> Self {
        TTable(vec![None; TABLE_SIZE])
    }
    pub fn get(&self, zobrist: ZobristHash) -> &Option<Entry> {
        let hash = zobrist.get() & TABLE_MASK;
        &self.0[hash as usize]
    }
    // TODO: Use more sophisticated replacement strategy
    pub fn set(&mut self, zobrist: ZobristHash, entry: Entry) {
        let hash = zobrist.get() & TABLE_MASK;
        self.0[hash as usize] = Some(entry);
    }
}

#[derive(Clone, Copy)]
pub enum EvalType {
    Exact,
    Beta,
}

// TODO: Store best move for move ordering?
#[derive(Clone, Copy)]
pub struct Entry {
    pub eval_type: EvalType,
    pub depth: u8,
    pub score: i16,
    pub zobrist: ZobristHash,
}
