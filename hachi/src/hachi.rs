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

use crate::eval::{eval, eval_batched, ModelType};
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
pub struct ChanceState {
    pub gamestate: GameState,
    pub garbage: u32
}

// Solve a two-board position using subgame perfect equilibrium.
// Returns: (best_move, win_probability)
pub fn solve_position(gamestate1: &GameState, gamestate2: &GameState, depth: usize, config: HachiConfig) -> (Move, f64) {

    let queue1 = gamestate1.queue;
    let queue2 = gamestate2.queue;

    let state1:State = gamestate_to_state(&gamestate1);
    let state2:State = gamestate_to_state(&gamestate2);

    // 20ms here
    let moves1 = get_pruned_moves(&state1, &queue1, config.max_moves);
    let moves2 = get_pruned_moves(&state2, &queue2, config.max_responses);

    // row states and column states, we join them later

    let make_state = |mv: &Move, state: &State, queue: &[Piece;5], gamestate: &GameState| {
        let mut s = state.clone();
        let lock = s.make(mv, queue);
        let mut gs = gamestate.clone();
        gs.board = s.board;
        gs.b2b = s.b2b;
        gs.combo = s.combo;

        let next_meter = gs.meter.saturating_sub(lock.sent);
        gs.attack = lock.sent - gs.meter.abs_diff(next_meter);
        gs.meter = next_meter;
        let garbage = if lock.cleared == 0 {
            0
        } else {
            gs.meter as u32
        };

        ChanceState {
            gamestate: gs,
            garbage,
        }
    };

    let row_states: Vec<ChanceState> = moves1
        .iter()
        .map(|mv| make_state(mv, &state1, &queue1, &gamestate1))
        .collect();

    let col_states: Vec<ChanceState> = moves2
        .iter()
        .map(|mv| make_state(mv, &state2, &queue2, &gamestate2))
        .collect();
        
    let row_features: Vec<Option<Features>> = row_states
        .iter()
        .map(|state| {
            if state.garbage == 0 && depth == 1 {
                Some(extract_features(&state.gamestate))
            } else {
                None
            }
        })
        .collect();

    let col_features: Vec<Option<Features>> = col_states
        .iter()
        .map(|state| {
            if state.garbage == 0 && depth == 1 {
                Some(extract_features(&state.gamestate))
            } else {
                None
            }
        })
        .collect();

    let m:usize = moves1.len();
    let n:usize = moves2.len();

    
    /*
        Subgame solving complexity:
        O(((m+n)*beam_search_cost)^depth + (m*n*eval_cost)^depth)
    */
    
    // calculate payoffs

    let mut payoffs: Vec<Vec<f64>> = vec![vec![0.0; n]; m];
    for i in 0..m {
        for j in 0..n {

            // it's impossible for both players to tank garbage at the same time
            // (no passthrough)

            let get_average_payoff = |
                chance_state: &ChanceState, 
                opponent_index: usize, 
                opponent_states: &Vec<ChanceState>,
                opponent_features: &Vec<Option<Features>>| {

                let realized_states: Vec<GameState> = (0..n).map(
                    |k| {
                        let mut gs = chance_state.gamestate.clone();
                        gs.tank_garbage(chance_state.garbage, k);
                        gs
                    }
                ).collect();
                if depth == 1 {
                    // use evaluation payoff
                    realized_states.iter().fold(0.0, |accum, &gamestate| {
                        if let None = opponent_features[opponent_index] {
                            unreachable!();
                        }
                        accum + eval(
                            &extract_features(&gamestate), 
                            opponent_features[opponent_index].as_ref().unwrap(),
                            config.model_type
                        )
                    })
                } else {
                    // use subgame payoff
                    realized_states.iter().fold(0.0, |accum, &gamestate| {
                        let (_, value) = solve_position(
                            &gamestate, 
                            &opponent_states[opponent_index].gamestate,
                            depth - 1,
                            config
                        );
                        accum + value
                    })
                }
            };

            let payoff = if row_states[i].garbage == 0 {
                get_average_payoff(&row_states[i], j, &col_states, &col_features)
            }
            else if col_states[j].garbage == 0{
                get_average_payoff(&col_states[j], i, &row_states, &row_features)
            }
            else {
                eval(row_features[i].as_ref().unwrap(), col_features[j].as_ref().unwrap(), config.model_type)
            };
            
            payoffs[i][j] = payoff;
        }
    }

    // calculate best strategy and win expectation

    let (row_strategy, _, value) = if config.use_exact {
        nash_equilibrium_exact(&payoffs) // 200 microseconds.
    } else {
        nash_equilibrium(&payoffs) // 20 microseconds.
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
            // 10ms per call here
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