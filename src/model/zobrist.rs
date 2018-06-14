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

#![cfg_attr(feature = "cargo-clippy", allow(unreadable_literal))]

use model::bitboard::{BitBoard, BitBoardIter};
use model::{Color, ColorMap};

pub type ZobristHash = u64;

pub fn new(fields: ColorMap<BitBoard>, hex_count: ColorMap<u8>, turn: Color) -> ZobristHash {
    let mut hash = 0;

    for (w, b) in BitBoardIter::new(fields.white).zip(BitBoardIter::new(fields.black)) {
        hash ^= PIECE_FIELD.white[w.trailing_zeros() as usize];
        hash ^= PIECE_FIELD.black[b.trailing_zeros() as usize];
    }

    hash ^= HEX_COUNT.white[hex_count.white as usize];
    hash ^= HEX_COUNT.black[hex_count.black as usize];

    if turn == Color::White {
        hash ^= WHITE_TO_MOVE;
    }

    hash
}

pub trait ZobristExt {
    fn toggle_field(&mut self, bb: BitBoard, color: Color);
    fn set_hex_count(&mut self, old: u8, new: u8, color: Color);
    fn switch_turn(&mut self);
}

impl ZobristExt for ZobristHash {
    fn toggle_field(&mut self, bb: BitBoard, color: Color) {
        *self ^= PIECE_FIELD.get_ref(color)[bb.trailing_zeros() as usize];
    }

    fn set_hex_count(&mut self, old: u8, new: u8, color: Color) {
        let hex_count = HEX_COUNT.get(color);
        *self ^= hex_count[old as usize];
        *self ^= hex_count[new as usize];
    }

    fn switch_turn(&mut self) {
        *self ^= WHITE_TO_MOVE;
    }
}

// These constants were generated with random.org
const WHITE_TO_MOVE: u64 = 0xb047cbc27fa474a6;

#[cfg_attr(rustfmt, rustfmt_skip)]
const HEX_COUNT: ColorMap<[u64; 18]> = ColorMap {
    white: [
        0x342ebe3aba0639e1, 0x85c3a94db6c410f8, 0x4a59e5d60c9c2578, 0x4f1c7aace25eaa2c, 0xa3bb92a83f5da3d8, 0x72212cadb3bc08fe, 0x233681dee2d6d5ff,
        0x5ef3c73350d9bda7, 0x8e38ec3164fc38a5, 0x03a042653893697a, 0x3f9979708df40801, 0xd5f60223a2b55bb7, 0xa808ebb244396dd9, 0x28c33cc450806c13,
        0x94e49b2aa83c22e2, 0xabbec57dfa22db0d, 0xe227882ce7892361, 0xd70c37a5db48a0cc
    ],
    black: [
        0xdc4fd270fe0e0416, 0xf585004def794791, 0x2ff23cf836844275, 0xac8ce52f5b958225, 0x84a9889ef13bee3d, 0x719faa8abf9ff555, 0x4528e720356d4e8d,
        0xa39df2fd9d955262, 0xcc348023546a1ba5, 0x9ffd7521b84169cc, 0x57e0760f168ccab0, 0xaba5b00ae8f2caa6, 0x22e5aeb8b589236d, 0x28b105cc511df709,
        0x0275768f21d8a85f, 0x8f54352baa0240c9, 0x8c2effd571b30cb9, 0x942fb16a30924b22
    ]
};

