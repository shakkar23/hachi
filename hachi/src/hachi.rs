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
    // if compute constrained, try this
    fn rapid() -> Self {
        Self {
            max_moves: 4,
            max_responses: 4,
            use_exact: false,
            model_type: ModelType::CatBoost_Small
        }
    }
}

#[derive(PartialEq, Eq)]
enum StateState {
    One(GameState),
    Ten([GameState;10])
}

// Solve a two-board position using subgame perfect equilibrium.
// Returns: (best_move, win_probability)
pub fn solve_position(gamestate1: &GameState, gamestate2: &GameState, depth: usize, config: HachiConfig) -> (Move, f64) {

    let queue1 = gamestate1.queue;
    let queue2 = gamestate2.queue;

    let state1:State = gamestate_to_state(&gamestate1);
    let state2:State = gamestate_to_state(&gamestate2);

    // awyzza: 20ms here
    let moves1 = get_pruned_moves(&state1, &queue1, config.max_moves);
    let moves2 = get_pruned_moves(&state2, &queue2, config.max_responses);

    // row states and column states, we join them later

    let row_states: Vec<StateState> = moves1
        .iter()
        .map(|mv| {
            let mut s = state1.clone();
            let lock = s.make(mv, &queue1);
            let mut gs = gamestate1.clone();
            gs.board = s.board;
            gs.b2b = s.b2b;
            gs.combo = s.combo;

            let next_meter = gs.meter.saturating_sub(lock.sent);
            gs.attack = lock.sent - gs.meter.abs_diff(next_meter);
            gs.meter = next_meter;
            if lock.cleared == 0 {
                let next_states = gs.tank_garbage(gs.meter as u32);
                if let Some(actual_next_states) = next_states {
                    return StateState::Ten(actual_next_states);
                }
            }
            StateState::One(gs)
        })
        .collect();
    let column_states: Vec<StateState> = moves2
        .iter()
        .map(|mv| {
            let mut s = state2.clone();
            let lock = s.make(mv, &queue2);
            let mut gs = gamestate2.clone();
            gs.board = s.board;
            gs.b2b = s.b2b;
            gs.combo = s.combo;

            let next_meter = gs.meter.saturating_sub(lock.sent);
            gs.attack = lock.sent - gs.meter.abs_diff(next_meter);
            gs.meter = next_meter;
            if lock.cleared == 0 {
                let next_states = gs.tank_garbage(gs.meter as u32);
                if let Some(actual_next_states) = next_states {
                    return StateState::Ten(actual_next_states);
                }
            }
            StateState::One(gs)
        })
        .collect();

    // calculate features along axes once and join later
    
    let row_features: Vec<Vec<Features>> = row_states
        .iter()
        .map(|state| {
            if let StateState::One(one) = state {
                return vec![extract_features(one)];
            }
            if let StateState::Ten(ten) = state {
                return ten.clone().map(|one_of_ten|{extract_features(&one_of_ten)}).to_vec();
            }
            unreachable!();
        })
        .collect();

    let col_features: Vec<Vec<Features>> = column_states
        .iter()
        .map(|state| {
            if let StateState::One(one) = state {
                return vec![extract_features(one)];
            }
            if let StateState::Ten(ten) = state {
                return ten.clone().map(|one_of_ten|{extract_features(&one_of_ten)}).to_vec();
            }
            unreachable!();
        })
        .collect();

    let m:usize = moves1.len();
    let n:usize = moves2.len();

    // the grid looks like this
    // -----------------------
    // |          |----------|
    // |   one    |----------|
    // |  state   |----------|
    // |          |----------|
    // -----------------------
    // |  | | | | |  | | | | |
    // |  | | | | |  | | | | |
    // |  | | | | |----------|
    // |  | | | | |  | | | | |
    // -----------------------
    let mut move_payoffs:Vec<Vec<f64>> = Vec::with_capacity(m);

    for i in 0..m {
        move_payoffs.push(Vec::with_capacity(n));
        for j in 0..n {
            // calculate one payoff
            // averages the move_payoffs evenly because garbage is evenly dispursed, and theres no action after this point
            let m2 = row_features[n].len();
            let mut pairs:Vec<(&Features, &Features)>= Vec::with_capacity(10*10);
            for ii in 0..m2 {
                let n2 = col_features[n].len();
                for jj in 0..n2 {
                    pairs.push((&row_features[i][ii], &col_features[j][jj]));
                }
            }
            let flat = eval_batched(&pairs, config.model_type);
            let sum: f64 = flat.iter().sum();
            let count = flat.len() as f64;

            let average = if count > 0.0 { sum / count } else { 0.0 };
            move_payoffs[i].push(average);
        }
    }
    
    // create batch for eval
    // let mut pairs: Vec<(&Features, &Features)> = Vec::with_capacity(m * n);
    // for i in 0..m {
    //     for j in 0..n {
    //         pairs.push((&row_features[i], &col_features[j]));
    //     }
    // }

    //let flat = eval_batched(&pairs);

    //let mut flat: Vec<f64> = Vec::new();

    if depth == 1 {
        // awyzza: 100 microseconds using catboost small
        // awyzza: 2 ms using lightgbm large
        // awyzza: but note this runs 64 times at depth 2
        // awyzza: so it's still like 100-200ms total
        // awyzza: or 5-10ms using cb_s
        //flat = eval_batched(&pairs, config.model_type);
    } else {
        /*
            O((m+n)*beam_search_cost + m*n*eval_cost)
            in principle, row/column siblings have the same
            movesets which they read from the table.
            note: table hot path could be bypassed altogether
            by computing values here and then passing them into
            each `solve_position` call. may reduce overhead
        */
        // awyzza: this loop costs about 200ms at depth = 2
        // awyzza: also, cousins can share moves too.
        // awyzza: you would still need the table for that.
        let mut flat:Vec<f64> = Vec::new();
        for i in 0..m {
            let mut subgame_values = Vec::new();
            for j in 0..n {
                let m_is_one = matches!(row_states[i], StateState::One(_));
                let m2 = if m_is_one {1} else {10};
                for ii in 0..m2 {
                    let n_is_one = matches!(column_states[j], StateState::One(_));

                    let m1_state = match row_states[i] {
                            StateState::One(s) => s,
                            StateState::Ten(t) => t[ii]
                        };

                    let n2 = if m_is_one {1} else {10};
                    for jj in 0..n2 {
                        
                        let n1_state = match column_states[j] {
                                StateState::One(s) => s,
                                StateState::Ten(t) => t[jj]
                            };
                        let (_, subgame_value) = solve_position(
                            &m1_state,
                            &n1_state,
                            depth - 1,
                            config
                        );
                        subgame_values.push(subgame_value);
                    }
                }
                flat.push(subgame_values.iter().sum::<f64>() / subgame_values.len() as f64);
            }
        }
    }

    // Player 1 perspective
    // let mut payoffs: Vec<Vec<f64>> = vec![vec![0.0; n]; m];
    // for i in 0..m {
    //     for j in 0..n {
    //         payoffs[i][j] = flat[i * n + j];
    //     }
    // }

    // calculate best strategy and win expectation

    let (row_strategy, _, value) = if config.use_exact {
        nash_equilibrium_exact(&move_payoffs) // awyzza: 200 microseconds.
    } else {
        nash_equilibrium(&move_payoffs) // awyzza: 20 microseconds.
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
            // awyzza: 10ms per call here
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
                    width: 75
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
        bag: Bag::all(), // doesn't matter because queue is always full.
        next: 0, // queue always starts at current piece.
        b2b: gamestate.b2b,
        combo: gamestate.combo,
    }
}