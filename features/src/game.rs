use tetris::{board::Board, piece::Piece, piece::Rotation};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Move {
    pub move_type:Option<Piece>,
    pub rotation:Rotation,
    pub x:u8,
    pub y:u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameState {
    pub board: Board,
    pub current_piece:Piece,
    pub placement:Move,
    pub meter:u8,
    pub combo:u8,
    pub attack:u8,
    pub b2b:u8,
    pub damage_received:u8,
    pub spun:bool,
    pub queue:[Piece;5],
    pub hold:Option<Piece>
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum State {
    PLAYING, // 0
    P1_WIN, // 1
    P2_WIN, // 2
    DRAW // 3
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Datum {
    pub p1:GameState,
    pub p2:GameState,
    pub state:State,
    pub game_id:u16,
    pub move_index:u16
}
