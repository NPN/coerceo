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

pub mod bitboard;
mod board;
mod constants;
pub mod ttable;
mod zobrist;

use std::cell::RefCell;
use std::fmt;
use std::mem;

use glium::glutin::EventsLoopProxy;

use self::bitboard::BitBoard;
pub use self::board::Board;
use crate::ai::AI;

pub struct Model {
    pub game_type: GameType,
    pub board: Board,
    pub exchange_one_hex: RefCell<bool>,
    pub ply_count: u64,
    pub players: ColorMap<Player>,
    pub selected_piece: Option<FieldCoord>,
    pub last_move: Option<MoveAnnotated>,
    pub exchanging: bool,
    pub ai: AI,
    pub ai_search_depth: RefCell<i32>,
    pub window_states: RefCell<WindowStates>,
    pub outcome: Outcome,
    undo_stack: Vec<(Board, Option<MoveAnnotated>, Outcome)>,
    redo_stack: Vec<(Board, Option<MoveAnnotated>, Outcome)>,
    pub events_proxy: EventsLoopProxy,
}

impl Model {
    pub fn new(
        game_type: GameType,
        players: ColorMap<Player>,
        events_proxy: EventsLoopProxy,
    ) -> Self {
        Self {
            game_type,
            board: Board::new(game_type, 2),
            exchange_one_hex: RefCell::new(false),
            ply_count: 0,
            players,
            selected_piece: None,
            last_move: None,
            exchanging: false,
            ai: AI::new(),
            ai_search_depth: RefCell::new(6),
            window_states: RefCell::new(WindowStates::default()),
            outcome: Outcome::InProgress,
            undo_stack: vec![],
            redo_stack: vec![],
            events_proxy,
        }
    }
    pub fn reset(&mut self, game_type: GameType, players: ColorMap<Player>) {
        self.game_type = game_type;
        self.players = players;

        let exchange_hex_count = if *self.exchange_one_hex.borrow() {
            1
        } else {
            2
        };
        self.board = Board::new(game_type, exchange_hex_count);
        self.ply_count = 0;
        self.selected_piece = None;
        self.last_move = None;
        self.exchanging = false;
        self.ai = AI::new();
        self.outcome = Outcome::InProgress;
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
    pub fn try_move(&mut self, mv: Move) -> bool {
        if self.board.can_apply_move(&mv) {
            self.ply_count += 1;
            self.push_undo_state();
            self.last_move = Some(self.board.annotated_apply_move(&mv));
            self.update_outcome();
            true
        } else {
            false
        }
    }
    pub fn can_undo(&self) -> bool {
        let comp_v_comp =
            self.players.white == Player::Computer && self.players.black == Player::Computer;
        !comp_v_comp && !self.undo_stack.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        let comp_v_comp =
            self.players.white == Player::Computer && self.players.black == Player::Computer;
        !comp_v_comp && !self.redo_stack.is_empty()
    }
    pub fn push_undo_state(&mut self) {
        self.undo_stack
            .push((self.board, self.last_move.clone(), self.outcome));
        self.redo_stack.clear();
    }
    pub fn undo_move(&mut self) {
        while let Some((board, last_move, outcome)) = self.undo_stack.pop() {
            self.redo_stack.push((
                mem::replace(&mut self.board, board),
                mem::replace(&mut self.last_move, last_move),
                mem::replace(&mut self.outcome, outcome),
            ));

            self.clear_selection();
            self.exchanging = false;

            if Player::Human == self.players.get(board.turn) {
                break;
            }
        }
    }
    pub fn redo_move(&mut self) {
        while let Some((board, last_move, outcome)) = self.redo_stack.pop() {
            self.undo_stack.push((
                mem::replace(&mut self.board, board),
                mem::replace(&mut self.last_move, last_move),
                mem::replace(&mut self.outcome, outcome),
            ));

            self.clear_selection();
            self.exchanging = false;

            if Player::Human == self.players.get(board.turn) {
                break;
            }
        }
    }
    pub fn board_list(&self) -> Vec<Board> {
        let mut board_list: Vec<_> = self.undo_stack.iter().map(|ref t| t.0).collect();
        board_list.push(self.board);
        board_list
    }
    pub fn clear_selection(&mut self) {
        self.selected_piece = None;
    }
    pub fn current_player(&self) -> Player {
        self.players.get(self.board.turn)
    }
    pub fn update_outcome(&mut self) {
        if self.outcome == Outcome::InProgress {
            // Only take positions after the last irreversible move
            let board_list: Vec<_> = self
                .board_list()
                .into_iter()
                .rev()
                .skip(1)
                .take_while(|b| b.vitals == self.board.vitals)
                .collect();

            if board_list.len() >= 8 && board_list.iter().filter(|&&b| b == self.board).count() >= 2
            {
                self.outcome = Outcome::DrawThreefoldRepetition;
            } else {
                self.outcome = self.board.outcome();
            }
        }
    }
    pub fn is_game_over(&self) -> bool {
        self.outcome != Outcome::InProgress
    }
    pub fn resign(&mut self) {
        assert_eq!(self.outcome, Outcome::InProgress);
        self.outcome = Outcome::Win(self.board.turn.switch());
    }
}

#[derive(Default)]
pub struct WindowStates {
    pub about: bool,
    pub ai_debug: bool,
    pub how_to_play: bool,
}

#[derive(Copy, Clone)]
pub enum GameType {
    Laurentius,
    Ocius,
}

/// The outcome of a game. This includes being in progress; a win/loss by capturing all of an
/// opponent's pieces; and a draw by stalemate (no legal moves left), insufficient material, or
/// threefold repetition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    InProgress,
    DrawStalemate,
    DrawInsufficientMaterial,
    DrawThreefoldRepetition,
    Win(Color),
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
    pub fn switch(self) -> Self {
        match self {
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

impl<T: Copy> ColorMap<T> {
    pub fn get(&self, color: Color) -> T {
        match color {
            Color::White => self.white,
            Color::Black => self.black,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Move {
    Exchange(BitBoard, Color),
    Move(BitBoard, BitBoard, Color),
}

impl Move {
    pub fn move_from_field(from: FieldCoord, to: FieldCoord) -> Self {
        Move::Move(from.to_bitboard(), to.to_bitboard(), from.color())
    }
    pub fn exchange_from_field(field: FieldCoord) -> Self {
        Move::Exchange(field.to_bitboard(), field.color())
    }
    pub fn annotate(&self, pieces: Vec<FieldCoord>, hexes: Vec<HexCoord>) -> MoveAnnotated {
        MoveAnnotated {
            mv: *self,
            removed_pieces: pieces,
            removed_hexes: hexes,
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Move::Move(from, to, color) => write!(
                f,
                "Move({}, {})",
                FieldCoord::from_bitboard(from, color).to_notation(),
                FieldCoord::from_bitboard(to, color).to_notation(),
            ),
            Move::Exchange(bb, color) => write!(
                f,
                "Exchange({})",
                FieldCoord::from_bitboard(bb, color).to_notation(),
            ),
        }
    }
}

/// A move that also holds the pieces and hexes removed by playing that move. Used by the board to
/// show the effects of the last move.
#[derive(Clone)]
pub struct MoveAnnotated {
    pub mv: Move,
    pub removed_pieces: Vec<FieldCoord>,
    pub removed_hexes: Vec<HexCoord>,
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

        let f = 2 * (index % 3)
            + match color {
                Color::White => 1,
                Color::Black => 0,
            };

        Self::from_hex_f(index / 3, f)
    }
    pub fn from_hex_f(hex: u8, f: u8) -> Self {
        assert!(hex < 19);
        assert!(f < 6);

        let hex = hex as i8
            + match hex {
                0...2 => 2,
                3...15 => 3,
                16...18 => 4,
                _ => unreachable!(),
            };
        Self::new(hex % 5 - 2, hex / 5 - 2, f as u8)
    }
    pub fn to_hex(&self) -> HexCoord {
        HexCoord {
            x: self.x,
            y: self.y,
        }
    }
    pub fn to_bitboard(&self) -> BitBoard {
        let hex = 5 * (self.y + 2) + self.x + 2;
        let hex = hex as u8
            - match hex {
                2...4 => 2,
                6...18 => 3,
                20...22 => 4,
                _ => unreachable!(),
            };

        1 << (hex * 3 + self.f / 2)
    }
    pub fn to_notation(&self) -> String {
        let mut notation = String::with_capacity(3);

        notation.push(match self.x {
            -2 => 'a',
            -1 => 'b',
            0 => 'c',
            1 => 'd',
            2 => 'e',
            _ => unreachable!(),
        });

        let offset = 3 + if self.x < 0 { self.x } else { 0 };
        notation.push(match self.y + offset {
            1 => '1',
            2 => '2',
            3 => '3',
            4 => '4',
            5 => '5',
            _ => unreachable!(),
        });

        notation.push(match self.f {
            5 => 'a',
            4 => 'b',
            3 => 'c',
            2 => 'd',
            1 => 'e',
            0 => 'f',
            _ => unreachable!(),
        });
        notation
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
    pub fn from_index(index: u8) -> Self {
        let hex = index as i8
            + match index {
                0...2 => 2,
                3...15 => 3,
                16...18 => 4,
                _ => unreachable!(),
            };
        Self {
            x: hex % 5 - 2,
            y: hex / 5 - 2,
        }
    }
    pub fn to_index(&self) -> usize {
        let hex = 5 * (self.y + 2) + self.x + 2;
        hex as usize
            - match hex {
                2...4 => 2,
                6...18 => 3,
                20...22 => 4,
                _ => unreachable!(),
            }
    }
    fn is_valid_coord(x: i8, y: i8) -> bool {
        x.abs() <= 2 && y.abs() <= 2 && (x + y).abs() <= 2
    }
}
