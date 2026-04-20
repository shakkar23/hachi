use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tetris::{
    board::Board,
    piece::{Piece, Rotation},
};
use features::{
    game::{GameState, Move},
    feature_extractor::extract_features,
};

fn make_state() -> GameState {
    GameState {
        board: Board {
            cols: [
                0b000000111111,
                0b000001111111,
                0b000000011111,
                0b000000000111,
                0b000000000001,
                0b000000000000,
                0b000000001101,
                0b000000011111,
                0b000000111111,
                0b000011111111,
            ],
        },
        current_piece: Piece::I,
        placement: Move {
            move_type: None,
            rotation: Rotation::North,
            x: 0,
            y: 0,
        },
        meter: 0,
        combo: 0,
        attack: 0,
        b2b: 0,
        damage_received: 0,
        spun: false,
        queue: [Piece::O, Piece::J, Piece::L, Piece::S, Piece::T],
        hold: None,
    }
}

fn bench_extract_features(c: &mut Criterion) {
    let state = make_state();
    c.bench_function("extract_features", |b| {
        b.iter(|| extract_features(black_box(&state)))
    });
}

criterion_group!(benches, bench_extract_features);
criterion_main!(benches);