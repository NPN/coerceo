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

mod board;

pub use self::board::Board;

pub struct Model {
    pub board: Board,
    pub turn: Color,
    pub white_hexes: u32,
    pub black_hexes: u32,
    pub selected_piece: Option<FieldCoord>,
    pub last_move: Move,
    pub available_moves: Option<Vec<FieldCoord>>,
    pub exchanging: bool,
    pub game_result: GameResult,
}

impl Model {
    pub fn new() -> Model {
        Model {
            board: Board::new(),
            turn: Color::White,
            white_hexes: 0,
            black_hexes: 0,
            selected_piece: None,
            last_move: Move::None,
            available_moves: None,
            exchanging: false,
            game_result: GameResult::InProgress,
        }
    }
    pub fn switch_turns(&mut self) {
        self.turn = match self.turn {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
    pub fn can_exchange(&self) -> bool {
        2 <= match self.turn {
            Color::Black => self.black_hexes,
            Color::White => self.white_hexes,
        }
    }
}

#[derive(PartialEq)]
pub enum Color {
    White,
    Black,
}

pub enum Move {
    Exchange(FieldCoord),
    Move(FieldCoord, FieldCoord),
    None,
}

#[derive(PartialEq)]
pub enum GameResult {
    InProgress,
    WhiteWin,
    BlackWin,
}

#[derive(Debug, PartialEq)]
pub struct FieldCoord {
    x: i32,
    y: i32,
    f: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
