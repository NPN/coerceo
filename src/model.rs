pub struct Model {
    pub board: Board,
    pub turn: Turn,
    pub white_pieces: u32,
    pub white_hexes: u32,
    pub black_pieces: u32,
    pub black_hexes: u32,
    pub selected_piece: Option<FieldCoord>,
}

impl Model {
    pub fn init() -> Model {
        Model {
            board: Board::new(),
            turn: Turn::White,
            white_pieces: 18,
            white_hexes: 0,
            black_pieces: 18,
            black_hexes: 0,
            selected_piece: None,
        }
    }
}

pub enum Turn {
    White,
    Black,
}

pub struct Board {
    // The hex board uses an axial coordinate system with (0, 0) at the center. The x-axis slopes
    // up to the right, and the y-axis goes up and down. The board is stored as a dense 2D array,
    // as a ragged array won't work (since each row would have to be a different type).
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

        // (0, 0) is the only empty hex
        board.set_hex(&HexCoord::new(0, 0), Some([Field::Empty; 6]));

        // Conveniently, every other hex has exactly two pieces on it in the starting position.
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

    pub fn get_field(&self, coord: &FieldCoord) -> &Field {
        match *self.get_hex(&coord.to_hex()) {
            Some(ref hex) => &hex[coord.f as usize],
            None => panic!("Tried to get field on removed hex: {:?}", coord),
        }
    }
    fn set_field(&mut self, coord: &FieldCoord, field: Field) {
        match *self.get_hex(&coord.to_hex()) {
            Some(mut hex) => hex[coord.f as usize] = field,
            None => panic!("Tried to set field on removed hex: {:?}", coord),
        }
    }
    /// Return fields that share an edge with the given field. These fields are always the opposite
    /// color of the given field. If all of a piece's edge neighbors are occupied, that piece might
    /// be capturable.
    pub fn get_field_edge_neighbors(&self, coord: &FieldCoord) -> Vec<FieldCoord> {
        let mut neighbors = vec![
            // There are always two edge neighbors on the same hex as the given field
            FieldCoord {
                f: (coord.f + 1) % 6,
                ..*coord
            },
            FieldCoord {
                f: (coord.f + 5) % 6,
                ..*coord
            },
        ];

        for (i, neighbor) in coord.to_hex().get_neighbors() {
            if coord.f == i && self.get_hex(&neighbor).is_some() {
                let f = (i + 3) % 6;
                neighbors.push(neighbor.to_field(f));
                break;
            }
        }
        neighbors
    }
    /// Return fields that share a vertex with the given field and have the same color as the given
    /// field. Pieces can move to fields that are vertex neighbors of the field they are on.
    pub fn get_field_vertex_neighbors(&self, coord: &FieldCoord) -> Vec<FieldCoord> {
        // A field's vertex neighbors can be defined as the edge neighbors of its edge neighbors
        let mut neighbors = vec![];
        for neighbor in self.get_field_edge_neighbors(coord) {
            neighbors.append(&mut self.get_field_edge_neighbors(&neighbor));
        }
        // A field is not its own neighbor
        neighbors.into_iter().filter(|n| n != coord).collect()
    }
    pub fn move_piece(&mut self, from: &FieldCoord, to: &FieldCoord) {
        assert_eq!(
            self.get_field(from),
            &Field::Piece,
            "Cannot move non-existant piece at {:?}",
            from
        );
        assert!(
            self.get_field_vertex_neighbors(from).contains(to),
            "Cannot move piece at {:?} to non-vertex neighbor {:?}",
            from,
            to
        );
        assert_eq!(
            self.get_field(to),
            &Field::Empty,
            "Cannot move piece at {:?} to occupied field at {:?}",
            from,
            to
        );

        self.set_field(from, Field::Empty);
        self.set_field(to, Field::Piece);
    }
    pub fn remove_piece(&mut self, coord: &FieldCoord) {
        assert_eq!(
            self.get_field(coord),
            &Field::Piece,
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
    pub fn get_hex_neighbors(&self, coord: &HexCoord) -> Vec<HexCoord> {
        coord
            .get_neighbors()
            .iter()
            .filter_map(|&(_, c)| match *self.get_hex(&c) {
                Some(_) => Some(c),
                None => None,
            })
            .collect()
    }
    /// Return fields that share an edge with the given hex and are outside of the given hex. If a
    /// hex is removed from the board, pieces occupying that hex's field neighbors might be
    /// capturable.
    pub fn get_hex_field_neighbors(&self, coord: &HexCoord) -> Vec<FieldCoord> {
        let mut neighbors = vec![];

        for (i, neighbor) in coord.get_neighbors() {
            if self.get_hex(&neighbor).is_some() {
                let f = (i + 3) % 6;
                neighbors.push(neighbor.to_field(f));
            }
        }
        neighbors
    }
    /// A hex is removable (and must be removed) if it is empty and is "attached to the board by 3
    /// or less adjacent sides."
    pub fn is_hex_removable(&self, coord: &HexCoord) -> bool {
        match *self.get_hex(coord) {
            Some(hex) => {
                if hex.iter().any(|&f| match f {
                    Field::Piece => true,
                    Field::Empty => false,
                }) {
                    return false;
                }
            }
            None => {
                panic!(
                    "Can't call Board::is_hex_removable on a removed hex: {:?}",
                    coord
                )
            }
        }

        let neighbor_idxs: Vec<u32> = coord.get_neighbors().iter().map(|&(i, _)| i).collect();
        let neighbor_idxs_slice = neighbor_idxs.as_slice();

        match neighbor_idxs_slice.len() {
            0 => panic!("A hex at {:?} is empty and has no neighbors", coord),
            1 => true,
            2 => {
                let valid_idx_combos = [[0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [0, 5]];
                valid_idx_combos.iter().any(|c| c == neighbor_idxs_slice)
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
                valid_idx_combos.iter().any(|c| c == neighbor_idxs_slice)
            }
            _ => false,
        }
    }
    pub fn remove_hex(&mut self, coord: &HexCoord) {
        assert!(
            self.is_hex_removable(coord),
            "Tried to remove non-removable hex at {:?}",
            coord
        );
        self.set_hex(coord, None);
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
        assert!((x + y).abs() <= 2 && x.abs() <= 2 && y.abs() <= 2 && f < 6);
        FieldCoord { x, y, f }
    }
    pub fn to_hex(&self) -> HexCoord {
        HexCoord::new(self.x, self.y)
    }
}

impl HexCoord {
    pub fn new(x: i32, y: i32) -> HexCoord {
        assert!((x + y).abs() <= 2 && x.abs() <= 2 && y.abs() <= 2);
        HexCoord { x, y }
    }
    pub fn to_field(&self, f: u32) -> FieldCoord {
        FieldCoord::new(self.x, self.y, f)
    }
    // We return a Vec of tuples so that get_hex_field_neighbors and is_hex_removable know which
    // neighbors are on which side of the hex. They need to know this for different reasons:
    //   * get_hex_field_neighbors: the index of each neighboring field depends on which hex
    //                              neighbor that field neighbor is on
    //   * is_hex_removable: a hex is removable only if it is "attached to the board by 3 or less
    //                       adjacent sides"
    fn get_neighbors(&self) -> Vec<(u32, HexCoord)> {
        let mut neighbors = vec![];

        if self.y < 2 && (self.x + self.y) != 2 {
            neighbors.push((0, HexCoord::new(self.x, self.y + 1)));
        }
        if (self.x + self.y) != 2 && self.x < 2 {
            neighbors.push((1, HexCoord::new(self.x + 1, self.y)));
        }
        if self.x < 2 && self.y > -2 {
            neighbors.push((2, HexCoord::new(self.x + 1, self.y - 1)));
        }
        if self.y > -2 && (self.x + self.y) > -2 {
            neighbors.push((3, HexCoord::new(self.x, self.y - 1)));
        }
        if (self.x + self.y) > -2 && self.x > -2 {
            neighbors.push((4, HexCoord::new(self.x - 1, self.y)));
        }
        if self.x > -2 && self.y < 2 {
            neighbors.push((5, HexCoord::new(self.x - 1, self.y + 1)));
        }

        neighbors
    }
}
