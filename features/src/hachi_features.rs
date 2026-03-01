use tetris::board::Board;
use crate::game::{GameState};

fn get_heights(board: &Board) -> [u32; 10] {
    let mut heights = board.heights();

    heights
}

fn get_height_differences(board: &Board) -> [i16; 9] {
    let mut heights = board.heights();
    let mut diffs = [0; 9];

    for x in 0..9 {
        diffs[x] = (heights[x+1] as i16 - heights[x] as i16);
    }

    diffs
}

// bits from surface until first hole (0 for no holes)
fn get_first_hole_depths(board: &Board) -> [i16; 10] {
    let heights = board.heights();
    let mut depths = [0; 10];

    for x in 0..10 {
        depths[x] = board.cols[x].leading_ones() as i16;
        if heights[x] as i16 == depths[x] {
            depths[x] = 0;
        }
    }

    depths
}

// locations of all garbage holes
fn get_garbage_hole_sequence(board: &Board) -> [i16; 20] {
    let mut locations = [-1; 20];
    for y in 0..20 {
        let xs: Vec<i16> = board.cols.iter().map(|col| {
            ((col >> y) & 1) as i16
        }).collect();
        
        let mut sum = 0;
        for x in 1..10 {
            sum += xs[x];
            if xs[x] == 0 {
                locations[y] = x as i16;
            }
        }
        if sum != 9 {
            locations[y] = -1;
        }
    }

    locations
}

fn get_distance_to_next_piece(gamestate: &GameState) -> [i16; 7] {
    let queue = gamestate.queue;
    let mut distances = [5;7];

    for i in (0..5).rev() {
        distances[queue[i] as usize] = i as i16;
    }

    distances
}

fn get_count_of_pieces(gamestate: &GameState) -> [i16; 7] {let queue = gamestate.queue;
    let mut counts = [0;7];

    for i in 0..5 {
        counts[queue[i] as usize] += 1;
    }

    counts
}

// more complex, requires lookahead calculation
fn get_maximum_combo(gamestate: &GameState) -> i16 {
    0
}

fn get_height_variance(board: &Board) -> f32 {
    0.0f32
}

// get one hot codings for certain pieces in queue
fn get_hold_or_current_piece(gamestate: &GameState) -> [i16; 7] {
    let mut onehot = [0;7];

    match gamestate.hold {
        Some(value) => (onehot[value as usize] += 1),
        None => ()
    }

    onehot[gamestate.current_piece as usize] += 1;

    onehot
}

// one hot for next piece
fn get_next_piece(gamestate: &GameState) -> [i16; 7] {
    let mut onehot = [0;7];

    onehot[gamestate.queue[0] as usize] = 1;

    onehot
}

fn get_3x3s(board: &Board) -> ([i16; 512], [i16; 512], [i16; 512]) {
    
    let height = board.heights().iter().max().unwrap().clone();

    let mut counts = [0; 512];
    let mut counts_with_x = [0; 512];
    let mut counts_with_y = [0; 512];

    // max encoding sizes
    const x_cutoff:i16 = i16::MAX.ilog(8) as i16; // 5
    const y_cutoff:i16 = i16::MAX.ilog(10) as i16; // 4

    for y in (0..height).rev() {
        for x in 0..8 {
            let mask = [0b111 << y, 0b111 << y, 0b111 << y];
            let cols:&[u64] = &board.cols[x..=x+2];
            let window = [
                (cols[0] & mask[0]) >> y,
                (cols[1] & mask[1]) >> y,
                (cols[2] & mask[2]) >> y
            ];
            // ID of 3x3 filter that matches exactly
            let idx = (window[0] | (window[1] << 3) | (window[2] << 6)) as usize;

            // increment counter for how often this pattern has appeared
            counts[idx] += 1;

            counts_with_x[idx] += x as i16;

            counts_with_y[idx] += y as i16;
        }
    }
    (counts, counts_with_x, counts_with_y)
}

fn get_2x2s(board: &Board) -> ([i16; 16], [i16; 16], [i16; 16]) {
    
    let height = board.heights().iter().max().unwrap().clone();

    let mut counts = [0; 16];
    let mut counts_with_x = [0; 16];
    let mut counts_with_y = [0; 16];

    // max encoding sizes
    const x_cutoff:i16 = i16::MAX.ilog(9) as i16; // 5
    const y_cutoff:i16 = i16::MAX.ilog(10) as i16; // 4

    for y in (0..height).rev() {
        for x in 0..9 {
            let mask = [0b11 << y, 0b11 << y];
            let cols:&[u64] = &board.cols[x..=x+1];
            let window = [
                (cols[0] & mask[0]) >> y,
                (cols[1] & mask[1]) >> y
            ];
            // ID of 2x2 filter that matches exactly
            let idx = (window[0] | (window[1] << 2)) as usize;

            // increment counter for how often this pattern has appeared
            counts[idx] += 1;

            counts_with_x[idx] += x as i16;

            counts_with_y[idx] += y as i16;
        }
    }
    (counts, counts_with_x, counts_with_y)
}