#[cfg_attr(rustfmt, rustfmt_skip)]
const PIECE_FIELD: ColorMap<[u64; 57]> = ColorMap {
    white: [
        0x3f7a3a29caa1b6d0, 0xd1b37d9434ecaa46, 0x9386bf627bfcb69c, 0xd589972483045b43, 0x2aa564f33adb00b5, 0xe7625c3ed0b8e824, 0x0a5d51afa7eed359,
        0x0622a73c0a4c2b0e, 0x78e6b5df45d04ace, 0xaa145dd19db5dc2e, 0x5cedf536370a3fbd, 0x294253fbd719b9f2, 0x6e88fbfa10a66590, 0x7b123dae75d46126,
        0x3c30e6e9654bfb11, 0x2128c41b2a95d1ce, 0xc5b88b05cbc74161, 0x527910a75923246a, 0xb4ff3fdad94dd71c, 0x682e90c4379a6ecb, 0x3dd43e445be175b6,
        0x1432672741ad5be6, 0x60033e0b11adddd3, 0xda9ec847cf948ceb, 0xd89f7e9b7025cfa1, 0x842b144ed55136ee, 0x22c9bc8740ff041f, 0xbea9603c562af301,
        0x52d3767a794dc122, 0xd174f7002164c5bb, 0x3a11be246c6b1df3, 0x29a73875bbb53b35, 0x60e1006064ed2fd0, 0x4260792bd25dbc22, 0x5c98aa8c0f8c1b0c,
        0x84c08894a69b00a0, 0x31d1a4ee50bda292, 0x0f0451e31231301c, 0x81397987a0edc0f1, 0x008a0dac7e8d2006, 0x5629df139e717bc2, 0x9ad42e175a9daa47,
        0x1f71bafa5d1facce, 0x0e24e6e4e3893bef, 0xbb6cc826cb1d6d41, 0xefd12eaaaced6d5a, 0x65ca57879ff16e86, 0x9bf5031a431f00a1, 0x9df2f2c66d2376f8,
        0x8038b142e5d3f482, 0xb43a4f13baf9d569, 0x6ded1bb43e1cd19d, 0xbe31c1eb2c9bcf15, 0xdd6fa06971c67a1c, 0xefa70b3af71d2d70, 0xa39d595872d89c84,
        0xca501e0a9fecbc4d
    ],
    black: [
        0xf387c474ba6cc1b8, 0x626d96716dea6238, 0xa9e3ad33ee222d46, 0xb5123225c387678b, 0x82aeef3da3f628eb, 0x8bdab9ddcf87c506, 0x802966dc01df4cd9,
        0xda37b5aed3ce2ab4, 0xcc02fd17db8cb9ec, 0xdf8a8af25d7e6502, 0x73d3829524b806f8, 0x444ec2cfd00b3ab6, 0x703fbedc5cbb5ff3, 0xd98f928b76ed259a,
        0x66738e2b3e24a1e9, 0x0a39f901b8e9664d, 0xce99942dada06891, 0x5c3eecbfb5fb37e9, 0x7030467b059bea65, 0xf2d1e9b72c56da26, 0x305aa0cb1ebcb015,
        0xbbfbbfc1f02abbfa, 0x12f4e6dc3a9b0394, 0xbb601f50db527bab, 0x15a71f71341ce096, 0x7cfe664726b1d21b, 0x289331725ef2e552, 0xdc451b0fa2157b9a,
        0x0a8a30a205f899bf, 0x1c09fa747ffe7734, 0x6eb8954087c70439, 0x7319d2c4f2ceac48, 0x3800fd0e35ace820, 0xb09e7c6326c3a22b, 0x3a29b3f454d799ec,
        0x549e1ecd7d2952ec, 0xbe8dc57ac717460a, 0xd722f3a1bc6f173a, 0xe0d1d6cee798e11e, 0x57d969acea163c25, 0xcc80a018e8698596, 0x8b7246e1df9a268e,
        0x64099808ca8eab9a, 0xcaa925fa97ef8c54, 0x591a24b4332ab894, 0x67311cc8b14481c2, 0x8408649ec212c0aa, 0x6657835d267abe52, 0xda3078ea008febe7,
        0x9ca7f8c6691910ba, 0xa153c7397dca67a8, 0xd4dce795048a3c99, 0xf4c42004d59173d5, 0x13f30069ef419cf1, 0xb127b093af118466, 0xc425e30bb162ad36,
        0x616c7c649457c74a
    ]
};
