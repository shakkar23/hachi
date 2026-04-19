use crate::state::MacroState;
use features::game::GameState;
use std::cell::RefCell;

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

use crate::eval::{eval_batched, ModelType};
use crate::solver::{nash_equilibrium, nash_equilibrium_exact};
use crate::table::{MoveTable, TableKey, TableValue};

thread_local! {
    static TTABLE: RefCell<MoveTable> = RefCell::new(MoveTable::new());
}

#[derive(Debug, Clone, Copy)]
pub struct HachiConfig {
    pub max_moves: usize,
    pub max_responses: usize,
    pub use_exact: bool,
    pub model_type: ModelType
}

/*
    `max_moves`: Number of moves to consider for playing side.
    `max_responses`: Number of moves to consider for opponent side.
    `use_exact`: `true` to solve equilibriums exactly, or `false` to approximate.
    `model_type`: Model to use for evaluation at leaf positions.

    Smaller values for `max_moves` and `max_responses` result
    in more time spent on beam search. Larger values result in
    more time spent evaluating the payoff model.
*/

impl Default for HachiConfig {
    fn default() -> Self {
        Self {
            max_moves: 8,
            max_responses: 8,
            use_exact: false,
            model_type: ModelType::LightGBM_Large
        }
    }
}

impl HachiConfig {
    fn rapid() -> Self {
        Self {
            max_moves: 4,
            max_responses: 4,
            use_exact: true,
            model_type: ModelType::CatBoost_Small
        }
    }
}


// Solve a two-board position using subgame perfect equilibrium.
// Returns: (best_move, win_probability)
pub fn solve_position(gamestate1: &GameState, gamestate2: &GameState, depth: usize, config: HachiConfig) -> (Move, f64) {

    let queue1 = gamestate1.queue;
    let queue2 = gamestate2.queue;

    let state1:State = gamestate_to_state(&gamestate1);
    let state2:State = gamestate_to_state(&gamestate2);

    let moves1 = get_pruned_moves(&state1, &queue1, config.max_moves);
    let moves2 = get_pruned_moves(&state2, &queue2, config.max_responses);

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

    let mut flat: Vec<f64> = Vec::new();

    if depth == 1 {
        flat = eval_batched(&pairs, config.model_type);
    } else {
        /*
            m + n work using transposition table
            but table remains a hot path
            option: compute moves matrix here and distribute
        */
        for i in 0..m {
            for j in 0..n {
                let (_, subgame_value) = solve_position(
                    &row_states[i],
                    &column_states[j],
                    depth - 1,
                    config
                );
                flat.push(subgame_value);
            }
        }
    }

    // Player 1 perspective
    let mut payoffs: Vec<Vec<f64>> = vec![vec![0.0; n]; m];
    for i in 0..m {
        for j in 0..n {
            payoffs[i][j] = flat[i * n + j];
        }
    }

    let (row_strategy, _, value) = if config.use_exact {
        nash_equilibrium_exact(&payoffs)
    } else {
        nash_equilibrium(&payoffs)
    };

    // execute mixed strategy

    let dist = WeightedIndex::new(&row_strategy).unwrap();
    let mut rng = thread_rng();

    let selected_index = dist.sample(&mut rng);

    (moves1[selected_index], value)
}

// Pruning essential to avoid payoff model going OOD.
pub fn get_pruned_moves(state: &State, queue: &[Piece; 5], n:usize) -> Vec<Move> {
    let key = TableKey { 
        state: state, 
        piece: queue[0] 
    };
    TTABLE.with(|table| {
        if let Some(entry) = table.borrow().get(&key) {
            return entry.moves.clone();
        } else {
            let result = sunbeam_top_n(
                state.clone(),
                Lock {
                    cleared: 0,
                    sent: 0,
                    softdrop: false,
                },
                queue,
                Weights::default(),
                BotConfigs {
                    width: 250
                },
                n
            ).unwrap();
            table.borrow_mut().put(
                &key,
                TableValue{ 
                   moves: result.clone()
                }
            );
            return result;
        }
    })
}

pub fn sunbeam_top_n(
    root: State,
    lock: Lock,
    full_queue: &[Piece],
    weights: Weights,
    configs: BotConfigs,
    n: usize,
) -> Result<Vec<Move>, BotError> {
    let queue: Vec<Piece> = full_queue[..5].to_vec();
    let bot = BotState::new(root, lock, queue, weights)?;
    let result = bot.search_to_n(n, configs)?;

    Ok(top_n(&result, n))
}

/// Sort candidates best-first and take the top `n`.
pub fn top_n(result: &BotResult, n: usize) -> Vec<Move> {
    let mut sorted = result.candidates.clone();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.truncate(n);
    sorted.into_iter().map(|(mv, _)| mv).collect()
}

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