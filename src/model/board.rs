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

use std::cmp;

use model::bitboard::{BitBoard, BitBoardIter};
use model::constants::*;
use model::zobrist::ZobristHash;
use model::{Color, ColorMap, FieldCoord, HexCoord, Move};

#[derive(Clone, Copy, PartialEq)]
pub struct Board {
    /*
    Board layout:
    The hex board uses an axial coordinate system with (0, 0) at the center. The x-axis slopes
    up and to the right, and the y-axis goes up and down.
    See http://www.redblobgames.com/grids/hexagons/#coordinates-axial for more info.
                                             ____
                                            /    \
                                       ____/  +y  \____
                                      /    \      /    \
                                     /      \____/  +x  \
                                     \      /    \      /
                                      \____/      \____/
                                      /    \      /    \
                                     /  -x  \____/      \
                                     \      /    \      /
                                      \____/  -y  \____/
                                           \      /
                                            \____/

    Field layout:
    Fields are numbered clockwise from the top. Even indicies are black, odd indicies are white.
                                          _________
                                         / \     / \
                                        /   \ 0 /   \
                                       /  5  \ /  1  \
                                      (-------*-------)
                                       \  4  / \  2  /
                                        \   / 3 \   /
                                         \_/_____\_/
    u64 Bitboard layout:

     MSB                                                                              LSB

     7 bits                            57 bits (19 groups of 3)
     +-----+  +-------------------------------------------------------------------------+
    [0000000][000 111 000 000 000 000 000 000 000 000 000 000 000 000 000 000 000 010 000]
     Unused   -+- -+-                 ||+-- Field 0           ||+-- Field 1       |+-- Piece on field
               |   +-- Extant hex     |+--- Field 2           |+--- Field 3       +--- No piece on field
               +------ Removed hex    +---- Field 4           +---- Field 5            (Field bitboards)
                      (Hex bitboard)  (Black field bitboard)  (White field bitboard)
    */
    fields: ColorMap<BitBoard>,
    hexes: BitBoard,
    extant_hex_count: u8,
    turn: Color,
    vitals: ColorMap<PlayerVitals>,
    outcome: Outcome,
    zobrist: ZobristHash,
}

/// A struct tracking a player's piece and captured hex count. So named because these two numbers are
/// essential to a player's survival (i.e. vital signs).
#[derive(Clone, Copy, PartialEq)]
pub struct PlayerVitals {
    pieces: u8,
    hexes: u8,
}

impl PlayerVitals {
    fn new() -> Self {
        Self {
            pieces: 18,
            hexes: 0,
        }
    }
}

/// The outcome of a game. Wins or draws caused by a resignation or an offered and accepted draw are
/// not differentiated from wins and draws by capturing all of an opponent's pieces, running out of
/// moves, being unable to capture any of the opponent's pieces, or reaching threefold repetition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    InProgress,
    Win(Color),
    Draw,
}

