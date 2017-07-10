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
#[derive(Clone, Copy)]
pub enum Field {
    Piece,
    Empty,
}

impl Board {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    /// Create a new board with the "Laurentius" starting position.
    pub fn new() -> Board {
        let mut board = [[None; 5]; 5];

        // (0, 0) is the only empty hex
        board[2][2] = Some([Field::Empty; 6]);

        // Conveniently, each hex (except for (0, 0)) has exactly two pieces on it in the starting
        // position. The format here is (hex_x, hex_y, field1, field2).
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
            board[(x + 2) as usize][(y + 2) as usize] = Some(hex);
        }

        Board { board }
    }

    pub fn get_field(&self, coord: &FieldCoord) -> &Field {
        match *self.get_hex(&coord.to_hex()) {
            Some(ref hex) => &hex[coord.f as usize],
            None => panic!("Tried to get field on removed hex: {:?}", coord),
        }
    }
    /// Return fields that share an edge with the given field. These fields are always the opposite
    /// color of the given field. If all of a piece's edge neighbors are occupied, that piece might
    /// be capturable.
    pub fn get_field_edge_neighbors(&self, coord: &FieldCoord) -> Vec<FieldCoord> {
        unimplemented!();
    }
    /// Return fields that share a vertex with the given field and have the same color as the given
    /// field. Pieces can move to fields that are vertex neighbors of the field they are on.
    pub fn get_field_vertex_neighbors(&self, coord: &FieldCoord) -> Vec<FieldCoord> {
        unimplemented!();
    }
    pub fn move_piece(&mut self, from: &FieldCoord, to: &FieldCoord) {
        unimplemented!();
    }
    pub fn remove_piece(&mut self, coord: &FieldCoord) {
        unimplemented!();
    }

    fn get_hex(&self, coord: &HexCoord) -> &Option<Hex> {
        &self.board[(coord.x + 2) as usize][(coord.y + 2) as usize]
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
        unimplemented!();
    }
    pub fn remove_hex(&mut self, coord: &HexCoord) {
        unimplemented!();
    }
}

#[derive(Debug)]
pub struct FieldCoord {
    x: i32,
    y: i32,
    f: u32,
}

#[derive(Clone, Copy)]
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
