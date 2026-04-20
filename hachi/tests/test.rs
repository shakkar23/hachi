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

fn make_state() -> GameState {
    GameState {
        board: Board {
            cols: [
                0b000000111111,
                0b000011111111,
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

#[test]
fn test_position() {
    let state1 = make_state();
    let state2 = make_state();

    let eval_value = eval(&extract_features(&state1), &extract_features(&state2), ModelType::LightGBM_Large);
    
    println!("eval value: {:#?}", eval_value);

    //let (mv, value) = solve_position(state1, state2, 1, HachiConfig::rapid());
    
    //println!("best move: {:#?}", mv);
    //println!("position value: {}", value);
}