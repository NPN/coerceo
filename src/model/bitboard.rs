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

pub type BitBoard = u64;

pub struct BitBoardIter {
    bb: BitBoard,
}

impl BitBoardIter {
    pub fn new(bb: BitBoard) -> Self {
        Self { bb }
    }
}

impl Iterator for BitBoardIter {
    type Item = BitBoard;
    // Pop the least significant set bit
    fn next(&mut self) -> Option<Self::Item> {
        if self.bb != 0 {
            // (1 + !self.bb) is a two's complement negation for u64
            let bit = self.bb & (1 + !self.bb);
            self.bb ^= bit;
            Some(bit)
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.bb.count_ones() as usize;
        (n, Some(n))
    }
}
