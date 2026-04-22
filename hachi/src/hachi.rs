use features::game::GameState;
use std::cell::RefCell;

use bot::{
    bot::{BotConfigs, BotError, BotResult, BotState},
    eval::Weights,
};

use tetris::{
    moves::Move,
    piece::{Piece, Rotation},
    state::{Lock, State},
    bag::Bag
};

use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

use rand::prelude::*;
use rand::distributions::WeightedIndex;

use features::feature_extractor::Features;
use features::feature_extractor::extract_features;

use crate::eval::{eval, eval_batched, ModelType};
use crate::solver::{nash_equilibrium, nash_equilibrium_exact};
use crate::table::{MoveTable, TableKey, TableValue};

thread_local! {
    static TTABLE: RefCell<MoveTable> = RefCell::new(MoveTable::new());
    static XORSHIFT_STATE: RefCell<u64> = RefCell::new(0x9E3779B97F4A7C15);
    static TREE_DUMP: RefCell<TreeDump> = RefCell::new(TreeDump::new());
}

fn xorshift64() -> u64 {
    XORSHIFT_STATE.with(|s| {
        let mut x = *s.borrow();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *s.borrow_mut() = x;
        x
    })
}

#[derive(Serialize, Clone)]
pub struct TreeNode {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub depth: usize,
    pub gamestate_row: String,
    pub gamestate_col: String,
    pub value: f64
}

pub struct TreeDump {
    pub nodes: Vec<TreeNode>,
}

impl TreeDump {
    fn new() -> Self { Self { nodes: vec![] } }
}

pub fn drain_tree_dump() -> String {
    TREE_DUMP.with(|d| {
        let nodes = std::mem::take(&mut d.borrow_mut().nodes);
        serde_json::to_string_pretty(&nodes).unwrap()
    })
}
pub fn reset_tree_dump() {
    TREE_DUMP.with(|d| *d.borrow_mut() = TreeDump::new());
    NODE_COUNTER.store(0, Ordering::Relaxed);
}

#[derive(Debug, Clone, Copy)]
pub struct HachiConfig {
    pub max_moves: usize,
    pub max_responses: usize,
    pub beam_width: usize,
    pub use_exact: bool,
    pub model_type: ModelType
}

/*
    `max_moves`: Number of moves to consider for playing side.
    `max_responses`: Number of moves to consider for opponent side.
    `use_exact`: `true` to solve equilibriums exactly, or `false` to approximate.
    `model_type`: Model to use for evaluation at leaf positions.
*/

impl Default for HachiConfig {
    fn default() -> Self {
        Self {
            max_moves: 4,
            max_responses: 4,
            use_exact: false,
            beam_width: 150,
            model_type: ModelType::LightGBM_Large
        }
    }
}

