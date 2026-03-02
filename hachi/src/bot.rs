use tetris::board::Board;
use tetris::moves::Move;
use tetris::piece::{Piece, Rotation};
use tetris::movegen::{movegen};
use tetris::state::{State};

use crate::eval::{light_eval, heavy_eval}

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

    To be more specific, we define a pruning decay rate according to desired search depth:

    gamma = starting_width^(-1/depth)

    To target a search budget of about 300ms, we use the following defaults:

    beam width: 100
    beam depth: 3
    minimax depth: 5
    minimax root width: 60

*/

pub struct MacroState {
    pub p1: State,
    pub p2: State
}

/*
    We use two transposition tables.

    The first transposition table is used for beam search and memoizes the results of light eval and movegen.

    The second transposition table is used for alpha beta pruning and stores post-pruning moves and aspiration windows.
*/

// hash types

pub struct PseudoHash(u64);
pub struct ExactHash(u64);

// for beam search

pub struct BTable {
    pub value: HashMap<PseudoHash, i32>, // light evaluation
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

// hash that ignores vertical translations 
pub fn beam_hash(state: &State) -> PseudoHash {

}

// hash preserving all information
pub fn minimax_hash(state: &MacroState) -> ExactHash {

}

/*
    Initialise new search, including transposition tables
    Then return best move
*/

pub fn search(
    state: &MacroState,
    depth: i32
) -> Move {

}

/*
    Generated max_moves moves, sorted according to goodness.

    Exit early if # of candidates <= max_moves.
*/

pub fn beam_search(
    state: &State, 
    btable: &mut BTable,
    depth: i32, 
    width: usize, 
    max_moves: usize
) -> Vec<Move> {

}

/*
    Alpha-beta search using beam search for move ordering and early pruning.


*/

pub fn alpha_beta_search(
    state: &MacroState,
    btable: &mut BTable,
    mtable: &mut MTable,
    depth: i32
) -> Move {
    Move (
        x: 0,
        y: 0,
        r: Rotation::North,
        kind: Piece::O,
        tspin: None,
    )
}