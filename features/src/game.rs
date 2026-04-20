use tetris::{board::Board, piece::Piece, piece::Rotation};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    pub move_type:Option<Piece>,
    pub rotation:Rotation,
    pub x:u8,
    pub y:u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl GameState {
    pub fn tank_garbage(&mut self, amount:u32, column:usize) {
        if amount == 0 {
            return;
        }

        for i in 0..10 {
            self.board.cols[i] <<= amount as u64;
            let should_fill = column == i;
            self.board.cols[i] |= if should_fill {(1 << amount) - 1} else {0};
        }
    }
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
