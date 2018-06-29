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

use model::bitboard::*;
use model::constants::*;
use model::zobrist::{self, ZobristExt, ZobristHash};
use model::{Color, ColorMap, FieldCoord, GameType, HexCoord, Move, MoveAnnotated, Outcome};

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
    pub turn: Color,
    pub vitals: ColorMap<PlayerVitals>,
    pub zobrist: ZobristHash,
    pub hexes_to_exchange: u8,
}

/// A struct tracking a player's piece and captured hex count. So named because these two numbers are
/// essential to a player's survival (i.e. vital signs).
#[derive(Clone, Copy, PartialEq)]
pub struct PlayerVitals {
    pub pieces: u8,
    pub hexes: u8,
}

// Public methods
impl Board {
    /// Create a new board with the "Laurentius" starting position.
    pub fn new(game_type: GameType, hexes_to_exchange: u8) -> Self {
        assert!(hexes_to_exchange == 1 || hexes_to_exchange == 2);

        let starting_position = match game_type {
            GameType::Laurentius => LAURENTIUS,
            GameType::Ocius => OCIUS,
        };

        Self {
            fields: starting_position.fields,
            hexes: starting_position.hexes,
            turn: Color::White,
            vitals: starting_position.vitals,
            zobrist: zobrist::new(starting_position.fields, ColorMap::new(0, 0), Color::White),
            hexes_to_exchange,
        }
    }
    pub fn apply_move(&mut self, mv: &Move) {
        assert!(self.can_apply_move(mv), "Cannot apply {:?}", mv);
        match *mv {
            Move::Move(from, to, color) => {
                self.toggle_field(from | to, color);

                self.zobrist.toggle_field(from, color);
                self.zobrist.toggle_field(to, color);

                let (capture_count, mut fields_to_check) = self.check_hexes(from.to_index());
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
                    self.zobrist.set_hex_count(
                        vitals.hexes,
                        vitals.hexes - self.hexes_to_exchange,
                        color,
                    );
                    vitals.hexes -= self.hexes_to_exchange;
                }

                // Players don't collect hexes removed due to an exchange
                let (_, fields_to_check) = self.check_hexes(bb.to_index());
                self.check_captures(fields_to_check);
            }
        }
        self.turn = self.turn.switch();
        self.zobrist.switch_turn();
    }
    /// Applies a `Move` and returns it as a `MoveAnnotated`, that is, holding `Vec`s of the pieces
    /// and hexes removed by playing the move.
    pub fn annotated_apply_move(&mut self, mv: &Move) -> MoveAnnotated {
        let opp_color = self.turn.switch();
        let old_opp_fields = self.fields.get(opp_color);
        let old_hexes = self.hexes;

        self.apply_move(mv);

        let captured_pieces = (old_opp_fields ^ self.fields.get(opp_color))
            .iter()
            .map(|bb| FieldCoord::from_bitboard(bb, opp_color))
            .collect();

        let removed_hexes = (HEX_COORD_MASK & (old_hexes ^ self.hexes))
            .iter()
            .map(|bb| HexCoord::from_index(bb.trailing_zeros() as u8 / 3))
            .collect();

        mv.annotate(captured_pieces, removed_hexes)
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

        fields
            .iter()
            .flat_map(move |origin| {
                let empty_vertex_neighbors =
                    VERTEX_NEIGHBORS.bb_get(origin, turn) & (!fields & hexes);
                empty_vertex_neighbors
                    .iter()
                    .map(move |dest| Move::Move(origin, dest, turn))
            })
            .chain(
                opp_fields
                    .iter()
                    .map(move |exchanged| Move::Exchange(exchanged, opp_color)),
            )
    }
    pub fn generate_captures(&self) -> impl Iterator<Item = Move> {
        let hexes = self.hexes;
        let can_exchange = self.can_exchange();

        let our_color = self.turn;
        let our_fields = self.fields.get(our_color);

        let opp_color = self.turn.switch();
        let opp_fields = self.fields.get(opp_color);

        // A bitboard of pieces that can be moved to capture a hex.
        let mut hex_capture_pieces = 0;

        // By exchanging these pieces, we capture a hex, which also captures another opponent piece.
        // This does not consider "hex capture chains" where removing the first hex doesn't capture
        // an opponent piece, but causes another hex to be captured, which captures a piece. That
        // would be too hard to check for and is rare. So, we ignore some potential captures, but
        // also examine less quiescence nodes in the long run, which is (hopefully) an net gain.
        let mut exchange_captures = 0;

        for (i, hex) in HEX_MASK.iter().enumerate() {
            if self.is_hex_extant(i) && self.is_hex_maybe_removable(i) {
                let opp_piece = opp_fields & hex;
                let our_piece = our_fields & hex;

                if can_exchange && our_piece == 0 && opp_piece.is_one_bit_set() {
                    let hex_field_neighbors =
                        HEX_FIELD_NEIGHBORS.index_get(i, opp_color) & opp_fields;
                    for opp_piece in hex_field_neighbors.iter() {
                        let edge_neighbors = EDGE_NEIGHBORS.bb_get(opp_piece, opp_color) & hexes;
                        if !edge_neighbors & (our_fields | hex) == 0 {
                            exchange_captures |= opp_piece;
                            break;
                        }
                    }
                }

                if opp_piece == 0 && our_piece.is_one_bit_set() {
                    hex_capture_pieces |= our_piece;
                }
            }
        }

        exchange_captures
            .iter()
            .map(move |opp_piece| Move::Exchange(opp_piece, opp_color))
            .chain(opp_fields.iter().flat_map(move |opp_piece| {
                let edge_neighbors = EDGE_NEIGHBORS.bb_get(opp_piece, opp_color) & hexes;
                let empty_neighbor = edge_neighbors & !our_fields;

                let origins = if empty_neighbor.is_one_bit_set() {
                    VERTEX_NEIGHBORS.bb_get(empty_neighbor, our_color)
                        & our_fields
                        & !edge_neighbors
                } else {
                    // Again, we need to return a single, concrete iterator type
                    0
                };

                origins
                    .iter()
                    .map(move |our_piece| Move::Move(our_piece, empty_neighbor, our_color))
            }))
            .chain(hex_capture_pieces.iter().flat_map(move |origin| {
                let hex = HEX_MASK[origin.to_index()];
                let vertex_neighbors =
                    VERTEX_NEIGHBORS.bb_get(origin, our_color) & (!hex & !our_fields & hexes);
                vertex_neighbors
                    .iter()
                    .map(move |dest| Move::Move(origin, dest, our_color))
            }))
    }
    pub fn available_moves_for_piece(&self, field: FieldCoord) -> Vec<FieldCoord> {
        if self.is_piece_on_field(field) {
            let color = field.color();
            let vertex_neighbors = self.hexes & VERTEX_NEIGHBORS.bb_get(field.to_bitboard(), color);
            let mut moves = Vec::with_capacity(3);

            for dest in vertex_neighbors.iter() {
                moves.push(FieldCoord::from_bitboard(dest, color));
            }
            moves
        } else {
            vec![]
        }
    }
    pub fn can_exchange(&self) -> bool {
        self.vitals.get(self.turn).hexes >= self.hexes_to_exchange
    }
    pub fn is_piece_on_field(&self, coord: FieldCoord) -> bool {
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
    pub fn pieces(&self, color: Color) -> u8 {
        self.vitals.get(color).pieces
    }
    pub fn hexes(&self, color: Color) -> u8 {
        self.vitals.get(color).hexes
    }
    // This function does NOT consider draw by threefold repetition because move history is not the
    // concern of Board. See Model or AI for that.
    pub fn outcome(&self) -> Outcome {
        let fields = self.fields.get(self.turn);

        if fields == 0 {
            // No more pieces left
            Outcome::Win(self.turn.switch())
        } else if fields == self.hexes && !self.can_exchange() {
            // There are no empty fields to move to and we can't exchange
            Outcome::DrawStalemate
        } else {
            use model::Color::*;

            let wp = self.pieces(White);
            let bp = self.pieces(Black);
            let wh = self.hexes(White);
            let bh = self.hexes(Black);

            // If neither side can capture the other's pieces, the game is drawn
            if wp == 1 && bp == 1
                && (self.hexes.count_ones() as u8 / 3 + cmp::max(wh, bh) - 1
                    < self.hexes_to_exchange)
            {
                Outcome::DrawInsufficientMaterial
            } else {
                Outcome::InProgress
            }
        }
    }
}

