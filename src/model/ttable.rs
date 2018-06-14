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
pub struct TTable {
    table: Vec<Entry>,
    age: u8,
}

impl TTable {
    pub fn new() -> Self {
        Self {
            table: vec![Default::default(); TABLE_SIZE],
            age: 0,
        }
    }
    pub fn inc_age(&mut self) {
        self.age.wrapping_add(1);
    }
    pub fn get(&self, zobrist: ZobristHash) -> &Entry {
        let hash = (zobrist & TABLE_MASK) as usize;
        let mut entry = self.table[hash];
        if entry.zobrist != 0 {
            entry.age = self.age;
        }
        &self.table[hash]
    }
    pub fn set(&mut self, zobrist: ZobristHash, eval_type: EvalType, depth: u8, score: i16) {
        let hash = (zobrist & TABLE_MASK) as usize;
        let mut entry = self.table[hash];
        let mut replace = false;
        if entry.zobrist != 0 {
            // TODO: Fine tune this score calculation?
            if depth + self.age.wrapping_sub(entry.age) > entry.depth {
                replace = true;
            }
        } else {
            replace = true;
        }

        if replace {
            entry.eval_type = eval_type;
            entry.age = self.age;
            entry.depth = depth;
            entry.score = score;
            entry.zobrist = zobrist;
        }
    }
}

#[derive(Clone, Copy)]
pub enum EvalType {
    Exact,
    Beta,
}

// TODO: Store best move for move ordering?
// TODO: Use lower bits of ZobristHash to save space?
#[derive(Clone, Copy)]
pub struct Entry {
    pub eval_type: EvalType,
    pub age: u8,
    pub depth: u8,
    pub score: i16,
    pub zobrist: ZobristHash,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            eval_type: EvalType::Exact,
            age: 0,
            depth: 0,
            score: 0,
            // The only field that matters for determining if this is an empty entry or not.
            // Assume (and hope) that no valid board ever hashes to 0.
            zobrist: 0,
        }
    }
}
