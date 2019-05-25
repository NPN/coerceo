#[macro_use]
extern crate criterion;

use criterion::{black_box, Criterion};

use coerceo::model::{Board, GameType};

fn perft(board: &Board, depth: u8) -> u64 {
    if depth == 0 {
        1
    } else {
        let mut sum = 0;
        for mv in board.generate_moves() {
            let mut new_board = *board;
            new_board.apply_move(&mv);
            sum += perft(&new_board, depth - 1);
        }
        sum
    }
}

fn laurentius_perft_1(c: &mut Criterion) {
    c.bench_function("laurentius perft 1", |b| {
        let board = Board::new(GameType::Laurentius, 2);
        b.iter(|| {
            perft(&board, black_box(1));
        });
    });
}

fn laurentius_perft_2(c: &mut Criterion) {
    c.bench_function("laurentius perft 2", |b| {
        let board = Board::new(GameType::Laurentius, 2);
        b.iter(|| {
            perft(&board, black_box(2));
        });
    });
}

fn laurentius_perft_3(c: &mut Criterion) {
    c.bench_function("laurentius perft 3", |b| {
        let board = Board::new(GameType::Laurentius, 2);
        b.iter(|| {
            perft(&board, black_box(3));
        });
    });
}

criterion_group!(
    laurentius_perft,
    laurentius_perft_1,
    laurentius_perft_2,
    laurentius_perft_3
);
criterion_main!(laurentius_perft);