// Field and piece methods
impl Board {
    fn toggle_field(&mut self, bb: BitBoard, color: Color) {
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
        for bb in fields_to_check.iter() {
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
        self.is_hex_maybe_removable(index)
    }
    /// Assuming this hex is empty, would it be removable?
    fn is_hex_maybe_removable(&self, index: usize) -> bool {
        // Combining colors here is okay because there won't be overlaps
        let hex = self.hexes
            & (HEX_FIELD_NEIGHBORS.index_get(index, Color::White)
                | HEX_FIELD_NEIGHBORS.index_get(index, Color::Black));

        // There are 18 combinations to check for each hex
        for &comb in &REMOVABLE_HEX_COMBS[index * 18..index * 18 + 18] {
            if hex == comb {
                return true;
            }
        }
        false
    }
    fn remove_hex(&mut self, index: usize) -> bool {
        let removable = self.is_hex_removable(index);

        if removable {
            self.hexes &= !HEX_MASK[index];
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

            let our_neighbors = HEX_FIELD_NEIGHBORS.index_get(index, self.turn);
            let their_neighbors = HEX_FIELD_NEIGHBORS.index_get(index, self.turn.switch());

            for neighbor in (self.hexes & (our_neighbors | their_neighbors)).iter() {
                let check_result = self.check_hexes(neighbor.to_index());
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
