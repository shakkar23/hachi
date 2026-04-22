
use std::cmp::Reverse;
use std::collections::hash_map::Entry;
use std::collections::{HashMap};
use std::cell::RefCell;

use tetris::state::State;
use tetris::moves::Move;
use tetris::piece::Piece;

pub struct TableKey<'a> {
    pub state: &'a State,
    pub piece: Piece
}

#[derive(Debug, Clone)]
pub struct TableValue {
    pub moves: Vec<Move>
}

pub struct MoveTable {
    pub map: HashMap<u64, TableValue>
}

fn hash(key: &TableKey) -> u64 {
    let mut state: u64 = 0x9E3779B97F4A7C15;
    
    for x in key.state.board.cols {
        state ^= x;
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state = state.wrapping_mul(0x5851F42D4C957F2D);
    }
    
    state ^= key.piece as u64;
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state = state.wrapping_mul(0x5851F42D4C957F2D);
    
    state
}

impl MoveTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::with_capacity(1 << 12)
        }
    }
    pub fn get(&self, key: &TableKey) -> Option<&TableValue> {
        self.map.get(&hash(key))
    }

    pub fn put(&mut self, key: &TableKey, value: TableValue){
        return;
        self.map.insert(hash(key), value);
    }
}