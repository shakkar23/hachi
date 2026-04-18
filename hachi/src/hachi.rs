use crate::state::MacroState;
use features::game::GameState;
use tetris::board::Board;
use tetris::state::State;
use tetris::moves::Move;
use tetris::piece::Piece;
use tetris::bag::Bag;
use tetris::movegen::movegen;

use rand::prelude::*;
use rand::distributions::WeightedIndex;

use features::feature_extractor::Features;
use features::feature_extractor::extract_features;

use crate::eval::eval;
use crate::solver::nash_equilibrium;

fn solve_position(state: &MacroState) -> Move {

    let mut gamestate1: GameState = state.p1.clone();
    let mut gamestate2: GameState = state.p2.clone();

    let board1:Board = state.p1.board;
    let board2:Board = state.p2.board;

    let piece1:Piece = state.p1.current_piece;
    let piece2:Piece = state.p2.current_piece;

    let queue1 = state.p1.queue;
    let queue2 = state.p2.queue;


    let state1:State = State {
        board: board1,
        hold: gamestate1.hold,
        bag: Bag::all(),
        next: 0,
        b2b: gamestate1.b2b,
        combo: gamestate1.combo,
    };

    let state2:State = State {
        board: board2,
        hold: gamestate2.hold,
        bag: Bag::all(),
        next: 0,
        b2b: gamestate2.b2b,
        combo: gamestate2.combo,
    };

    let moves1 = movegen(&board1, piece1);
    let moves2 = movegen(&board2, piece2);

    // advance queues
    // puts the current piece at the end of the queue
    // no real reason. it's just faster than speculating or putting in a random piece

    gamestate1.queue.rotate_left(1);
    gamestate2.queue.rotate_left(1);

    // row states and column states, we join them later

    let row_states: Vec<GameState> = moves1
        .iter()
        .map(|mv| {
            let mut s = state1.clone();
            s.make(mv, &queue1);
            let mut gs = gamestate1.clone();
            gs.board = s.board;
            gs.b2b = s.b2b;
            gs.combo = s.combo;
            gs
        })
        .collect();

    let column_states: Vec<GameState> = moves2
        .iter()
        .map(|mv| {
            let mut s = state2.clone();
            s.make(mv, &queue2);
            let mut gs = gamestate2.clone();
            gs.board = s.board;
            gs.b2b = s.b2b;
            gs.combo = s.combo;
            gs
        })
        .collect();

    // calculate features along axes once and join later

    let row_features: Vec<Features> = row_states
        .iter()
        .map(|state| {
            extract_features(state)
        })
        .collect();

    let col_features: Vec<Features> = column_states
        .iter()
        .map(|state| {
            extract_features(state)
        })
        .collect();

    let n:usize = moves1.len().max(moves2.len());

    // Player 1 perspective
    let mut payoffs: Vec<Vec<f64>> = vec![vec![0.0; n]; n];

    for i in 0..n {
        for j in 0..n {
            if i >= moves1.len() {
                // Player 1 has no move
                payoffs[i][j] = -1.0;
                continue;
            }
            if j >= moves2.len() {
                // Player 2 has no move
                payoffs[i][j] = 1.0;
                continue;
            }

            payoffs[i][j] = eval(&row_features[i], &col_features[j]);
        }
    }

    let (row_strategy, col_strategy, game_value) = nash_equilibrium_exact(&payoffs);

    // execute mixed strategy

    let dist = WeightedIndex::new(&row_strategy).unwrap();
    let mut rng = thread_rng();

    let selected_index = dist.sample(&mut rng);

    moves1[selected_index]
}
