use tetris::piece::Piece;
use tetris::state::{State};

pub struct FullState {
    pub state: State,
    pub queue: [Piece;5]
}

pub struct MacroState {
    pub p1: FullState,
    pub p2: FullState
}