use tetris::board::Board;
use tetris::moves::Move;
use tetris::piece::{Piece, Rotation};
use tetris::movegen::{movegen};
use tetris::state::{State};

pub struct FullState {
    pub state: State,
    pub queue: [Piece;5]
}

pub struct MacroState {
    pub p1: FullState,
    pub p2: FullState
}