impl HachiConfig {
    pub fn rapid() -> Self {
        Self {
            max_moves: 2,
            max_responses: 1,
            use_exact: true,
            beam_width: 250,
            model_type: ModelType::LightGBM_Large
        }
    }
    pub fn beam() -> Self {
        Self {
            max_moves: 1,
            max_responses: 1,
            use_exact: false,
            beam_width: 250,
            model_type: ModelType::LightGBM_Large
        }
    }
    pub fn beam_narrow() -> Self {
        Self {
            max_moves: 1,
            max_responses: 1,
            use_exact: false,
            beam_width: 100,
            model_type: ModelType::LightGBM_Large
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ChanceState {
    pub gamestate: GameState,
    pub garbage: u32
}


pub fn default_move() -> Move {
    Move {
        x: 0,
        y: 0,
        r: Rotation::North,
        kind: Piece::O,
        tspin: None,
    }
}

pub fn u64_to_piece(x:u64) -> Piece{
    match x {
        0 => Piece::I,
        1 => Piece::O,
        2 => Piece::T,
        3 => Piece::L,
        4 => Piece::S,
        5 => Piece::J,
        _ => Piece::Z,
    }
}

static NODE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_node_id() -> u64 {
    NODE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub fn solve_position(gs1: GameState, gs2: GameState, depth: usize, config: HachiConfig) -> (Move, f64) {
    solve_position_inner(gs1, gs2, depth, config, None, true)
}

// Solve a two-board position using subgame perfect equilibrium.
// Returns: (best_move, win_probability)
fn solve_position_inner(
    mut gamestate1: GameState, 
    mut gamestate2: GameState, 
    depth: usize, 
    config: HachiConfig,
    parent_id: Option<u64>,
    canonical: bool
) -> (Move, f64) {
    
    let node_id = next_node_id();
    let gs1_snapshot = format!("{:#?}", if canonical { gamestate1 } else { gamestate2 });
    let gs2_snapshot = format!("{:#?}", if canonical { gamestate2 } else { gamestate1 });

    let queue1 = gamestate1.queue;
    let queue2 = gamestate2.queue;

    let state1:State = gamestate_to_state(&gamestate1);
    let state2:State = gamestate_to_state(&gamestate2);

    // 20ms here
    let moves1 = get_pruned_moves(&state1, &queue1, config.max_moves, config.beam_width);
    let moves2 = get_pruned_moves(&state2, &queue2, config.max_responses, config.beam_width);

    // check if we are dead
    if moves1.len() == 0 {
        TREE_DUMP.with(|d| d.borrow_mut().nodes.push(TreeNode {
            id: node_id, parent_id, depth,
            gamestate_row: gs1_snapshot, gamestate_col: gs2_snapshot,
            value: 0.0,
        }));
        return (default_move(), 0.0);
    }

    // check if they are dead
    if moves2.len() == 0 {
        TREE_DUMP.with(|d| d.borrow_mut().nodes.push(TreeNode {
            id: node_id, parent_id, depth,
            gamestate_row: gs1_snapshot, gamestate_col: gs2_snapshot,
            value: 1.0,
        }));
        return (moves1[0], 1.0);
    }

    gamestate1.attack = 0;
    gamestate2.attack = 0;

    let make_state = |mv: &Move, state: &State, queue: &[Piece;5], gamestate: &GameState| {
        let mut s = state.clone();
        let lock = s.make(mv, queue);
        let mut gs = gamestate.clone();
        gs.board = s.board;
        gs.b2b = s.b2b;
        gs.hold = s.hold;
        gs.combo = s.combo;
        gs.queue.rotate_left(1);

        // cheeky pseudo rng - monte carlo speculation
        gs.queue[4] = u64_to_piece(xorshift64() % 7);

        let next_meter = gs.meter.saturating_sub(lock.sent);
        gs.attack = lock.sent - gs.meter.abs_diff(next_meter);
        gs.meter = next_meter;
        // println!("cleared lines {}", lock.cleared);
        let garbage = if lock.cleared == 0 {
            let ret = gs.meter as u32;
            gs.meter = 0;
            ret
        } else {
            0
        };

        ChanceState {
            gamestate: gs,
            garbage,
        }
    };

    // println!("row player");
    let row_states: Vec<ChanceState> = moves1
        .iter()
        .map(|mv| make_state(mv, &state1, &queue1, &gamestate1))
        .collect();

    // println!("col player");
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

    // println!("depth {}", depth);
    
    /*
        Subgame solving complexity:
        O(((m^depth + n^depth)*beam_search_cost) + (m*n*eval_cost)^depth)
    */
    
    // calculate payoffs

    let mut payoffs: Vec<Vec<f64>> = vec![vec![0.0; n]; m];
    for i in 0..m {
        for j in 0..n {

            // it's impossible for both players to tank garbage at the same time
            // (no passthrough)
            debug_assert!(!(row_states[i].garbage > 0 && col_states[j].garbage > 0));

            // it's also impossible for both meters to contain garbage
            debug_assert!(!(row_states[i].gamestate.meter > 0 && col_states[j].gamestate.meter > 0));

            // get payoff for chance state from that player's perspective 
            // (make sure to use correct sign afterwards)
            let get_average_payoff = |
                chance_state: &ChanceState, 
                opponent_index: usize, 
                opponent_states: &Vec<ChanceState>,
                opponent_features: &Vec<Option<Features>>,
                flip: bool| {

                let realized_states: Vec<GameState> = [xorshift64() % 10; 1].iter().map(
                        |k| {
                            let mut gs = chance_state.gamestate.clone();
                            gs.tank_garbage(chance_state.garbage, *k as usize);
                            // take incoming damage into meter
                            gs.meter += opponent_states[opponent_index].gamestate.attack;
                            gs
                        }
                    ).collect();

                if depth == 1 {
                    // use evaluation payoff
                    realized_states.iter().fold(0.0, |accum, &gamestate| {
                        if let None = opponent_features[opponent_index] {
                            println!("{:#?}", chance_state);
                            println!("{:#?}", opponent_states[opponent_index]);
                            unreachable!();
                        }
                        accum + eval(
                            &extract_features(&gamestate), 
                            opponent_features[opponent_index].as_ref().unwrap(),
                            config.model_type
                        )
                    }) / 10.0
                } else {
                    // use subgame payoff
                    realized_states.iter().fold(0.0, |accum, &gamestate| {
                        let (_, value) = solve_position_inner(
                            gamestate, 
                            opponent_states[opponent_index].gamestate,
                            depth - 1,
                            config,
                            Some(node_id),
                            canonical ^ flip
                        );
                        accum + value
                    }) / 10.0
                }
            };

            let payoff = if 
                row_states[i].garbage == 0 && // not tanking
                col_states[j].garbage == 0 &&
                (row_states[i].gamestate.attack ==
                col_states[j].gamestate.attack) // meters not changing
            {
                
                // println!("no interaction");
                // case 1: not a chance state and no interaction
                if depth == 1 {
                    eval(row_features[i].as_ref().unwrap(), col_features[j].as_ref().unwrap(), config.model_type)
                } else {
                    let (_, value) = solve_position_inner(
                        row_states[i].gamestate, 
                        col_states[j].gamestate,
                        depth - 1,
                        config,
                        Some(node_id),
                        canonical
                    );
                    value
                }
            }
            else if row_states[i].garbage != 0 {
                
                // println!("row player is tanking");
                // row player has a chance state
                get_average_payoff(&row_states[i], j, &col_states, &col_features, false)
            }
            else if col_states[j].garbage != 0 {
                // col player has a chance state
                // subtract from 1 to get row's score
                // println!("col player is tanking");
                1.0 - get_average_payoff(&col_states[j], i, &row_states, &row_features, true)
            }
            else {
                // no chance state, but interaction exists
                // at least one player is sending damage
                // the player who sends more 'wins' the trade. the other 'loses'.
                
                // println!("nobody is tanking but trade is happening");

                let (winner, winner_features, loser, loser_features, row_wins) =
                    if row_states[i].gamestate.attack > col_states[j].gamestate.attack {
                        // println!("row player wins trade");
                        (&row_states[i], &row_features[i], &col_states[j], &col_features[j], true)
                    } else {
                        // println!("col player wins trade");
                        (&col_states[j], &col_features[j], &row_states[i], &row_features[i], false)
                    };

                let mut loser_state = loser.gamestate.clone();
                loser_state.meter += winner.gamestate.attack;

                // use the fact that eval(row,col) = 1 - eval(col,row)

                let winner_value = if depth == 1 {
                    eval(
                        winner_features.as_ref().unwrap(),
                        &extract_features(&loser_state),
                        config.model_type,
                    )
                } else {
                    let (_, v) = solve_position_inner(
                        winner.gamestate.clone(),
                        loser_state,
                        depth - 1,
                        config,
                        Some(node_id),
                        canonical ^ (!row_wins)
                    );
                    v
                };

                if row_wins { winner_value } else { 1.0 - winner_value }
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
    let display_payoffs: Vec<Vec<f64>> = payoffs.iter()
    .map(|row| row.iter().map(|&v| if canonical { v } else { 1.0 - v }).collect())
    .collect();
    println!("depth {}", depth);
    println!("payoffs {:?}", display_payoffs);
    println!("row player? {}", canonical);
    println!("value {}", if canonical { value } else { 1.0 - value });

    // execute mixed strategy

    let dist = WeightedIndex::new(&row_strategy).unwrap();
    let mut rng = thread_rng();

    let selected_index = dist.sample(&mut rng);

    TREE_DUMP.with(|d| {
        d.borrow_mut().nodes.push(TreeNode {
            id: node_id,
            parent_id,
            depth,
            gamestate_row: gs1_snapshot,
            gamestate_col: gs2_snapshot,
            value: if canonical { value } else { 1.0 - value },
        });
    });

    (moves1[selected_index], value)
}


pub fn get_pruned_moves(state: &State, queue: &[Piece; 5], n: usize, w: usize) -> Vec<Move> {
    let key = TableKey { 
        state: state, 
        piece: queue[0] 
    };
    TTABLE.with(|table| {
        if let Some(entry) = table.borrow().get(&key) {
            return entry.moves.clone();
        } else {
            // 10ms per call here
            let result = beam_top_n(
                state.clone(),
                Lock {
                    cleared: 0,
                    sent: 0,
                    softdrop: false,
                },
                queue,
                Weights::default(),
                BotConfigs {
                    width: w
                },
                n
            );
            if let Ok(moves) = result {    
                table.borrow_mut().put(
                    &key,
                    TableValue{ 
                        moves: moves.clone()
                    }
                );
                moves
            } else {
                Vec::new() 
            }
        }
    })
}

pub fn beam_top_n(
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