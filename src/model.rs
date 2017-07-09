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
        unimplemented!();
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
pub enum Field {
    Piece,
    Empty,
}

impl Board {
    pub fn get_field(&self, coord: &FieldCoord) -> &Field {
        unimplemented!();
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
        unimplemented!();
    }
    pub fn get_hex_neighbors(&self, coord: &HexCoord) -> Vec<HexCoord> {
        unimplemented!();
    }
    /// Return fields that share an edge with the given hex and are outside of the given hex. If a
    /// hex is removed from the board, pieces occupying that hex's field neighbors might be
    /// capturable.
    pub fn get_hex_field_neighbors(&self, coord: &HexCoord) -> Vec<FieldCoord> {
        unimplemented!();
    }
    pub fn is_hex_removable(&self, coord: &HexCoord) -> bool {
        unimplemented!();
    }
    pub fn remove_hex(&mut self, coord: &HexCoord) {
        unimplemented!();
    }
}

pub struct FieldCoord {
    x: i32,
    y: i32,
    f: u32,
}

pub struct HexCoord {
    x: i32,
    y: i32,
}

impl FieldCoord {
    pub fn new(x: i32, y: i32, f: u32) -> FieldCoord {
        unimplemented!();
    }
    pub fn to_hex(&self) -> HexCoord {
        unimplemented!();
    }
}

impl HexCoord {
    pub fn new(x: i32, y: i32) -> HexCoord {
        unimplemented!();
    }
    pub fn to_field(&self, f: u32) -> FieldCoord {
        unimplemented!();
    }
    // We return an array of Options instead of a Vec so that get_hex_field_neighbors and
    // is_hex_removable know which neighbors are on which side of the hex. They need to know this
    // for different reasons:
    //   * get_hex_field_neighbors: the index of each neighboring field depends on which hex
    //                              neighbor that field neighbor is on
    //   * is_hex_removable: a hex is removable if it is attached to the board by 3 or less
    //                       *adjacent* sides
    fn get_neighbors(&self) -> [Option<HexCoord>; 6] {
        unimplemented!();
    }
}
