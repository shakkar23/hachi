use tetris::piece::Piece;
use features::game::{GameState};

pub struct FullState {
    pub state: GameState,
    pub queue: [Piece;5]
}

pub struct MacroState {
    pub p1: FullState,
    pub p2: FullState
}