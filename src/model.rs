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

pub struct Model {
    pub board: Board,
    pub turn: Color,
    pub white_pieces: u32,
    pub white_hexes: u32,
    pub black_pieces: u32,
    pub black_hexes: u32,
    pub selected_piece: Option<FieldCoord>,
    pub last_move: Option<(FieldCoord, FieldCoord)>,
    pub available_moves: Option<Vec<FieldCoord>>,
}

impl Model {
    pub fn new() -> Model {
        Model {
            board: Board::new(),
            turn: Color::White,
            white_pieces: 18,
            white_hexes: 0,
            black_pieces: 18,
            black_hexes: 0,
            selected_piece: None,
            last_move: None,
            available_moves: None,
        }
    }
    pub fn switch_turns(&mut self) {
        self.turn = match self.turn {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(PartialEq)]
pub enum Color {
    White,
    Black,
}

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
        up to the right, and the y-axis goes up and down. The board is stored as a dense 2D array,
        because a ragged array won't work (since each row would have to be a different type).

        See http://www.redblobgames.com/grids/hexagons/#coordinates-axial for more info.
    */
    board: [[Option<Hex>; 5]; 5],
}

// Fields are numbered clockwise from the top. Even indicies are black, odd indicies are white.
type Hex = [Field; 6];

// There's no need to store the color of the piece on this field, since only white pieces can be on
// white fields, and vice versa.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Field {
    Piece,
    Empty,
}

impl Board {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    /// Create a new board with the "Laurentius" starting position.
    pub fn new() -> Board {
        let mut board = Board {
            board: [[None; 5]; 5]
        };

        // (0, 0) is the only empty hex.
        board.set_hex(&HexCoord::new(0, 0), Some([Field::Empty; 6]));

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
            let mut hex = [Field::Empty; 6];
            hex[f1] = Field::Piece;
            hex[f2] = Field::Piece;
            board.set_hex(&HexCoord::new(x, y), Some(hex));
        }

