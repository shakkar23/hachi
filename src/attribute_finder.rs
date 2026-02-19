use rusqlite::fallible_iterator::IteratorExt;
use tetris::board::Board;
use tetris::moves::Move;
use tetris::piece::{Piece,Rotation};

// copied from https://github.com/citrus610/sunbeam/blob/main/bot/src/eval.rs
#[derive(Debug, Clone, Copy)]
pub struct Weights {
    pub height: i32,
    pub well: i32,
    pub center: i32,
    pub bumpiness: i32,
    pub holes: i32,
    pub garbage: i32,
    pub tslot: [i32; 4],
    pub b2b_bonus: i32,
    pub combo_bonus: i32,

    pub clear: [i32; 4],
    pub tspin: [i32; 3],
    pub tspin_mini: [i32; 2],
    pub combo: [i32; 5],
    pub b2b: i32,
    pub pc: i32,
    pub waste_t: i32,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            height: -50,
            well: 25,
            center: -100,
            bumpiness: -25,
            holes: -400,
            garbage: -300,
            tslot: [150, 200, 250, 500],
            b2b_bonus: 200,
            combo_bonus: 200,

            clear: [-400, -350, -300, 250],
            tspin: [50, 400, 800],
            tspin_mini: [0, 0],
            combo: [200, 500, 1000, 1500, 2000],
            b2b: 100,
            pc: 2000,
            waste_t: -100,
        }
    }
}

// Return the well's depth and the position of the well
fn well(board: &Board, heights: &[u32; 10]) -> (i32, usize) {
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

fn bumpiness(heights: &[u32; 10], well_x: usize) -> i32 {
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
fn holes(board: &Board, heights: &[u32; 10], well_x: usize) -> (i32, i32) {
    let min_height = heights[well_x];

    let mut holes = 0;

    for i in 0..10 {
        holes += heights[i] - min_height - (board.cols[i] >> min_height).count_ones();
    }

    (holes as i32, min_height as i32)
}

// Find the highest tslot
fn tslot(board: &Board, heights: &[u32; 10]) -> Option<Move> {
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

fn donations(board: &mut Board, heights: &mut [u32; 10], depth: usize) -> ([i32; 4], i32) {
    let mut tslots = [0; 4];
    let mut donations = 0;

    for _ in 0..depth {
        if let Some(tslot) = tslot(board, heights) {
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

pub struct StaticAttributes {
    citrus_max_height:u32,
    citrus_bumpiness:i32,
    citrus_well_x:usize,
    citrus_well_depth:i32,
    citrus_max_donated_height:u32,
    citrus_n_donations:i32,
    citrus_t_clears:[i32;4],
}

pub fn get_attributes(board:Board) -> StaticAttributes {
    let citrus_heights = board.heights();
    let (citrus_well, citrus_well_x_pos) = well(&board, &citrus_heights);
    let mut donated_board = board;
    let mut donated_heights = citrus_heights;
    let (citrus_t_slots, citrus_donations) = donations(&mut donated_board, &mut donated_heights, 2);
    StaticAttributes {
        citrus_max_height:*citrus_heights.iter().max().unwrap(),
        citrus_bumpiness: bumpiness(&citrus_heights, citrus_well_x_pos),
        citrus_well_x: citrus_well_x_pos,
        citrus_well_depth:citrus_well,
        citrus_max_donated_height:*donated_heights.iter().max().unwrap(),
        citrus_n_donations:citrus_donations,
        citrus_t_clears:citrus_t_slots,
    }
}