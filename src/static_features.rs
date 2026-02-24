use rusqlite::fallible_iterator::IteratorExt;
use tetris::board::Board;
use tetris::moves::Move;
use tetris::piece::{Piece,Rotation};

use crate::game::{GameState};

pub struct StaticFeatures {
    pub sunbeam_max_height:u32,
    pub sunbeam_bumpiness:i32,
    pub sunbeam_well_x:usize,
    pub sunbeam_well_depth:i32,
    pub sunbeam_max_donated_height:u32,
    pub sunbeam_n_donations:i32,
    pub sunbeam_t_clears:[i32;4],
    pub cc_holes:i32,
    pub cc_coveredness:i32,
    pub cc_row_transitions:i32
}

// Return the well's depth and the position of the well
pub fn sunbeam_well(board: &Board, heights: &[u32; 10]) -> (i32, usize) {
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

    (mask.count_ones() as i32, x)
}

pub fn sunbeam_bumpiness(heights: &[u32; 10], well_x: usize) -> i32 {
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

    bumpiness as i32
}

// Get the number of holes overground and underground
pub fn sunbeam_holes(board: &Board, heights: &[u32; 10], well_x: usize) -> (i32, i32) {
    let min_height = heights[well_x];

    let mut holes = 0;

    for i in 0..10 {
        holes += heights[i] - min_height - (board.cols[i] >> min_height).count_ones();
    }

    (holes as i32, min_height as i32)
}

// Find the highest tslot
pub fn sunbeam_tslot(board: &Board, heights: &[u32; 10]) -> Option<Move> {
    for x in 0..8 {
        if heights[x] > heights[x + 1] && heights[x] + 1 < heights[x + 2] {
            if ((board.cols[x] >> (heights[x] - 1)) & 0b111) == 0b001
                && ((board.cols[x + 1] >> (heights[x] - 1)) & 0b111) == 0b000
                && ((board.cols[x + 2] >> (heights[x] - 1)) & 0b111) == 0b101
            {
                return Some(Move {
                    x: x as i8 + 1,
                    y: heights[x] as i8,
                    r: Rotation::South,
                    kind: Piece::T,
                    tspin: None,
                });
            }
        }

        if heights[x + 2] > heights[x + 1] && heights[x + 2] + 1 < heights[x] {
            if ((board.cols[x] >> (heights[x + 2] - 1)) & 0b111) == 0b101
                && ((board.cols[x + 1] >> (heights[x + 2] - 1)) & 0b111) == 0b000
                && ((board.cols[x + 2] >> (heights[x + 2] - 1)) & 0b111) == 0b001
            {
                return Some(Move {
                    x: x as i8 + 1,
                    y: heights[x + 2] as i8,
                    r: Rotation::South,
                    kind: Piece::T,
                    tspin: None,
                });
            }
        }

        if heights[x + 1] >= 3
            && heights[x + 1] >= heights[x]
            && heights[x + 1] + 1 < heights[x + 2]
        {
            if ((board.cols[x] >> (heights[x + 1] - 3)) & 0b11000) == 0b00000
                && ((board.cols[x + 1] >> (heights[x + 1] - 3)) & 0b11110) == 0b00100
                && ((board.cols[x + 2] >> (heights[x + 1] - 3)) & 0b11111) == 0b10000
                && (board.has(x as i8 + 1, heights[x + 1] as i8 - 3)
                    || (!board.has(x as i8 + 1, heights[x + 1] as i8 - 3)
                        && board.has(x as i8 + 2, heights[x + 1] as i8 - 4)))
            {
                return Some(Move {
                    x: x as i8 + 2,
                    y: heights[x + 1] as i8 - 2,
                    r: Rotation::West,
                    kind: Piece::T,
                    tspin: None,
                });
            }
        }

        if heights[x + 1] >= 3
            && heights[x + 1] >= heights[x + 2]
            && heights[x + 1] + 1 < heights[x]
        {
            if ((board.cols[x] >> (heights[x + 1] - 3)) & 0b11111) == 0b10000
                && ((board.cols[x + 1] >> (heights[x + 1] - 3)) & 0b11110) == 0b00100
                && ((board.cols[x + 2] >> (heights[x + 1] - 3)) & 0b11000) == 0b00000
                && (board.has(x as i8 + 1, heights[x + 1] as i8 - 3)
                    || (!board.has(x as i8 + 1, heights[x + 1] as i8 - 3)
                        && board.has(x as i8, heights[x + 1] as i8 - 4)))
            {
                return Some(Move {
                    x: x as i8,
                    y: heights[x + 1] as i8 - 2,
                    r: Rotation::East,
                    kind: Piece::T,
                    tspin: None,
                });
            }
        }
    }

    None
}

pub fn sunbeam_donations(board: &mut Board, heights: &mut [u32; 10], depth: usize) -> ([i32; 4], i32) {
    let mut tslots = [0; 4];
    let mut donations = 0;

    for _ in 0..depth {
        if let Some(tslot) = sunbeam_tslot(board, heights) {
            let mut clone = board.clone();

            clone.place(&tslot);

            let clear = clone.clear_lines();

            tslots[clear as usize] += 1;

            if clear >= 2 {
                *board = clone;
                *heights = board.heights();

                donations += 1;
            } else {
                break;
            }
        }
    }

    (tslots, donations)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_donations() {
        let mut board = Board::new();
        board.cols[0] = 0b111;
        board.cols[1] = 0b101;
        board.cols[2] = 0b000;
        board.cols[3] = 0b001;
        board.cols[4] = 0b111;
        board.cols[5] = 0b111;
        board.cols[6] = 0b111;
        board.cols[7] = 0b111;
        board.cols[8] = 0b111;
        board.cols[9] = 0b111;
        let mut heights = board.heights();
        assert_eq!(sunbeam_donations(&mut board, &mut heights, 2).0 ,[0,0,1,0]);
    }
}

pub fn cc_count_holes(board: &Board, heights: &[u32; 10]) -> i32 {
    let mut holes = 0i32;

    for x in 0..10 {
        let h = heights[x];
        if h == 0 {
            continue;
        }
        let underneath = (1u64 << h) - 1;
        let empty_bits = (!board.cols[x]) & underneath;
        holes += empty_bits.count_ones() as i32;
    }

    holes
}

pub fn cc_coveredness(board: &Board) -> i32 {
    let mut coveredness = 0;
    for &c in &board.cols {
        let height = 64 - c.leading_zeros();
        let underneath = (1 << height) - 1;
        let mut holes = !c & underneath;
        while holes != 0 {
            let y = holes.trailing_zeros();
            coveredness += (height - y) as i32;
            holes &= !(1 << y);
        }
    }

    coveredness
}

pub fn cc_row_transitions(board: &Board) -> i32 {
    let mut row_transitions = 0;
    row_transitions += (!0 ^ board.cols[0]).count_ones();
    row_transitions += (!0 ^ board.cols[9]).count_ones();
    for cs in board.cols.windows(2) {
        row_transitions += (cs[0] ^ cs[1]).count_ones();
    }

    row_transitions as i32
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