        board
    }

    fn get_field(&self, coord: &FieldCoord) -> &Field {
        match *self.get_hex(&coord.to_hex()) {
            Some(ref hex) => &hex[coord.f as usize],
            None => panic!(
                "Cannot get field {} on removed hex at {:?}",
                coord.f,
                coord.to_hex()
            ),
        }
    }
    fn set_field(&mut self, coord: &FieldCoord, field: Field) {
        match *self.get_hex(&coord.to_hex()) {
            Some(mut hex) => {
                hex[coord.f as usize] = field;
                self.set_hex(&coord.to_hex(), Some(hex));
            }
            None => panic!(
                "Cannot set field {} on removed hex at {:?}",
                coord.f,
                coord.to_hex()
            ),
        }
    }
    /// Return fields that share an edge with the given field. These fields are always the opposite
    /// color of the given field. If all of a piece's edge neighbors are occupied, that piece might
    /// be capturable.
    pub fn get_field_edge_neighbors(&self, coord: &FieldCoord) -> Vec<FieldCoord> {
        let mut neighbors = vec![
            // There are always two edge neighbors on the same hex as the given field
            FieldCoord::new(coord.x, coord.y, (coord.f + 1) % 6),
            FieldCoord::new(coord.x, coord.y, (coord.f + 5) % 6),
        ];

        if let Some(hex) = self.get_hex_neighbor(&coord.to_hex(), coord.f) {
            neighbors.push(hex.to_field((coord.f + 3) % 6));
        }

        neighbors
    }
    /// Return fields that share a vertex with the given field and have the same color as the given
    /// field. Pieces can move to fields that are vertex neighbors of the field they are on.
    pub fn get_field_vertex_neighbors(&self, coord: &FieldCoord) -> Vec<FieldCoord> {
        // A field's vertex neighbors can be defined as the edge neighbors of its edge neighbors
        let mut neighbors = vec![];
        for field in self.get_field_edge_neighbors(coord) {
            neighbors.append(&mut self.get_field_edge_neighbors(&field));
        }
        // A field is not its own neighbor
        neighbors.into_iter().filter(|n| n != coord).collect()
    }
    pub fn is_piece_on_field(&self, coord: &FieldCoord) -> bool {
        self.get_field(coord) == &Field::Piece
    }
    pub fn get_available_moves(&self, field: &FieldCoord) -> Vec<FieldCoord> {
        if self.is_piece_on_field(field) {
            self.get_field_vertex_neighbors(field)
                .into_iter()
                .filter(|c| !self.is_piece_on_field(c))
                .collect()
        } else {
            vec![]
        }
    }
    pub fn can_move_piece(&self, from: &FieldCoord, to: &FieldCoord) -> bool {
        self.is_piece_on_field(from) && !self.is_piece_on_field(to)
            && self.get_field_vertex_neighbors(from).contains(to)
    }
    pub fn move_piece(&mut self, from: &FieldCoord, to: &FieldCoord) {
        assert!(
            self.can_move_piece(from, to),
            "Can't move {:?} at {:?} to {:?} at {:?}. These fields {} vertex neighbors.",
            self.get_field(from),
            from,
            self.get_field(to),
            to,
            if self.get_field_vertex_neighbors(from).contains(to) {
                "ARE"
            } else {
                "ARE NOT"
            },
        );

        self.set_field(from, Field::Empty);
        self.set_field(to, Field::Piece);
    }
    pub fn remove_piece(&mut self, coord: &FieldCoord) {
        assert!(
            self.is_piece_on_field(coord),
            "There is no piece at {:?} to remove",
            coord
        );
        self.set_field(coord, Field::Empty);
    }
    fn get_hex(&self, coord: &HexCoord) -> &Option<Hex> {
        &self.board[(coord.x + 2) as usize][(coord.y + 2) as usize]
    }
    fn set_hex(&mut self, coord: &HexCoord, hex: Option<Hex>) {
        self.board[(coord.x + 2) as usize][(coord.y + 2) as usize] = hex;
    }
    pub fn get_hex_neighbor(&self, coord: &HexCoord, direction: u32) -> Option<HexCoord> {
        assert!(direction < 6);

        let neighbors = [
            (coord.x, coord.y + 1),
            (coord.x + 1, coord.y),
            (coord.x + 1, coord.y - 1),
            (coord.x, coord.y - 1),
            (coord.x - 1, coord.y),
            (coord.x - 1, coord.y + 1),
        ];

        let (x, y) = neighbors[direction as usize];
        self.try_hex(x, y)
    }
    /// A hex is removable (and must be removed) if it is empty and is "attached to the board by 3
    /// or less adjacent sides."
    pub fn is_hex_removable(&self, coord: &HexCoord) -> bool {
        match *self.get_hex(coord) {
            Some(hex) => if hex != [Field::Empty; 6] {
                return false;
            },
            None => return false,
        }

        let neighbor_indexes: Vec<u32> = vec![0, 1, 2, 3, 4, 5]
            .into_iter()
            .filter(|&f| self.get_hex_neighbor(coord, f).is_some())
            .collect();
        let neighbor_indexes = neighbor_indexes.as_slice();

        match neighbor_indexes.len() {
            0 => panic!("A hex at {:?} is empty and has no neighbors", coord),
            1 => true,
            2 => {
                let valid_idx_combos = [[0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [0, 5]];
                valid_idx_combos.iter().any(|c| c == neighbor_indexes)
            }
            3 => {
                let valid_idx_combos = [
                    [0, 1, 2],
                    [1, 2, 3],
                    [2, 3, 4],
                    [3, 4, 5],
                    [0, 4, 5],
                    [0, 1, 5],
                ];
                valid_idx_combos.iter().any(|c| c == neighbor_indexes)
            }
            _ => false,
        }
    }
    pub fn remove_hex(&mut self, coord: &HexCoord) {
        assert!(
            self.is_hex_removable(coord),
            "Cannot remove the hex at {:?} which is {:?} and has {} neighbors",
            coord,
            self.get_hex(coord),
            [0, 1, 2, 3, 4, 5]
                .into_iter()
                .filter(|&&f| self.get_hex_neighbor(coord, f).is_some())
                .collect::<Vec<_>>()
                .len(),
        );
        self.set_hex(coord, None);
    }
    fn try_hex(&self, x: i32, y: i32) -> Option<HexCoord> {
        if let Some(coord) = HexCoord::try_new(x, y) {
            if self.is_hex_extant(&coord) {
                return Some(coord);
            }
        }
        None
    }
    /// > extant (adj.): Still in existence; not destroyed, lost, or extinct (The Free Dictionary)
    ///
    /// Return true if the passed hex has not been removed yet.
    pub fn is_hex_extant(&self, coord: &HexCoord) -> bool {
        self.get_hex(coord).is_some()
    }
    /// > extant (adj.): Still in existence; not destroyed, lost, or extinct (The Free Dictionary)
    ///
    /// Return the coordinates of the hexes that have not been removed yet.
    pub fn extant_hexes(&self) -> Vec<HexCoord> {
        let mut coords = vec![];

        for x in -2..3 {
            for y in -2..3 {
                if let Some(hex) = self.try_hex(x, y) {
                    coords.push(hex);
                }
            }
        }
        coords
    }
}

#[derive(Debug, PartialEq)]
pub struct FieldCoord {
    x: i32,
    y: i32,
    f: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct HexCoord {
    x: i32,
    y: i32,
}

impl FieldCoord {
    pub fn new(x: i32, y: i32, f: u32) -> FieldCoord {
        assert!(Self::is_valid_coord(x, y, f));
        FieldCoord { x, y, f }
    }
    pub fn f(&self) -> u32 {
        self.f
    }
    pub fn to_hex(&self) -> HexCoord {
        HexCoord::new(self.x, self.y)
    }
    fn is_valid_coord(x: i32, y: i32, f: u32) -> bool {
        (x + y).abs() <= 2 && x.abs() <= 2 && y.abs() <= 2 && f < 6
    }
    pub fn color(&self) -> Color {
        if self.f % 2 == 0 {
            Color::Black
        } else {
            Color::White
        }
    }
}

impl HexCoord {
    pub fn new(x: i32, y: i32) -> HexCoord {
        assert!(Self::is_valid_coord(x, y));
        HexCoord { x, y }
    }
    pub fn try_new(x: i32, y: i32) -> Option<HexCoord> {
        if Self::is_valid_coord(x, y) {
            Some(HexCoord { x, y })
        } else {
            None
        }
    }
    pub fn x(&self) -> i32 {
        self.x
    }
    pub fn y(&self) -> i32 {
        self.y
    }
    pub fn to_field(&self, f: u32) -> FieldCoord {
        FieldCoord::new(self.x, self.y, f)
    }
    fn is_valid_coord(x: i32, y: i32) -> bool {
        (x + y).abs() <= 2 && x.abs() <= 2 && y.abs() <= 2
    }
}
