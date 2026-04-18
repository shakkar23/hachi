use crate::state::MacroState;
use features::game::GameState;

use bot::{
    bot::{BotConfigs, BotError, BotResult, BotScore, BotState},
    eval::Weights,
};

use tetris::{
    moves::Move,
    piece::Piece,
    state::{Lock, State},
    movegen::movegen,
    bag::Bag,
    board::Board
};

use rand::prelude::*;
use rand::distributions::WeightedIndex;

use features::feature_extractor::Features;
use features::feature_extractor::extract_features;

use crate::eval::eval;
use crate::solver::nash_equilibrium;

pub fn gamestate_to_state(gamestate: &GameState) -> State {
    State {
        board: gamestate.board,
        hold: gamestate.hold,
        bag: Bag::all(),
        next: 0,
        b2b: gamestate.b2b,
        combo: gamestate.combo,
    }
}

fn solve_position(state: &MacroState) -> Move {

    let gamestate1: &GameState = &state.p1;
    let gamestate2: &GameState = &state.p2;

    let board1:Board = state.p1.board;
    let board2:Board = state.p2.board;

    let piece1:Piece = state.p1.current_piece;
    let piece2:Piece = state.p2.current_piece;

    let queue1 = state.p1.queue;
    let queue2 = state.p2.queue;

    let state1:State = gamestate_to_state(&gamestate1);
    let state2:State = gamestate_to_state(&gamestate2);

    let moves1 = get_pruned_moves(&state1, &queue1, 16);
    let moves2 = get_pruned_moves(&state2, &queue2, 16);

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

    let (mut row_strategy, col_strategy, game_value) = nash_equilibrium(&payoffs);

    // remove padding moves

    row_strategy.truncate(moves1.len());

    // execute mixed strategy

    let dist = WeightedIndex::new(&row_strategy).unwrap();
    let mut rng = thread_rng();

    let selected_index = dist.sample(&mut rng);

    moves1[selected_index]
}


fn get_pruned_moves(state: &State, queue: &[Piece; 5], n:usize) -> Vec<Move> {
    sunbeam_top_n(
        state.clone(),
        Lock {
            cleared: 0,
            sent: 0,
            softdrop: false,
        },
        queue,
        Weights::default(),
        BotConfigs {
            width: 250,
            depth: 5, // doesn't do anything
            branch: 1, // doesn't do anything
        },
        5,
        n
    ).unwrap()
}

pub fn sunbeam_top_n(
    root: State,
    lock: Lock,
    full_queue: &[Piece],
    weights: Weights,
    configs: BotConfigs,
    depth: usize,
    n: usize,
) -> Result<Vec<Move>, BotError> {
    // queue length needed to reach `depth`: depth + (1 if no hold).
    let needed = depth + root.hold.is_none() as usize;

    // The bot requires queue.len() >= 2.
    let queue_len = needed.max(2);

    if full_queue.len() < queue_len {
        return Err(BotError::InvalidQueue);
    }

    let queue: Vec<Piece> = full_queue[..queue_len].to_vec();
    let bot = BotState::new(root, lock, queue, weights)?;
    let result = bot.search(configs)?;

    Ok(top_n(&result, n))
}

/// Sort candidates best-first and take the top `n`.
pub fn top_n(result: &BotResult, n: usize) -> Vec<Move> {
    let mut sorted = result.candidates.clone();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.truncate(n);
    sorted.into_iter().map(|(mv, _)| mv).collect()
}