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
    pub fn tank_garbage(&mut self, mut amount:u32, column:usize) {
        if amount == 0 {
            return;
        }

        if amount >= 32 {
            amount = 32;
        }

        for i in 0..10 {
            self.board.cols[i] <<= amount as u64;
            let should_fill = i != column;
            self.board.cols[i] |= if should_fill {(1 << amount) - 1} else {0};
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            board: Board {
                cols: [
                    0,0,0,0,0,
                    0,0,0,0,0
                ],
            },
            current_piece: Piece::I,
            placement: Move {
                move_type: None,
                rotation: Rotation::North,
                x: 0,
                y: 0,
            },
            meter: 0,
            combo: 0,
            attack: 0,
            b2b: 0,
            damage_received: 0,
            spun: false,
            queue: [Piece::I, Piece::I, Piece::I, Piece::I, Piece::I],
            hold: None,
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


#[test]
fn test_tank_garbage_all_columns() {
    let mut game = GameState::default();
    game.board = Board {
        cols: [1u64; 10], // single bit at position 0 in each column
    };
    
    game.tank_garbage(3, 7);
    
    for i in 0..10 {
        if i == 7 {
            // Column 7: shifted only, no garbage fill
            assert_eq!(game.board.cols[i], 1 << 3);
        } else {
            // Other columns: shifted + garbage bits in low 3 positions
            assert_eq!(game.board.cols[i], (1 << 3) | 0b111);
        }
    }
}