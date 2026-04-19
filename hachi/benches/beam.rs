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

pub fn main() {
    let boards = [
        Board::new(),
        Board {
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
        },
        Board {
            cols: [
                0b000111111111,
                0b000111111111,
                0b000011111111,
                0b000011111111,
                0b000000111111,
                0b000000100110,
                0b000010000001,
                0b000011110111,
                0b000011111111,
                0b000011111111,
            ],
        },
        Board {
            cols: [
                0b000011111111,
                0b000011000000,
                0b110011000000,
                0b110011001100,
                0b110011001100,
                0b110011001100,
                0b110011001100,
                0b110000001100,
                0b110000001100,
                0b111111111100,
            ],
        },
    ];

    const N: usize = 8;

    let mut total_nodes = 0usize;
    let mut total_ms = 0u128;
    let mut total_depth = 0usize;

    for (i, board) in boards.into_iter().enumerate() {
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
            .search_to_n(N, BotConfigs { width: 75 })
            .expect("bot dead!");

        let elapsed = start.elapsed();
        let ms = elapsed.as_millis();

        let unique_roots = {
            let mut idxs: Vec<usize> = result
                .candidates
                .iter()
                .map(|(_, s)| s.depth)
                .collect();
            idxs.sort_unstable();
            idxs.dedup();
            idxs.len()
        };

        println!(
            "board {}: nodes = {:>8}  depth = {:>2}  time = {:>5} ms  candidates = {:>3}  distinct-depths = {}",
            i,
            result.nodes,
            result.depth,
            ms,
            result.candidates.len(),
            unique_roots,
        );

        total_nodes += result.nodes;
        total_ms += ms;
        total_depth += result.depth;
    }

    let knps = if total_ms > 0 {
        total_nodes as u128 / total_ms
    } else {
        0
    };
    let avg_depth = total_depth as f64 / 4.0;
    let avg_ms = total_ms as f64 / 4.0;

    println!("---");
    println!(
        "search_to_n(N={}): total nodes = {}  total time = {} ms  avg time/board = {:.1} ms  avg depth = {:.2}  nps = {} kn/s",
        N, total_nodes, total_ms, avg_ms, avg_depth, knps
    );
}