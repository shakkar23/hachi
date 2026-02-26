use rusqlite::fallible_iterator::IteratorExt;
use tetris::board::Board;
use tetris::moves::Move;
use tetris::piece::{Piece,Rotation};

use crate::game::{GameState};

pub struct StaticFeatures {
    pub sunbeam_max_height:u32,
    pub sunbeam_bumpiness:i16,
    pub sunbeam_well_x:usize,
    pub sunbeam_well_depth:i16,
    pub sunbeam_max_donated_height:u32,
    pub sunbeam_n_donations:i16,
    pub sunbeam_t_clears:[i16;4],
    pub cc_holes:i16,
    pub cc_coveredness:i16,
    pub cc_row_transitions:i16
}

// Return the well's depth and the position of the well
pub fn sunbeam_well(board: &Board, heights: &[u32; 10]) -> (i16, usize) {
    let mut x = 0;

    for i in 1..10 {
        if heights[i] < heights[x] {
            x = i;
        }
    }

    let mut mask = u64::MAX;

    for i in 0..10 {
        if i == x {
            continue;
        }

        mask &= board.cols[i];
    }

    mask >>= heights[x];

    (mask.count_ones() as i16, x)
}

pub fn sunbeam_bumpiness(heights: &[u32; 10], well_x: usize) -> i16 {
    let mut bumpiness = 0;
    let mut left = 0;

    if well_x == 0 {
        left = 1;
    }

    for i in 1..10 {
        if i == well_x {
            continue;
        }

        let diff = heights[left].abs_diff(heights[i]);

        bumpiness += diff * diff;
        left = i;
    }

    bumpiness as i16
}

// Get the number of holes overground and underground
pub fn sunbeam_holes(board: &Board, heights: &[u32; 10], well_x: usize) -> (i16, i16) {
    let min_height = heights[well_x];

    let mut holes = 0;

    for i in 0..10 {
        holes += heights[i] - min_height - (board.cols[i] >> min_height).count_ones();
    }

    (holes as i16, min_height as i16)
}

pub fn sunbeam_donations(board: &mut Board, heights: &mut [u32; 10], depth: usize) -> ([i16; 4], i16) {
    ([0i16;4], 0i16)
}

pub fn cc_count_holes(board: &Board, heights: &[u32; 10]) -> i16 {
    let mut holes = 0i16;

    for x in 0..10 {
        let h = heights[x];
        if h == 0 {
            continue;
        }
        let underneath = (1u64 << h) - 1;
        let empty_bits = (!board.cols[x]) & underneath;
        holes += empty_bits.count_ones() as i16;
    }

    holes
}

pub fn cc_coveredness(board: &Board) -> i16 {
    let mut coveredness = 0;
    for &c in &board.cols {
        let height = 64 - c.leading_zeros();
        let underneath = (1 << height) - 1;
        let mut holes = !c & underneath;
        while holes != 0 {
            let y = holes.trailing_zeros();
            coveredness += (height - y) as i16;
            holes &= !(1 << y);
        }
    }

    coveredness
}

pub fn cc_row_transitions(board: &Board) -> i16 {
    let mut row_transitions = 0;
    row_transitions += (!0 ^ board.cols[0]).count_ones();
    row_transitions += (!0 ^ board.cols[9]).count_ones();
    for cs in board.cols.windows(2) {
        row_transitions += (cs[0] ^ cs[1]).count_ones();
    }

    row_transitions as i16
}

pub fn get_static_features(game:&GameState) -> StaticFeatures {
    let board = game.board;
    let sunbeam_heights = board.heights();
    let (sunbeam_well, sunbeam_well_x_pos) = sunbeam_well(&board, &sunbeam_heights);
    let mut board_copy = board;
    let mut heights_copy = sunbeam_heights;
    let (sunbeam_t_slots, sunbeam_donations) = sunbeam_donations(&mut board_copy, &mut heights_copy, 2);
    let cc_holes = cc_count_holes(&board, &sunbeam_heights);
    let cc_coveredness = cc_coveredness(&board);
    let cc_row_transitions = cc_row_transitions(&board);

    StaticFeatures {
        sunbeam_max_height:*sunbeam_heights.iter().max().unwrap(),
        sunbeam_bumpiness: sunbeam_bumpiness(&sunbeam_heights, sunbeam_well_x_pos),
        sunbeam_well_x: sunbeam_well_x_pos,
        sunbeam_well_depth:sunbeam_well,
        sunbeam_max_donated_height:*heights_copy.iter().max().unwrap(),
        sunbeam_n_donations:sunbeam_donations,
        sunbeam_t_clears:sunbeam_t_slots,
        cc_holes:cc_holes,
        cc_coveredness:cc_coveredness,
        cc_row_transitions:cc_row_transitions
    }
} 