// Public methods
impl Board {
    /// Create a new board with the "Laurentius" starting position.
    pub fn new() -> Self {
        let fields = generate_laurentius();

        Self {
            fields,
            hexes: HEX_STARTING_POSITION,
            extant_hex_count: 19,
            turn: Color::White,
            vitals: ColorMap::new(PlayerVitals::new(), PlayerVitals::new()),
            outcome: Outcome::InProgress,
            zobrist: ZobristHash::new(fields, ColorMap::new(0, 0), Color::White),
        }
    }
    pub fn apply_move(&mut self, mv: &Move) {
        assert!(self.can_apply_move(mv), "Cannot apply {:?}", mv);
        match *mv {
            Move::Move(from, to, color) => {
                self.toggle_field(from | to, color);

                self.zobrist.toggle_field(from, color);
                self.zobrist.toggle_field(to, color);

                let (capture_count, mut fields_to_check) =
                    self.check_hexes(from.trailing_zeros() as usize / 3);
                fields_to_check |= EDGE_NEIGHBORS.bb_get(to, color);
                self.check_captures(fields_to_check);

                if capture_count != 0 {
                    let vitals = self.vitals.get_mut(self.turn);
                    self.zobrist
                        .set_hex_count(vitals.hexes, vitals.hexes + capture_count, color);
                    vitals.hexes += capture_count;
                }
            }
            Move::Exchange(bb, color) => {
                self.remove_piece(bb, color);

                {
                    let vitals = self.vitals.get_mut(self.turn);
                    self.zobrist
                        .set_hex_count(vitals.hexes, vitals.hexes - 2, color);
                    vitals.hexes -= 2;
                }

                // Players don't collect hexes removed due to an exchange
                let (_, fields_to_check) = self.check_hexes(bb.trailing_zeros() as usize / 3);
                self.check_captures(fields_to_check);
            }
        }
        self.turn = self.turn.switch();
        self.zobrist.switch_turn();
        self.update_outcome();
    }
    pub fn can_apply_move(&self, mv: &Move) -> bool {
        match *mv {
            Move::Move(from, to, color) => {
                let vertex_neighbors = VERTEX_NEIGHBORS.bb_get(from, color);
                color == self.turn
                    && (to & vertex_neighbors != 0)
                    && self.is_piece_on_bitboard(from, color)
                    && !self.is_piece_on_bitboard(to, color)
            }
            Move::Exchange(bb, color) => {
                self.can_exchange() && color != self.turn && self.is_piece_on_bitboard(bb, color)
            }
        }
    }
    pub fn generate_moves(&self) -> impl Iterator<Item = Move> {
        let turn = self.turn;
        let fields = self.fields.get(turn);

        assert_ne!(fields, 0);

        let hexes = self.hexes;
        let opp_color = turn.switch();
        let opp_fields = if self.can_exchange() {
            self.fields.get(opp_color)
        } else {
            // impl Trait requires that we return a single, concrete type. So, if there are no
            // fields to exchange, we create an empty BitBoardIter and chain it on anyways. This
            // way, the type of the resulting iterator is always Chain.
            0
        };

        BitBoardIter::new(fields)
            .flat_map(move |origin| {
                let vertex_neighbors = VERTEX_NEIGHBORS.bb_get(origin, turn) & (!fields & hexes);
                BitBoardIter::new(vertex_neighbors).map(move |dest| Move::Move(origin, dest, turn))
            })
            .chain(
                BitBoardIter::new(opp_fields)
                    .map(move |exchanged| Move::Exchange(exchanged, opp_color)),
            )
    }
    pub fn available_moves_for_piece(&self, field: &FieldCoord) -> Vec<FieldCoord> {
        if self.is_piece_on_field(field) {
            let color = field.color();
            let vertex_neighbors = self.hexes & VERTEX_NEIGHBORS.bb_get(field.to_bitboard(), color);
            let mut moves = Vec::with_capacity(3);

            for dest in BitBoardIter::new(vertex_neighbors) {
                moves.push(FieldCoord::from_bitboard(dest, color));
            }
            moves
        } else {
            vec![]
        }
    }
    pub fn can_exchange(&self) -> bool {
        self.vitals.get(self.turn).hexes >= 2
    }
    pub fn is_piece_on_field(&self, coord: &FieldCoord) -> bool {
        self.is_piece_on_bitboard(coord.to_bitboard(), coord.color())
    }
    pub fn is_piece_on_bitboard(&self, bb: BitBoard, color: Color) -> bool {
        assert!(
            bb & self.hexes != 0,
            "Cannot cannot check if piece is on {:?}. Hex was removed.",
            FieldCoord::from_bitboard(bb, color),
        );

        bb & self.fields.get(color) != 0
    }
    /// > extant (adj.): Still in existence; not destroyed, lost, or extinct (The Free Dictionary)
    ///
    /// Return the coordinates of the hexes that have not been removed yet.
    pub fn extant_hexes(&self) -> Vec<HexCoord> {
        let mut coords = Vec::with_capacity(19);
        let try_coord = |coords: &mut Vec<HexCoord>, x, y| {
            if let Some(hex) = self.try_hex((x, y)) {
                coords.push(hex);
            }
        };
        try_coord(&mut coords, 1, 1);
        try_coord(&mut coords, -1, -1);

        for x in 0..3 {
            for y in -2..1 {
                try_coord(&mut coords, x, y);
                if x != y {
                    try_coord(&mut coords, -x, -y);
                }
            }
        }
        coords
    }
    /// > extant (adj.): Still in existence; not destroyed, lost, or extinct (The Free Dictionary)
    ///
    /// Returns true if a hex has not been removed yet.
    pub fn is_hex_extant(&self, index: usize) -> bool {
        self.hexes & HEX_MASK[index] != 0
    }
    pub fn resign(&mut self) {
        assert_eq!(self.outcome, Outcome::InProgress);
        self.outcome = Outcome::Win(self.turn.switch());
    }
    pub fn turn(&self) -> Color {
        self.turn
    }
    pub fn pieces(&self, color: Color) -> u8 {
        self.vitals.get(color).pieces
    }
    pub fn hexes(&self, color: Color) -> u8 {
        self.vitals.get(color).hexes
    }
    pub fn vitals(&self) -> ColorMap<PlayerVitals> {
        self.vitals
    }
    pub fn outcome(&self) -> Outcome {
        self.outcome
    }
}

