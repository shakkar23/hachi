use tetris::{
    board::Board,
    piece::{Piece, Rotation},
};
use features::{
    game::{GameState, Move},
};

use hachi::{
    hachi::{solve_position, HachiConfig}
};


use features::feature_extractor::Features;
use features::feature_extractor::extract_features;

use hachi::eval::{eval, eval_batched, ModelType};
use hachi::solver::{nash_equilibrium, nash_equilibrium_exact};

fn make_good_state() -> GameState {
    GameState {
        board: Board {
            cols: [
                0b00000000000110111111,  // col 0
                0b00000000000111111111,  // col 1
                0b00000000000111111111,  // col 2
                0b00000000000111111111,  // col 3
                0b00000000000011111111,  // col 4
                0b00000000000001000001,  // col 5 (the well — only bottom + row 6)
                0b00000000011011111111,  // col 6
                0b00000000011111111111,  // col 7
                0b00000000011111111111,  // col 8
                0b00000000001111111110,  // col 9 (bottom row empty — tetris-ready)
            ],
        },
        current_piece: Piece::T,
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
        queue: [Piece::I, Piece::J, Piece::L, Piece::S, Piece::T],
        hold: None,
    }
}


fn make_bad_state() -> GameState {
    GameState {
        board: Board {
            cols: [
                0b00000000000001100000,  // col 0: rows 5,6 only (the "███████ ██" overhang pair)
                0b00000001111111111111,  // col 1
                0b00011111111111111111,  // col 2 (tallest — 17 rows up to the tip)
                0b00001111111111111111,  // col 3
                0b00000111111111111111,  // col 4
                0b00000011111111111111,  // col 5
                0b00000011111111111111,  // col 6
                0b00000111111110011111,  // col 7 (gap at rows 5,6)
                0b00000111111111111111,  // col 8
                0b00000111111111111111,  // col 9
            ],
        },
        current_piece: Piece::Z,
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

#[test]
fn test_losing_position() {
    let state1 = make_bad_state();
    let state2 = make_good_state();

    let features1 = extract_features(&state1);
    let features2 = extract_features(&state2);

    // should be about 0.3
    let losing_eval = eval(&features1, &features2, ModelType::LightGBM_Large);
    println!("[EVAL] position value (losing): {}", losing_eval);

    // should be about 0.7
    let winning_eval = eval(&features2, &features1, ModelType::LightGBM_Large);
    println!("[EVAL] position value (winning): {}", winning_eval);

    for depth in 1..=4 {
        let (_, losing) = solve_position(state1, state2, depth, HachiConfig::rapid());
        println!("[DEPTH {}] position value (losing): {}", depth, losing);

        let (_, winning) = solve_position(state2, state1, depth, HachiConfig::rapid());
        println!("[DEPTH {}] position value (winning): {}", depth, winning);
    }
}