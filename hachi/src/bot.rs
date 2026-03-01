use features::game::{GameState};
use features::feature_extractor::{Features, extract_features};

use tetris::board::Board;
use tetris::moves::Move;
use tetris::piece::{Piece, Rotation};
use tetris::movegen::{movegen};
use tetris::state::{State};

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

    Explanation:

    Branching factor progression is

    60, 26, 11, 5, 2, 1.

    Number of beam searches = 26+11+5+2+1 = 45

    45 beam searches -> 100*30*3*45 = 9,000 * 45 = 405,000 calls to light eval.

    product of branches -> 171,600 calls to heavy eval.

    Linear eval benches at ~3M / second and heavy eval at 1M / second using distilled CatBoost on a i7-8665U.

*/

pub fn beam_search(state: &State, depth: i32, width: usize, max_moves: usize) -> Vec<Move> {

}

pub fn minimax_search(gamestate: &GameState, depth: i32) -> Move {
    Move (
        x: 0,
        y: 0,
        r: Rotation::North,
        kind: Piece::O,
        tspin: None,
    )
}