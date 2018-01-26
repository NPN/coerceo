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

use model::{Color, ColorMap, FieldCoord, HexCoord, Move};

#[derive(Clone, Copy)]
pub struct Board {
    /*
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

        The hex board uses an axial coordinate system with (0, 0) at the center. The x-axis slopes
        up to the right, and the y-axis goes up and down. The board is stored as a 1D array.
        See http://www.redblobgames.com/grids/hexagons/#coordinates-axial for more info.

        u8 hex layout:
                         Field number. Fields are numbered clockwise from the top.
               543210 -- Even indicies are black, odd indicies are white.
        [0][0][000000]
         |  |    +------ Store fields
         |  +----------- Has hex been removed?
         +-------------- Unused
    */
    board: [u8; 25],
    extant_hexes: u8,
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
    fn new() -> PlayerVitals {
        PlayerVitals {
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

// There's no need to store the color of the piece on this field, since only white pieces can be on
// white fields, and vice versa.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Field {
    Piece,
    Empty,
}

// Public methods
impl Board {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    /// Create a new board with the "Laurentius" starting position.
    pub fn new() -> Board {
        let mut board = Board {
            board: [0; 25],
            extant_hexes: 19,
            turn: Color::White,
            vitals: ColorMap::new(
                PlayerVitals::new(),
                PlayerVitals::new(),
            ),
            outcome: Outcome::InProgress
        };

        // (0, 0) is the only empty hex.
        board.set_hex(&HexCoord::new(0, 0), 0b1_000000);

        // All other hexes have exactly two pieces on them in the starting position.
        let piece_locations = [
            (-2,  2, 0, 4),
            (-2,  1, 0, 3),
            (-2,  0, 3, 5),
            (-1,  2, 1, 4),
            (-1,  1, 0, 4),
            (-1,  0, 3, 5),
            (-1, -1, 2, 5),
            ( 0,  2, 1, 5),
            ( 0,  1, 1, 5),
            ( 0, -1, 2, 4),
            ( 0, -2, 2, 4),
            ( 1,  1, 2, 5),
            ( 1,  0, 0, 2),
            ( 1, -1, 1, 3),
            ( 1, -2, 1, 4),
            ( 2,  0, 0, 2),
            ( 2, -1, 0, 3),
            ( 2, -2, 1, 3),
        ];

        for &(x, y, f1, f2) in &piece_locations {
            let hex = 0b1_000000 + (1 << f1) + (1 << f2);
            board.set_hex(&HexCoord::new(x, y), hex);
        }

        board
    }
    pub fn apply_move(&mut self, mv: &Move) {
        assert!(self.can_apply_move(mv));
        match *mv {
            Move::Move(from, to) => {
                self.set_field(&from, Field::Empty);
                self.set_field(&to, Field::Piece);

                let (capture_count, mut fields_to_check) = self.check_hexes(&from.to_hex());
                fields_to_check.append(&mut self.get_field_edge_neighbors(&to));
                self.check_captures(&fields_to_check);

                self.vitals.get_mut(self.turn).hexes += capture_count;
                self.turn = self.turn.switch();
            }
            Move::Exchange(coord) => {
                self.remove_piece(&coord);
                self.vitals.get_mut(self.turn).hexes -= 2;

                // Players don't collect hexes removed due to an exchange
                let (_, fields_to_check) = self.check_hexes(&coord.to_hex());
                self.check_captures(&fields_to_check);
                self.turn = self.turn.switch();
            }
        }
        self.update_outcome();
    }
    pub fn can_apply_move(&self, mv: &Move) -> bool {
        match *mv {
            Move::Move(from, to) => {
                from.color() == self.turn && self.get_field_vertex_neighbors(&from).contains(&to)
                    && self.is_piece_on_field(&from) && !self.is_piece_on_field(&to)
            }
            Move::Exchange(coord) => {
                self.can_exchange() && coord.color() != self.turn && self.is_piece_on_field(&coord)
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

        for hex in self.extant_hexes() {
            for f in 0..6 {
                let field = hex.to_field(f);
                if self.is_piece_on_field(&field) {
                    if field.color() == turn {
                        moves.append(&mut self.get_field_vertex_neighbors(&field)
                            .into_iter()
                            .filter_map(|to| {
                                if self.is_piece_on_field(&to) {
                                    None
                                } else {
                                    Some(Move::Move(field, to))
                                }
                            })
                            .collect());
                    } else if can_exchange {
                        moves.push(Move::Exchange(field));
                    }
                }
            }
        }
        moves
    }
    pub fn available_moves_for_piece(&self, field: &FieldCoord) -> Vec<FieldCoord> {
        if self.is_piece_on_field(field) {
            self.get_field_vertex_neighbors(field)
                .into_iter()
                .filter(|c| !self.is_piece_on_field(c))
                .collect()
        } else {
            vec![]
        }
    }
    pub fn can_exchange(&self) -> bool {
        self.vitals.get_ref(self.turn).hexes >= 2
    }
    pub fn is_piece_on_field(&self, coord: &FieldCoord) -> bool {
        self.get_field(coord) == Field::Piece
    }
    /// > extant (adj.): Still in existence; not destroyed, lost, or extinct (The Free Dictionary)
    ///
    /// Return the coordinates of the hexes that have not been removed yet.
    pub fn extant_hexes(&self) -> Vec<HexCoord> {
        let mut coords = Vec::with_capacity(19);
        for x in -2..3 {
            for y in -2..3 {
                if let Some(hex) = self.try_hex((x, y)) {
                    coords.push(hex);
                }
            }
        }
        coords
    }
    /// > extant (adj.): Still in existence; not destroyed, lost, or extinct (The Free Dictionary)
    ///
    /// Returns true if a hex has not been removed yet.
    pub fn is_hex_extant(&self, coord: &HexCoord) -> bool {
        self.get_hex(coord) & 0b1_000000 != 0
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
            if wp == 1 && bp == 1 && (self.extant_hexes + cmp::max(wh, bh) - 1 < 2) {
                self.outcome = Outcome::Draw;
            }
        }
    }
    fn can_move(&self) -> bool {
        if self.can_exchange() {
            return true;
        }

        let fields = match self.turn() {
            Color::White => [1, 3, 5],
            Color::Black => [0, 2, 4],
        };

        for hex in self.extant_hexes() {
            for &f in &fields {
                let field = hex.to_field(f);
                if self.is_piece_on_field(&field)
                    && self.get_field_vertex_neighbors(&field)
                        .iter()
                        .any(|to| !self.is_piece_on_field(to))
                {
                    return true;
                }
            }
        }
        false
    }
}

// Field and piece methods
impl Board {
    fn get_field(&self, coord: &FieldCoord) -> Field {
        assert!(
            self.is_hex_extant(&coord.to_hex()),
            "Cannot get field {} on removed hex at {:?}",
            coord.f,
            coord.to_hex()
        );

        if self.get_hex(&coord.to_hex()) & (1 << coord.f) == 0 {
            Field::Empty
        } else {
            Field::Piece
        }
    }
    fn set_field(&mut self, coord: &FieldCoord, field: Field) {
        let f = coord.f;
        let coord = coord.to_hex();

        assert!(
            self.is_hex_extant(&coord),
            "Cannot set field {} on removed hex at {:?}",
            f,
            coord
        );

        let hex = match field {
            Field::Piece => self.get_hex(&coord) | 1 << f,
            Field::Empty => self.get_hex(&coord) & !(1 << f),
        };
        self.set_hex(&coord, hex);
    }
    /// Return fields that share an edge with the given field. These fields are always the opposite
    /// color of the given field. If all of a piece's edge neighbors are occupied, that piece might
    /// be capturable.
    fn get_field_edge_neighbors(&self, coord: &FieldCoord) -> Vec<FieldCoord> {
        let mut neighbors = Vec::with_capacity(3);
        let hex = coord.to_hex();

        // There are always two edge neighbors on the same hex as the given field
        neighbors.push(hex.to_field((coord.f + 1) % 6));
        neighbors.push(hex.to_field((coord.f + 5) % 6));

        if let Some(hex) = self.get_hex_neighbor(&hex, coord.f) {
            neighbors.push(hex.to_field((coord.f + 3) % 6));
        }
        neighbors
    }
    /// Return fields that share a vertex with the given field and have the same color as the given
    /// field. Pieces can move to fields that are vertex neighbors of the field they are on.
    fn get_field_vertex_neighbors(&self, coord: &FieldCoord) -> Vec<FieldCoord> {
        let mut neighbors = Vec::with_capacity(6);
        let hex = coord.to_hex();

        // There are always two vertex neighbors on the same hex as the given field
        neighbors.push(hex.to_field((coord.f + 2) % 6));
        neighbors.push(hex.to_field((coord.f + 4) % 6));

        if let Some(hex) = self.get_hex_neighbor(&hex, coord.f) {
            neighbors.push(hex.to_field((coord.f + 2) % 6));
            neighbors.push(hex.to_field((coord.f + 4) % 6));
        }
        if let Some(hex) = self.get_hex_neighbor(&hex, (coord.f + 1) % 6) {
            neighbors.push(hex.to_field((coord.f + 4) % 6));
        }
        if let Some(hex) = self.get_hex_neighbor(&hex, (coord.f + 5) % 6) {
            neighbors.push(hex.to_field((coord.f + 2) % 6));
        }
        neighbors
    }
    fn remove_piece(&mut self, coord: &FieldCoord) {
        assert!(
            self.is_piece_on_field(coord),
            "There is no piece at {:?} to remove",
            coord
        );
        self.set_field(coord, Field::Empty);
        self.vitals.get_mut(coord.color()).pieces -= 1;
    }
    fn check_captures(&mut self, fields_to_check: &[FieldCoord]) {
        for field in fields_to_check {
            if field.color() != self.turn && self.is_piece_on_field(field)
                && self.get_field_edge_neighbors(field)
                    .iter()
                    .all(|coord| self.is_piece_on_field(coord))
            {
                self.remove_piece(field);
            }
        }
    }
}

// Hex methods
impl Board {
    fn get_hex(&self, coord: &HexCoord) -> u8 {
        let x = (coord.x + 2) as usize;
        let y = (coord.y + 2) as usize;

        self.board[x + y * 5]
    }
    fn set_hex(&mut self, coord: &HexCoord, hex: u8) {
        let x = (coord.x + 2) as usize;
        let y = (coord.y + 2) as usize;

        self.board[x + y * 5] = hex;
    }
    fn is_hex_empty(&self, coord: &HexCoord) -> bool {
        assert!(self.is_hex_extant(coord));
        self.get_hex(coord) & 0b0_111111 == 0
    }
    fn get_hex_neighbor(&self, coord: &HexCoord, direction: u32) -> Option<HexCoord> {
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
        if !self.is_hex_extant(coord) || !self.is_hex_empty(coord) {
            return false;
        }

        // After repeatedly failing to find an efficient and elegant way to check if a hex is
        // removable, I have opted to just use a lookup table. Even if it is not elegant, at
        // least it is simple and efficient.
        let neighbor_combinations = [
            0b000001, 0b000010, 0b000100, 0b001000, 0b010000, 0b100000, 0b000011, 0b000110,
            0b001100, 0b011000, 0b110000, 0b100001, 0b000111, 0b001110, 0b011100, 0b111000,
            0b110001, 0b100011,
        ];

        let mut neighbors = 0;
        for f in 0..6 {
            if self.get_hex_neighbor(coord, f).is_some() {
                neighbors += 1 << f;
            }
        }
        assert!(
            neighbors != 0,
            "A hex at {:?} is empty and has no neighbors",
            coord
        );
        neighbor_combinations.contains(&neighbors)
    }
    fn remove_hex(&mut self, coord: &HexCoord) -> bool {
        let removable = self.is_hex_removable(coord);

        if removable {
            self.set_hex(coord, 0);
            self.extant_hexes -= 1;
        }
        removable
    }
    fn try_hex(&self, coord: (i32, i32)) -> Option<HexCoord> {
        if let Some(coord) = HexCoord::try_new(coord.0, coord.1) {
            if self.is_hex_extant(&coord) {
                return Some(coord);
            }
        }
        None
    }
    fn check_hexes(&mut self, coord: &HexCoord) -> (u8, Vec<FieldCoord>) {
        let mut remove_count = 0;
        let mut fields = vec![];

        if self.remove_hex(coord) {
            remove_count += 1;
            for f in 0..6 {
                if let Some(neighbor) = self.get_hex_neighbor(coord, f) {
                    let (new_remove_count, mut new_fields) = self.check_hexes(&neighbor);
                    if new_remove_count == 0 {
                        fields.push(neighbor.to_field((f + 3) % 6));
                    } else {
                        remove_count += new_remove_count;
                        fields.append(&mut new_fields);
                    }
                }
            }
        }
        (remove_count, fields)
    }
}
