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

use model::{BitBoard, Color, ColorMap, FieldCoord, HexCoord, Move};
use model::pop_bit;
use model::constants::*;

#[derive(Clone, Copy)]
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
}

/// A struct tracking a player's piece and captured hex count. So named because these two numbers are
/// essential to a player's survival (i.e. vital signs).
#[derive(Clone, Copy)]
struct PlayerVitals {
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

lazy_static! {
    static ref EDGE_NEIGHBORS: ColorMap<[BitBoard; 57]> = ColorMap::new(
        generate_edge_neighbors(Color::White),
        generate_edge_neighbors(Color::Black),
    );
    static ref VERTEX_NEIGHBORS: ColorMap<[BitBoard; 57]> = ColorMap::new(
        generate_vertex_neighbors(Color::White),
        generate_vertex_neighbors(Color::Black),
    );
    static ref HEX_MASK: [BitBoard; 19] = generate_hex_mask();
    static ref HEX_FIELD_NEIGHBORS: ColorMap<[BitBoard; 19]> = ColorMap::new(
        generate_hex_field_neighbors(Color::White),
        generate_hex_field_neighbors(Color::Black),
    );
    static ref REMOVABLE_HEX_COMBS: [BitBoard; 342] = generate_removable_hex_combs();
}

// Public methods
impl Board {
    /// Create a new board with the "Laurentius" starting position.
    pub fn new() -> Self {
        Self {
            fields: generate_laurentius(),
            hexes: HEX_STARTING_POSITION,
            extant_hex_count: 19,
            turn: Color::White,
            vitals: ColorMap::new(PlayerVitals::new(), PlayerVitals::new()),
            outcome: Outcome::InProgress,
        }
    }
    pub fn apply_move(&mut self, mv: &Move) {
        assert!(self.can_apply_move(mv), "Cannot apply {:?}", mv);
        match *mv {
            Move::Move(from, to, color) => {
                self.toggle_field(from, color);
                self.toggle_field(to, color);

                let (capture_count, mut fields_to_check) =
                    self.check_hexes(&FieldCoord::from_bitboard(from, color).to_hex());
                fields_to_check |= EDGE_NEIGHBORS.get_ref(color)[to.trailing_zeros() as usize];
                self.check_captures(fields_to_check);

                self.vitals.get_mut(self.turn).hexes += capture_count;
                self.turn = self.turn.switch();
            }
            Move::Exchange(bb, color) => {
                self.remove_piece(bb, color);
                self.vitals.get_mut(self.turn).hexes -= 2;

                // Players don't collect hexes removed due to an exchange
                let (_, fields_to_check) =
                    self.check_hexes(&FieldCoord::from_bitboard(bb, color).to_hex());
                self.check_captures(fields_to_check);
                self.turn = self.turn.switch();
            }
        }
        self.update_outcome();
    }
    pub fn can_apply_move(&self, mv: &Move) -> bool {
        match *mv {
            Move::Move(from, to, color) => {
                let vertex_neighbors =
                    VERTEX_NEIGHBORS.get_ref(color)[from.trailing_zeros() as usize];
                color == self.turn && (to & vertex_neighbors != 0)
                    && self.is_piece_on_bitboard(from, color)
                    && !self.is_piece_on_bitboard(to, color)
            }
            Move::Exchange(bb, color) => {
                self.can_exchange() && color != self.turn && self.is_piece_on_bitboard(bb, color)
            }
        }
    }
    pub fn generate_moves(&self) -> Vec<Move> {
        let turn = self.turn();
        // A player with no pieces cannot make any moves, including exchange moves
        if self.pieces(turn) == 0 {
            return vec![];
        }

        let can_exchange = self.can_exchange();
        let mut moves = Vec::with_capacity(
            // 3 moves per piece is an untested guess
            self.pieces(turn) as usize * 3 + if can_exchange {
                self.pieces(turn.switch()) as usize
            } else {
                0
            },
        );

        let color = turn;
        let mut us = *self.fields.get_ref(color);
        while us != 0 {
            let from = pop_bit(&mut us);
            let mut vertex_neighbors =
                VERTEX_NEIGHBORS.get_ref(color)[from.trailing_zeros() as usize];
            vertex_neighbors &= !self.fields.get_ref(color) & self.hexes;

            while vertex_neighbors != 0 {
                let to = pop_bit(&mut vertex_neighbors);
                moves.push(Move::Move(from, to, color));
            }
        }

        if can_exchange {
            let color = turn.switch();
            let mut them = *self.fields.get_ref(color);
            while them != 0 {
                moves.push(Move::Exchange(pop_bit(&mut them), color));
            }
        }
        moves
    }
    pub fn available_moves_for_piece(&self, field: &FieldCoord) -> Vec<FieldCoord> {
        if self.is_piece_on_field(field) {
            let color = field.color();
            let mut vertex_neighbors =
                self.hexes & VERTEX_NEIGHBORS.get_ref(color)[field.to_index()];
            let mut moves = Vec::with_capacity(3);

            while vertex_neighbors != 0 {
                moves.push(FieldCoord::from_bitboard(
                    pop_bit(&mut vertex_neighbors),
                    color,
                ));
            }
            moves
        } else {
            vec![]
        }
    }
    pub fn can_exchange(&self) -> bool {
        self.vitals.get_ref(self.turn).hexes >= 2
    }
    pub fn is_piece_on_field(&self, coord: &FieldCoord) -> bool {
        assert!(
            self.is_hex_extant(&coord.to_hex()),
            "Cannot cannot check if piece is on {:?}. Hex at {:?} was removed.",
            coord,
            coord.to_hex()
        );

        coord.to_bitboard() & self.fields.get_ref(coord.color()) != 0
    }
    pub fn is_piece_on_bitboard(&self, bb: BitBoard, color: Color) -> bool {
        assert!(
            bb & self.hexes != 0,
            "Cannot cannot check if piece is on {:?}. Hex was removed.",
            FieldCoord::from_bitboard(bb, color),
        );

        bb & self.fields.get_ref(color) != 0
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
    pub fn is_hex_extant(&self, coord: &HexCoord) -> bool {
        self.hexes & HEX_MASK[coord.to_index()] != 0
    }
    pub fn resign(&mut self) {
        assert_eq!(self.outcome, Outcome::InProgress);
        self.outcome = Outcome::Win(self.turn.switch());
    }
    pub fn turn(&self) -> Color {
        self.turn
    }
    pub fn pieces(&self, color: Color) -> u8 {
        self.vitals.get_ref(color).pieces
    }
    pub fn hexes(&self, color: Color) -> u8 {
        self.vitals.get_ref(color).hexes
    }
    pub fn outcome(&self) -> Outcome {
        self.outcome
    }
}

impl Board {
    // TODO: check for threefold repetition
    fn update_outcome(&mut self) {
        if self.pieces(self.turn) == 0 {
            self.outcome = Outcome::Win(self.turn.switch());
        } else if !self.can_move() {
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
    fn can_move(&self) -> bool {
        if self.can_exchange() {
            return true;
        }

        let color = self.turn;
        let mut us = *self.fields.get_ref(color);
        while us != 0 {
            let field = FieldCoord::from_bitboard(pop_bit(&mut us), color);
            let vertex_neighbors = VERTEX_NEIGHBORS.get_ref(color)[field.to_index()];
            if self.is_piece_on_field(&field)
                && (vertex_neighbors & self.fields.get_ref(color) != vertex_neighbors)
            {
                return true;
            }
        }
        false
    }
}

// Field and piece methods
impl Board {
    fn toggle_field(&mut self, bb: BitBoard, color: Color) {
        assert!(
            bb & self.hexes != 0,
            "Cannot set field {:?}. Hex was removed.",
            FieldCoord::from_bitboard(bb, color),
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
        fields_to_check &= self.hexes & self.fields.get_ref(them);
        while fields_to_check != 0 {
            let bb = pop_bit(&mut fields_to_check);
            let neighbors = self.hexes & EDGE_NEIGHBORS.get_ref(them)[bb.trailing_zeros() as usize];
            if !self.fields.get_ref(us) & neighbors == 0 {
                self.remove_piece(bb, them);
            }
        }
    }
}

// Hex methods
impl Board {
    fn get_hex_neighbor(&self, coord: &HexCoord, direction: u8) -> Option<HexCoord> {
        self.try_hex(match direction {
            0 => (coord.x, coord.y + 1),
            1 => (coord.x + 1, coord.y),
            2 => (coord.x + 1, coord.y - 1),
            3 => (coord.x, coord.y - 1),
            4 => (coord.x - 1, coord.y),
            5 => (coord.x - 1, coord.y + 1),
            _ => panic!("Direction must be less than 6"),
        })
    }
    /// A hex is removable (and must be removed) if it is empty and is "attached to the board by 3
    /// or less adjacent sides."
    fn is_hex_removable(&self, coord: &HexCoord) -> bool {
        let index = coord.to_index();

        let is_hex_empty =
            (self.fields.white | self.fields.black) & HEX_MASK[index] == 0;
        if !self.is_hex_extant(coord) || !is_hex_empty {
            return false;
        }

        let hex =
            self.hexes & (HEX_FIELD_NEIGHBORS.white[index] | HEX_FIELD_NEIGHBORS.black[index]);
        // There are 18 combinations to check for each hex
        REMOVABLE_HEX_COMBS
            .iter()
            .skip(index * 18)
            .take(18)
            .any(|&comb| hex == comb)
    }
    fn remove_hex(&mut self, coord: &HexCoord) -> bool {
        let removable = self.is_hex_removable(coord);

        if removable {
            self.hexes &= !HEX_MASK[coord.to_index()];
            self.extant_hex_count -= 1;
        }
        removable
    }
    fn try_hex(&self, coord: (i8, i8)) -> Option<HexCoord> {
        if let Some(coord) = HexCoord::try_new(coord.0, coord.1) {
            if self.is_hex_extant(&coord) {
                return Some(coord);
            }
        }
        None
    }
    fn check_hexes(&mut self, coord: &HexCoord) -> (u8, BitBoard) {
        let mut remove_count = 0;
        let mut fields = 0;

        if self.remove_hex(coord) {
            remove_count += 1;
            for f in 0..6 {
                if let Some(neighbor) = self.get_hex_neighbor(coord, f) {
                    let (new_remove_count, new_fields) = self.check_hexes(&neighbor);
                    if new_remove_count == 0 {
                        // TODO: actually use HEX_FIELD_NEIGHBORS?
                        let field = neighbor.to_field((f + 3) % 6);
                        if field.color() != self.turn() {
                            fields |= field.to_bitboard();
                        }
                    } else {
                        remove_count += new_remove_count;
                        fields |= new_fields;
                    }
                }
            }
        }
        (remove_count, fields)
    }
}
