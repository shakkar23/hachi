use std::time::Instant;

use bot::{
    bot::{BotConfigs, BotState},
    eval::Weights,
};
use tetris::{
    bag::Bag,
    board::Board,
    piece::Piece,
    state::{Lock, State},
};

#[test]
pub fn test_ranker() {
    let board = Board {
        cols: [
            0b000000111111,
            0b000000111111,
            0b000000011111,
            0b000000000111,
            0b000000000001,
            0b000000000000,
            0b000000001101,
            0b000000011111,
            0b000000111111,
            0b000011111111,
        ],
    };

    let queue = vec![
        Piece::I,
        Piece::O,
        Piece::L,
        Piece::J,
        Piece::S,
    ];

    let bot = BotState::new(
        State {
            board,
            hold: None,
            bag: Bag::all(),
            next: 0,
            b2b: 0,
            combo: 0,
        },
        Lock {
            cleared: 0,
            sent: 0,
            softdrop: false,
        },
        queue,
        Weights::default(),
    )
    .expect("error!");

    let start = Instant::now();
    let result = bot
        .get_full_rankings(BotConfigs { width: 2500 })
        .expect("bot dead!");
    let elapsed = start.elapsed();

    println!(
        "nodes = {}  depth = {}  time = {} ms  candidates = {}",
        result.nodes,
        result.depth,
        elapsed.as_millis(),
        result.candidates.len(),
    );
    println!("---");

    for (i, (mv, score)) in result.candidates.iter().enumerate() {
        println!(
            "{:>3}. score = {:>8}  depth = {:>2}  move = {:?}",
            i + 1,
            score.score,
            score.depth,
            mv,
        );
    }
}