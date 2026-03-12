use tetris::board::Board;
use tetris::moves::Move;
use tetris::piece::{Piece, Rotation};
use tetris::movegen::{movegen};
use tetris::state::{State};

use crate::state::{FullState, MacroState};

use std::collections::HashMap;
use std::mem;
use std::cmp;

use crate::eval::{light_eval, lock_eval, heavy_eval};

/*
    Nested search combining minimax with beam search.

    The core method is alpha-beta search, but since the game tree is
    large, we use beam search at each ply to reduce the number of moves we 
    consider for 2-player interactions. This also provides a move-ordering strategy.

    At lower depths of the tree, we use prune exponentially to control the number of leaf states.

    For example, using a log_2 pruning strategy:
    At the root, we consider our top 32 moves. Then we consider opponent's 16 responses, our 8 responses,
    their 4, our 2, and we finally end at depth 5. This results in a maximum of 32,768 leaf states, and
    the same amount of interior states.

    This seems like a lot, but nearly all interior nodes hit the transposition table,
    because there are very few player interactions that can affect the opponent's legal moves.

    To be more specific, we can define a pruning decay rate according to desired search depth:

    gamma = starting_width^(-1/depth)

    To target a search budget of about 300ms, we use the following defaults:

    beam width: 100
    beam depth: 3
    minimax depth: 5
    minimax root width: 60

*/


/*
    We use two transposition tables.

    The first transposition table is used for beam search and memoizes the results of light eval and movegen.

    The second transposition table is used for alpha beta pruning and stores post-pruning moves and aspiration windows.
*/

// hash types

type PseudoHash = u64;

type ExactHash = u64;

// for beam search

pub struct BTable {
    pub values: HashMap<PseudoHash, i32>, // light evaluation
    pub legal_moves: HashMap<PseudoHash, Vec<Move>>, // moves found by movegen (used for beam search)
    pub top_moves: HashMap<PseudoHash, Vec<Move>>, // moves selected by beam search (used for alpha beta)
}

// for alpha beta

pub struct TTEntry {
    pub bound: (i32, Bound), // bound value, heavy evaluation
    pub depth: u16,
}

type MTable = HashMap<ExactHash, TTEntry>;

enum Bound {
    Exact,
    Lower,
    Upper,
}

// hash that preserves legal moves
pub fn pseudo_hash(state: &State) -> PseudoHash {
    state.board.cols.iter().fold(0, |acum, &item|
        acum ^ (acum << 19) ^ (acum >> 11) ^ item 
    )
}

// hash preserving all information
pub fn exact_hash(state: &MacroState) -> ExactHash {
    let mut hash: u64 = 0;
    hash ^= pseudo_hash(&state.p1.state) << 32;
    hash ^= pseudo_hash(&state.p2.state);
    hash
}

/*
    Initialise new search, including transposition tables
    Then return best move
*/

pub fn search(
    state: &MacroState,
    depth: i32
) -> Move {
    let mut btable = BTable {
        values: HashMap::new(),
        legal_moves: HashMap::new(),
        top_moves: HashMap::new(),
    };
    let mut mtable: MTable = HashMap::new();
    alpha_beta_search(state, &mut btable, &mut mtable, depth)
}

/*
    Generated max_moves moves, sorted according to goodness.

    Exit early if # of candidates <= max_moves.
*/

struct BeamItem {
    pub state: State,
    pub root_idx: usize,
    pub hash: PseudoHash,
    pub score: i32
}

struct ScoredMove {
    pub value: Move,
    pub score: i32
}

fn tt_movegen(btable: &mut BTable, state: &State, queue: &[Piece]) -> Vec<Move> {
    let hash = pseudo_hash(state);
    btable.legal_moves.entry(hash).or_insert_with(|| {
        let current = queue[state.next];
        let hold = state
            .hold
            .unwrap_or_else(|| queue[state.next + 1]);

        let mut moves = movegen(&state.board, current);

        if hold != current {
            moves.extend(movegen(&state.board, hold));
        }

        moves
    }).clone()
}

pub fn beam_search(
    root: &FullState, 
    mut btable: &mut BTable,
    depth: i32, 
    width: usize, 
    max_moves: usize
) -> Vec<Move> {

    let mut beam: Vec<BeamItem> = Vec::new();

    let mut new_beam: Vec<BeamItem> = Vec::new();

    let queue = root.queue; // queue is fixed

    let mut root_moves:Vec<ScoredMove> = tt_movegen(&mut btable, &root.state, &queue).into_iter().map(|m| {
        ScoredMove{value: m, score: i32::MIN}
    }).collect();

    // expand root

    for idx in 0..root_moves.len() {
        let mv = root_moves[idx].value;
        let mut child_state = root.state.clone();
        let lock = child_state.make(&mv, &queue);
        let child_hash = pseudo_hash(&child_state);
        let score = btable.values.entry(child_hash).or_insert_with(|| {
            light_eval(&child_state)
        });

        beam.push(BeamItem{
            state: child_state,
            root_idx: idx,
            hash: child_hash,
            score: *score + lock_eval(&lock)
        });
    }

    // do beam search

    for d in 0..depth {

        // crunch beam width
        beam.select_nth_unstable_by_key(width - 1, |a| {
            -a.score
        });

        beam.truncate(width);

        for item in &beam {

            let state = &item.state;

            // movegen
            let moves = tt_movegen(&mut btable, &state, &queue);

            // create new nodes
            for mv in &moves {
                let mut child_state = state.clone();
                let lock = child_state.make(&mv, &queue);
                let child_hash = pseudo_hash(&child_state);
                let score = btable.values.entry(child_hash).or_insert_with(|| {
                    light_eval(&child_state)
                });

                new_beam.push(BeamItem{
                    state: child_state,
                    root_idx: item.root_idx,
                    hash: child_hash,
                    score: *score + lock_eval(&lock)
                });
            }
        }

        mem::swap(&mut beam, &mut new_beam);
        new_beam.clear();
    }

    for item in &beam {
        let root_move = &mut root_moves[item.root_idx];
        root_move.score = cmp::max(item.score, root_move.score);
    }

    root_moves.select_nth_unstable_by_key(max_moves - 1, |a| -a.score);
    root_moves.truncate(max_moves);
    root_moves.into_iter().map(|m| m.value).collect()
}