impl Board {
    // TODO: check for threefold repetition
    fn update_outcome(&mut self) {
        let fields = self.fields.get(self.turn);

        if fields == 0 {
            // No more pieces left
            self.outcome = Outcome::Win(self.turn.switch());
        } else if fields == self.hexes && !self.can_exchange() {
            // There are no empty fields to move to and we can't exchange
            self.outcome = Outcome::Draw;
        } else {
            use model::Color::*;

            let wp = self.pieces(White);
            let bp = self.pieces(Black);
            let wh = self.hexes(White);
            let bh = self.hexes(Black);

            // If neither side can capture the other's pieces, the game is drawn
            if wp == 1 && bp == 1 && (self.extant_hex_count + cmp::max(wh, bh) - 1 < 2) {
                self.outcome = Outcome::Draw;
            }
        }
    }
}

// Field and piece methods
impl Board {
    fn toggle_field(&mut self, bb: BitBoard, color: Color) {
        assert_ne!(
            bb & self.hexes,
            0,
            "Trying to toggle field(s) on removed hex(es)"
        );

        *self.fields.get_mut(color) ^= bb;
    }
    fn remove_piece(&mut self, bb: BitBoard, color: Color) {
        assert!(
            self.is_piece_on_bitboard(bb, color),
            "There is no piece at {:?} to remove",
            FieldCoord::from_bitboard(bb, color)
        );
        self.toggle_field(bb, color);
        self.vitals.get_mut(color).pieces -= 1;
    }
    fn check_captures(&mut self, mut fields_to_check: BitBoard) {
        // fields_to_check must be a BitBoard for the opponent player (i.e. opposite of current turn)
        let us = self.turn;
        let them = us.switch();
        fields_to_check &= self.hexes & self.fields.get(them);
        for bb in BitBoardIter::new(fields_to_check) {
            let neighbors = self.hexes & EDGE_NEIGHBORS.bb_get(bb, them);
            if !self.fields.get(us) & neighbors == 0 {
                self.remove_piece(bb, them);
            }
        }
    }
}

// Hex methods
impl Board {
    /// A hex is removable (and must be removed) if it is empty and is "attached to the board by 3
    /// or less adjacent sides."
    fn is_hex_removable(&self, index: usize) -> bool {
        debug_assert!(self.is_hex_extant(index));

        if (self.fields.white | self.fields.black) & HEX_MASK[index] != 0 {
            return false;
        }

        // Combining colors here is okay because there won't be overlaps
        let hex = self.hexes
            & (HEX_FIELD_NEIGHBORS.index_get(index, Color::White)
                | HEX_FIELD_NEIGHBORS.index_get(index, Color::Black));
        // There are 18 combinations to check for each hex
        REMOVABLE_HEX_COMBS
            .iter()
            .skip(index * 18)
            .take(18)
            .any(|&comb| hex == comb)
    }
    fn remove_hex(&mut self, index: usize) -> bool {
        let removable = self.is_hex_removable(index);

        if removable {
            self.hexes &= !HEX_MASK[index];
            self.extant_hex_count -= 1;
        }
        removable
    }
    fn try_hex(&self, coord: (i8, i8)) -> Option<HexCoord> {
        if let Some(coord) = HexCoord::try_new(coord.0, coord.1) {
            if self.is_hex_extant(coord.to_index()) {
                return Some(coord);
            }
        }
        None
    }
    fn check_hexes(&mut self, index: usize) -> (u8, BitBoard) {
        let mut remove_count = 0;
        let mut fields = 0;

        if self.remove_hex(index) {
            remove_count += 1;

            let our_neighbors = self.hexes & HEX_FIELD_NEIGHBORS.index_get(index, self.turn);
            let their_neighbors =
                self.hexes & HEX_FIELD_NEIGHBORS.index_get(index, self.turn.switch());

            for neighbor in BitBoardIter::new(our_neighbors | their_neighbors) {
                let check_result = self.check_hexes(neighbor.trailing_zeros() as usize / 3);
                remove_count += check_result.0;
                fields |= check_result.1;
            }

            // Add in the opponent's neighbors, excluding those on hexes that have been removed
            fields |= their_neighbors;
            fields &= self.hexes;
        }
        (remove_count, fields)
    }
}
