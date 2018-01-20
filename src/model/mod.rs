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

use std::mem;

mod board;

pub use self::board::Board;

pub struct Model {
    pub board: Board,
    pub players: ColorMap<PlayerType>,
    pub last_move: Option<Move>,
    pub game_result: GameResult,
    pub selected_piece: Option<FieldCoord>,
    pub available_moves: Option<Vec<FieldCoord>>,
    pub exchanging: bool,
    undo_stack: Vec<(Board, Option<Move>, GameResult)>,
    redo_stack: Vec<(Board, Option<Move>, GameResult)>,
}

impl Model {
    pub fn new(players: ColorMap<PlayerType>) -> Model {
        Model {
            players,
            ..Default::default()
        }
    }
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
    pub fn commit_state(&mut self) {
        self.undo_stack
            .push((self.board, self.last_move, self.game_result));
        self.redo_stack.clear();
    }
    pub fn undo_move(&mut self) {
        if let Some((board, last_move, game_result)) = self.undo_stack.pop() {
            self.redo_stack.push((
                mem::replace(&mut self.board, board),
                mem::replace(&mut self.last_move, last_move),
                mem::replace(&mut self.game_result, game_result),
            ));

            self.clear_selection();
            self.exchanging = false;
        }
    }
    pub fn redo_move(&mut self) {
        if let Some((board, last_move, game_result)) = self.redo_stack.pop() {
            self.undo_stack.push((
                mem::replace(&mut self.board, board),
                mem::replace(&mut self.last_move, last_move),
                mem::replace(&mut self.game_result, game_result),
            ));

            self.clear_selection();
            self.exchanging = false;
        }
    }
    pub fn clear_selection(&mut self) {
        self.selected_piece = None;
        self.available_moves = None;
    }
}

impl Default for Model {
    fn default() -> Self {
        Model {
            board: Board::new(),
            players: ColorMap::new(PlayerType::Human, PlayerType::Human),
            selected_piece: None,
            last_move: None,
            available_moves: None,
            exchanging: false,
            game_result: GameResult::InProgress,
            undo_stack: vec![],
            redo_stack: vec![],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn switch(&self) -> Color {
        match *self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PlayerType {
    Human,
    Computer,
}

#[derive(Clone, Copy)]
pub enum Move {
    Exchange(FieldCoord),
    Move(FieldCoord, FieldCoord),
}

#[derive(Clone, Copy, PartialEq)]
pub enum GameResult {
    InProgress,
    Win(Color),
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

/// A map to associate any two values with the variants of the Color enum. Useful for keeping
/// track of player-specific information, which almost always comes in pairs.
#[derive(Clone, Copy, PartialEq)]
pub struct ColorMap<T> {
    pub white: T,
    pub black: T,
}

impl<T> ColorMap<T> {
    pub fn new(white: T, black: T) -> ColorMap<T> {
        ColorMap { white, black }
    }
    pub fn get_ref(&self, color: Color) -> &T {
        match color {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }
    pub fn get_mut(&mut self, color: Color) -> &mut T {
        match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }
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