/*
    Alpha-beta search using beam search for move ordering and early pruning.
*/
pub fn alpha_beta_search(
    state: &MacroState,
    btable: &mut BTable,
    mtable: &mut MTable,
    depth: i32,
) -> Move {
    let mut best_move = Move {
        x: 0, y: 0,
        r: Rotation::North,
        kind: Piece::O,
        tspin: None,
    };
    let mut alpha = i32::MIN + 1; // avoid overflow on negation
    let beta = i32::MAX;

    let beam_width = 100;
    let beam_depth = 3;
    let max_moves: usize = 60;
    let candidates = beam_search(&state.p1, btable, beam_depth, beam_width, max_moves);

    // Cache candidates
    let p1_hash = pseudo_hash(&state.p1.state);
    btable.top_moves.insert(p1_hash, candidates.clone());

    for mv in &candidates {
        let mut child_macro = MacroState {
            p1: FullState {
                state: state.p1.state.clone(),
                queue: state.p1.queue,
            },
            p2: FullState {
                state: state.p2.state.clone(),
                queue: state.p2.queue,
            },
        };
        child_macro.p1.state.make(mv, &child_macro.p1.queue);

        let score = -negamax(
            &child_macro,
            btable,
            mtable,
            beam_width,
            cmp::max(max_moves >> 1, 2),
            depth - 1,
            -beta,
            -alpha,
            false, // next ply is opponent (p2)
        );

        if score > alpha {
            alpha = score;
            best_move = *mv;
        }
    }

    best_move
}

/// Negamax with alpha-beta pruning.
/// `maximizing` = true means it's p1's turn, false = p2's turn.
fn negamax(
    state: &MacroState,
    btable: &mut BTable,
    mtable: &mut MTable,
    beam_width: usize,
    max_moves: usize,
    depth: i32,
    mut alpha: i32,
    beta: i32,
    maximizing: bool,
) -> i32 {
    let hash = exact_hash(state);

    // Transposition table lookup
    if let Some(entry) = mtable.get(&hash) {
        if entry.depth >= depth as u16 {
            let (value, ref bound) = entry.bound;
            match bound {
                Bound::Exact => return value,
                Bound::Lower => {
                    if value >= beta { return value; }
                }
                Bound::Upper => {
                    if value <= alpha { return value; }
                }
            }
        }
    }

    // Terminal / horizon node: full evaluation
    if depth <= 0 {
        let score = heavy_eval(&state);
        mtable.insert(hash, TTEntry {
            bound: (score, Bound::Exact),
            depth: 0,
        });
        return score;
    }

    // Select which player acts this ply
    let (active, _passive) = if maximizing {
        (&state.p1, &state.p2)
    } else {
        (&state.p2, &state.p1)
    };

    let active_hash = pseudo_hash(&active.state);
    let candidates: Vec<Move> = if let Some(cached) = btable.top_moves.get(&active_hash) {
        cached.clone()
    } else {
        let moves = beam_search(active, btable, 2, beam_width, max_moves);
        btable.top_moves.insert(active_hash, moves.clone());
        moves
    };

    if candidates.is_empty() {
        // No moves available — treat as terminal
        let score = heavy_eval(state);
        return score;
    }

    let original_alpha = alpha;
    let mut best = i32::MIN + 1;

    for mv in &candidates {
        // Build child MacroState
        let mut child = MacroState {
            p1: FullState { state: state.p1.state.clone(), queue: state.p1.queue },
            p2: FullState { state: state.p2.state.clone(), queue: state.p2.queue },
        };

        if maximizing {
            child.p1.state.make(mv, &child.p1.queue);
        } else {
            child.p2.state.make(mv, &child.p2.queue);
        }

        let score = -negamax(
            &child,
            btable,
            mtable,
            beam_width,
            cmp::max(max_moves >> 1, 2 as usize),
            depth - 1,
            -beta,
            -alpha,
            !maximizing,
        );

        if score > best {
            best = score;
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break; // beta cut-off
        }
    }

    // Store result in transposition table with appropriate bound
    let bound = if best <= original_alpha {
        Bound::Upper
    } else if best >= beta {
        Bound::Lower
    } else {
        Bound::Exact
    };

    mtable.insert(hash, TTEntry {
        bound: (best, bound),
        depth: depth as u16,
    });

    best
}