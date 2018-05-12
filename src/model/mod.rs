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
mod constants;

pub use self::board::{Board, Outcome};
use ai::AIHandle;

pub struct Model {
    pub board: Board,
    pub players: ColorMap<Player>,
    pub last_move: Option<Move>,
    pub selected_piece: Option<FieldCoord>,
    pub available_moves: Option<Vec<FieldCoord>>,
    pub exchanging: bool,
    pub ai_handle: Option<AIHandle>,
    undo_stack: Vec<(Board, Option<Move>)>,
    redo_stack: Vec<(Board, Option<Move>)>,
}

impl Model {
    pub fn new(players: ColorMap<Player>) -> Self {
        Self {
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
        self.undo_stack.push((self.board, self.last_move));
        self.redo_stack.clear();
    }
    pub fn undo_move(&mut self) {
        if let Some((board, last_move)) = self.undo_stack.pop() {
            self.redo_stack.push((
                mem::replace(&mut self.board, board),
                mem::replace(&mut self.last_move, last_move),
            ));

            self.clear_selection();
            self.exchanging = false;
        }
    }
    pub fn redo_move(&mut self) {
        if let Some((board, last_move)) = self.redo_stack.pop() {
            self.undo_stack.push((
                mem::replace(&mut self.board, board),
                mem::replace(&mut self.last_move, last_move),
            ));

            self.clear_selection();
            self.exchanging = false;
        }
    }
    pub fn clear_selection(&mut self) {
        self.selected_piece = None;
        self.available_moves = None;
    }
    pub fn is_ai_turn(&self) -> bool {
        self.players.get_ref(self.board.turn()) == &Player::Computer
    }
    pub fn is_game_over(&self) -> bool {
        self.board.outcome() != Outcome::InProgress
    }
}

impl Default for Model {
    fn default() -> Self {
        Self {
            board: Board::new(),
            players: ColorMap::new(Player::Human, Player::Human),
            selected_piece: None,
            last_move: None,
            available_moves: None,
            exchanging: false,
            ai_handle: None,
            undo_stack: vec![],
            redo_stack: vec![],
        }
    }
}

pub type BitBoard = u64;

pub fn pop_bit(bb: &mut BitBoard) -> BitBoard {
    // bb & -bb
    let bit = *bb & (1 + !*bb);
    *bb ^= bit;
    bit
}

#[derive(Debug, PartialEq)]
pub enum Player {
    Human,
    Computer,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn switch(&self) -> Self {
        match *self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

/// A map to associate any two values with the variants of the Color enum. Useful for keeping
/// track of player-specific information, which almost always comes in pairs.
#[derive(Clone, Copy, PartialEq)]
pub struct ColorMap<T> {
    pub white: T,
    pub black: T,
}

impl<T> ColorMap<T> {
    pub fn new(white: T, black: T) -> Self {
        Self { white, black }
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

#[derive(Clone, Copy, Debug)]
pub enum Move {
    Exchange(BitBoard, Color),
    Move(BitBoard, BitBoard, Color),
}

impl Move {
    pub fn move_from_field(from: &FieldCoord, to: &FieldCoord) -> Self {
        Move::Move(from.to_bitboard(), to.to_bitboard(), from.color())
    }
    pub fn exchange_from_field(field: &FieldCoord) -> Self {
        Move::Exchange(field.to_bitboard(), field.color())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FieldCoord {
    x: i8,
    y: i8,
    f: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HexCoord {
    x: i8,
    y: i8,
}

impl FieldCoord {
    pub fn new(x: i8, y: i8, f: u8) -> Self {
        assert!(Self::is_valid_coord(x, y, f));
        Self { x, y, f }
    }
    pub fn from_bitboard(bb: BitBoard, color: Color) -> Self {
        Self::from_index(bb.trailing_zeros() as u8, color)
    }
    pub fn from_index(index: u8, color: Color) -> Self {
        assert!(index < 57);

        let f = 2 * (index % 3) + match color {
            Color::White => 1,
            Color::Black => 0,
        };

        Self::from_hex_f(index / 3, f)
    }
    pub fn from_hex_f(hex: u8, f: u8) -> Self {
        assert!(hex < 19);
        assert!(f < 6);

        let hex = hex as i8 + match hex {
            0...2 => 2,
            3...15 => 3,
            16...18 => 4,
            _ => unreachable!(),
        };
        Self::new(hex % 5 - 2, hex / 5 - 2, f as u8)
    }
    pub fn to_index(&self) -> usize {
        self.to_bitboard().trailing_zeros() as usize
    }
    pub fn to_hex(&self) -> HexCoord {
        HexCoord {
            x: self.x,
            y: self.y,
        }
    }
    pub fn to_bitboard(&self) -> BitBoard {
        let hex = 5 * (self.y + 2) + self.x + 2;
        let hex = hex as u8 - match hex {
            2...4 => 2,
            6...18 => 3,
            20...22 => 4,
            _ => unreachable!(),
        };

        1 << (hex * 3 + self.f / 2)
    }
    pub fn f(&self) -> u8 {
        self.f
    }
    pub fn color(&self) -> Color {
        if self.f % 2 == 0 {
            Color::Black
        } else {
            Color::White
        }
    }
    fn is_valid_coord(x: i8, y: i8, f: u8) -> bool {
        (x + y).abs() <= 2 && x.abs() <= 2 && y.abs() <= 2 && f < 6
    }
}

impl HexCoord {
    pub fn try_new(x: i8, y: i8) -> Option<Self> {
        if Self::is_valid_coord(x, y) {
            Some(Self { x, y })
        } else {
            None
        }
    }
    pub fn x(&self) -> i8 {
        self.x
    }
    pub fn y(&self) -> i8 {
        self.y
    }
    pub fn to_field(&self, f: u8) -> FieldCoord {
        assert!(f < 6);
        FieldCoord {
            x: self.x,
            y: self.y,
            f,
        }
    }
    pub fn to_index(&self) -> usize {
        let hex = 5 * (self.y + 2) + self.x + 2;
        hex as usize - match hex {
            2...4 => 2,
            6...18 => 3,
            20...22 => 4,
            _ => unreachable!(),
        }
    }
    fn is_valid_coord(x: i8, y: i8) -> bool {
        (x + y).abs() <= 2 && x.abs() <= 2 && y.abs() <= 2
    }
}