fn get_2x3s(board: &Board) -> ([i16; 64], [i16; 64], [i16; 64]) {
    
    let height = board.heights().iter().max().unwrap().clone();

    let mut counts = [0; 64];
    let mut counts_with_x = [0; 64];
    let mut counts_with_y = [0; 64];

    // max encoding sizes
    const x_cutoff:i16 = i16::MAX.ilog(9) as i16; // 5
    const y_cutoff:i16 = i16::MAX.ilog(10) as i16; // 4

    for y in (0..height).rev() {
        for x in 0..9 {
            let mask = [0b111 << y, 0b111 << y];
            let cols:&[u64] = &board.cols[x..=x+1];
            let window = [
                (cols[0] & mask[0]) >> y,
                (cols[1] & mask[1]) >> y
            ];
            // ID of 2x2 filter that matches exactly
            let idx = (window[0] | (window[1] << 3)) as usize;

            // increment counter for how often this pattern has appeared
            counts[idx] += 1;

            counts_with_x[idx] += x as i16;

            counts_with_y[idx] += y as i16;
        }
    }
    (counts, counts_with_x, counts_with_y)
}


fn get_3x2s(board: &Board) -> ([i16; 64], [i16; 64], [i16; 64]) {
    
    let height = board.heights().iter().max().unwrap().clone();

    let mut counts = [0; 64];
    let mut counts_with_x = [0; 64];
    let mut counts_with_y = [0; 64];

    // max encoding sizes
    const x_cutoff:i16 = i16::MAX.ilog(9) as i16; // 5
    const y_cutoff:i16 = i16::MAX.ilog(10) as i16; // 4

    for y in (0..height).rev() {
        for x in 0..8 {
            let mask = [0b11 << y, 0b11 << y, 0b11 << y];
            let cols:&[u64] = &board.cols[x..=x+2];
            let window = [
                (cols[0] & mask[0]) >> y,
                (cols[1] & mask[1]) >> y,
                (cols[2] & mask[2]) >> y
            ];
            // ID of 2x2 filter that matches exactly
            let idx = (window[0] | (window[1] << 2 | window[2] << 4)) as usize;

            // increment counter for how often this pattern has appeared
            counts[idx] += 1;

            counts_with_x[idx] += x as i16;

            counts_with_y[idx] += y as i16;
        }
    }
    (counts, counts_with_x, counts_with_y)
}

pub struct HachiFeatures {
    pub heights:[u32;10],
    pub height_differences:[i16;9],
    pub first_hole_depths:[i16;10],
    pub garbage_holes:[i16;20],
    pub piece_distance:[i16;7],
    pub piece_counts:[i16;7],
    pub hold_or_current_onehot:[i16;7],
    pub next_onehot:[i16;7],
    pub all_3x3s:[i16;512],
    pub all_3x3s_with_x:[i16;512],
    pub all_3x3s_with_y:[i16;512],
    pub all_2x2s:[i16;16],
    pub all_2x2s_with_x:[i16;16],
    pub all_2x2s_with_y:[i16;16],
    pub all_2x3s:[i16;64],
    pub all_2x3s_with_x:[i16;64],
    pub all_2x3s_with_y:[i16;64],
    pub all_3x2s:[i16;64],
    pub all_3x2s_with_x:[i16;64],
    pub all_3x2s_with_y:[i16;64],
    pub meter: i16,
    pub combo: i16,
    pub b2b: i16,
}

pub fn get_hachi_features(gamestate: &GameState) -> HachiFeatures {
    let mut board = gamestate.board;
    let (all_3x3s, all_3x3s_with_x, all_3x3s_with_y) = get_3x3s(&board);
    let (all_2x2s, all_2x2s_with_x, all_2x2s_with_y) = get_2x2s(&board);
    let (all_2x3s, all_2x3s_with_x, all_2x3s_with_y) = get_2x3s(&board);
    let (all_3x2s, all_3x2s_with_x, all_3x2s_with_y) = get_3x2s(&board);
    HachiFeatures {
        heights: get_heights(&board),
        height_differences: get_height_differences(&board),
        first_hole_depths: get_first_hole_depths(&board),
        garbage_holes: get_garbage_hole_sequence(&board),
        piece_distance: get_distance_to_next_piece(&gamestate),
        piece_counts: get_count_of_pieces(&gamestate),
        hold_or_current_onehot: get_hold_or_current_piece(&gamestate),
        next_onehot: get_next_piece(&gamestate),
        all_3x3s: all_3x3s,
        all_3x3s_with_x: all_3x3s_with_x,
        all_3x3s_with_y: all_3x3s_with_y,
        all_2x2s: all_2x2s,
        all_2x2s_with_x: all_2x2s_with_x,
        all_2x2s_with_y: all_2x2s_with_y,
        all_2x3s: all_2x3s,
        all_2x3s_with_x: all_2x3s_with_x,
        all_2x3s_with_y: all_2x3s_with_y,
        all_3x2s: all_3x2s,
        all_3x2s_with_x: all_3x2s_with_x,
        all_3x2s_with_y: all_3x2s_with_y,
        meter: gamestate.damage_received as i16,
        combo: gamestate.combo as i16,
        b2b: gamestate.b2b as i16,
    }
}