use crate::state::MacroState;
use features::game::GameState;

use bot::{
    bot::{BotConfigs, BotError, BotResult, BotState},
    eval::Weights,
};

use tetris::{
    moves::Move,
    piece::Piece,
    state::{Lock, State},
    bag::Bag
};

use rand::prelude::*;
use rand::distributions::WeightedIndex;

use features::feature_extractor::Features;
use features::feature_extractor::extract_features;

use crate::eval::eval_batched;
use crate::solver::nash_equilibrium;

pub fn gamestate_to_state(gamestate: &GameState) -> State {
    State {
        board: gamestate.board,
        hold: gamestate.hold,
        bag: Bag::all(), // gamestate contains no bag information (yet).
        next: 0, // queue always starts at current piece.
        b2b: gamestate.b2b,
        combo: gamestate.combo,
    }
}

fn solve_position(state: &MacroState) -> Move {

    let gamestate1: &GameState = &state.p1;
    let gamestate2: &GameState = &state.p2;

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
            let lock = s.make(mv, &queue1);
            let mut gs = gamestate1.clone();
            gs.board = s.board;
            gs.b2b = s.b2b;
            gs.combo = s.combo;
            gs.attack = lock.sent;
            gs
        })
        .collect();

    let column_states: Vec<GameState> = moves2
        .iter()
        .map(|mv| {
            let mut s = state2.clone();
            let lock = s.make(mv, &queue2);
            let mut gs = gamestate2.clone();
            gs.board = s.board;
            gs.b2b = s.b2b;
            gs.combo = s.combo;
            gs.attack = lock.sent;
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

    let m:usize = moves1.len();
    let n:usize = moves2.len();

    // create batch for eval
    let mut pairs: Vec<(&Features, &Features)> = Vec::with_capacity(m * n);
    for i in 0..m {
        for j in 0..n {
            pairs.push((&row_features[i], &col_features[j]));
        }
    }

    let flat = eval_batched(&pairs);

    // Player 1 perspective
    let mut payoffs: Vec<Vec<f64>> = vec![vec![0.0; n]; m];
    for i in 0..m {
        for j in 0..n {
            payoffs[i][j] = flat[i * n + j];
        }
    }

    let (row_strategy, _, _) = nash_equilibrium(&payoffs);

    // execute mixed strategy

    let dist = WeightedIndex::new(&row_strategy).unwrap();
    let mut rng = thread_rng();

    let selected_index = dist.sample(&mut rng);

    moves1[selected_index]
}

// Pruning essential to avoid payoff model going OOD.
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
            depth: 4, // doesn't do anything yet
            branch: 1, // doesn't do anything yet
        },
        4, // shallow. beam search has diminishing power at depth
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

    assert!(needed <= 5, "depth exceeds queue");

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