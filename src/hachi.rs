use tetris::board::Board;
use crate::game::{GameState};

fn get_heights(board: &Board) -> [u32; 10] {
    let mut heights = board.heights();

    heights
}

fn get_height_differences(board: &Board) -> [i32; 9] {
    let mut heights = board.heights();
    let mut diffs = [0; 9];

    for x in 0..9 {
        diffs[x] = (heights[x+1] as i32 - heights[x] as i32);
    }

    diffs
}

// bits from surface until first hole (0 for no holes)
fn get_first_hole_depths(board: &Board) -> [i32; 10] {
    let heights = board.heights();
    let mut depths = [0; 10];

    for x in 0..10 {
        depths[x] = board.cols[x].trailing_ones() as i32;
        if heights[x] as i32 == depths[x] {
            depths[x] = 0;
        }
    }

    depths
}

// next 10 garbage holes
fn get_garbage_hole_sequence(board: &Board) -> [i32; 10] {
    [0; 10]
}

fn get_distance_to_next_piece(gamestate: &GameState) -> [i32; 7] {
    let queue = gamestate.queue;
    let mut distances = [5;7];

    for i in (0..5).rev() {
        distances[queue[i] as usize] = i as i32;
    }

    distances
}

fn get_count_of_pieces(gamestate: &GameState) -> [i32; 7] {let queue = gamestate.queue;
    let mut counts = [0;7];

    for i in 0..5 {
        counts[queue[i] as usize] += 1;
    }

    counts
}

// more complex, requires lookahead calculation
fn get_maximum_combo(gamestate: &GameState) -> i32 {
    0
}

fn get_height_variance(board: &Board) -> f32 {
    0.0f32
}

// get one hot codings for certain pieces in queue
fn get_hold_or_current_piece(gamestate: &GameState) -> [i32; 7] {
    let mut onehot = [0;7];

    match gamestate.hold {
        Some(value) => (onehot[value as usize] += 1),
        None => ()
    }

    onehot[gamestate.current_piece as usize] += 1;

    onehot
}

// one hot for next piece
fn get_next_piece(gamestate: &GameState) -> [i32; 7] {
    let mut onehot = [0;7];

    onehot[gamestate.queue[0] as usize] = 1;

    onehot
}

fn get_3x3s(board: &Board) -> [i32; 512] {
    let mut counts = [0; 512];
    let height = board.heights().iter().copied().max().unwrap() as usize;

    for x in 0..7 {
        for y in 0..height {
            let mask = [0b111 << y, 0b111 << y, 0b111 << y];
            let cols = &board.cols[x..=x+2];
            let window = [
                (cols[0] & mask[0]) >> y,
                (cols[1] & mask[1]) >> y,
                (cols[2] & mask[2]) >> y
            ];
            counts[(window[0] | (window[1] << 3) | (window[2] << 6)) as usize] += 1;
        }
    }

    counts
}

pub struct HachiFeatures {
    pub heights:[u32;10],
    pub height_differences:[i32;9],
    pub first_hole_depths:[i32;10],
    pub piece_distance:[i32;7],
    pub piece_counts:[i32;7],
    pub hold_or_current_onehot:[i32;7],
    pub next_onehot:[i32;7],
    pub all_3x3s:[i32;512]
}

pub fn get_hachi_features(gamestate: &GameState) -> HachiFeatures {
    let mut board = gamestate.board;
    HachiFeatures {
        heights: get_heights(&board),
        height_differences: get_height_differences(&board),
        first_hole_depths: get_first_hole_depths(&board),
        piece_distance: get_distance_to_next_piece(&gamestate),
        piece_counts: get_count_of_pieces(&gamestate),
        hold_or_current_onehot: get_hold_or_current_piece(&gamestate),
        next_onehot: get_next_piece(&gamestate),
        all_3x3s: get_3x3s(&board)
    }
}