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
    pub last_move: Move,
    pub turn: Color,
    pub game_result: GameResult,
    pub selected_piece: Option<FieldCoord>,
    pub available_moves: Option<Vec<FieldCoord>>,
    pub exchanging: bool,
    undo_stack: Vec<(Board, Move, Color, GameResult)>,
    redo_stack: Vec<(Board, Move, Color, GameResult)>,
}

impl Model {
    pub fn new() -> Model {
        Model {
            board: Board::new(),
            turn: Color::White,
            selected_piece: None,
            last_move: Move::None,
            available_moves: None,
            exchanging: false,
            game_result: GameResult::InProgress,
            undo_stack: vec![],
            redo_stack: vec![],
        }
    }
    pub fn switch_turns(&mut self) {
        self.turn = self.turn.switch();
    }
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
    pub fn commit_move(&mut self) {
        self.undo_stack
            .push((self.board, self.last_move, self.turn, self.game_result));
        self.redo_stack.clear();
    }
    pub fn undo_move(&mut self) {
        if let Some((board, last_move, turn, game_result)) = self.undo_stack.pop() {
            self.redo_stack.push((
                mem::replace(&mut self.board, board),
                mem::replace(&mut self.last_move, last_move),
                mem::replace(&mut self.turn, turn),
                mem::replace(&mut self.game_result, game_result),
            ));

            self.clear_selection();
            self.exchanging = false;
        }
    }
    pub fn redo_move(&mut self) {
        if let Some((board, last_move, turn, game_result)) = self.redo_stack.pop() {
            self.undo_stack.push((
                mem::replace(&mut self.board, board),
                mem::replace(&mut self.last_move, last_move),
                mem::replace(&mut self.turn, turn),
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

#[derive(Clone, Copy, PartialEq)]
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

#[derive(Clone, Copy)]
pub enum Move {
    Exchange(FieldCoord),
    Move(FieldCoord, FieldCoord),
    None,
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
