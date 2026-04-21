use tetris::{
    board::Board,
    piece::{Piece, Rotation},
};
use features::{
    game::{GameState, Move},
};

use hachi::{
    hachi::{solve_position, HachiConfig}
};

use features::feature_extractor::Features;
use features::feature_extractor::extract_features;

use hachi::eval::{eval, eval_batched, ModelType};
use hachi::solver::{nash_equilibrium, nash_equilibrium_exact};

fn make_good_state() -> GameState {
    GameState {
        board: Board {
            cols: [
                0b00000000011111111111,  // col 0
                0b00000000011111111111,  // col 1
                0b00000000011111111111,  // col 2
                0b00000000011111111111,  // col 3
                0b00000000001111111111,  // col 4
                0b00000000000000000000,  // col 5 (the well — only bottom + row 6)
                0b00000000001111111111,  // col 6
                0b00000000011111111111,  // col 7
                0b00000000011111111111,  // col 8
                0b00000000011111111111,  // col 9 (bottom row empty — tetris-ready)
            ],
        },
        current_piece: Piece::Z,
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
        queue: [Piece::O, Piece::L, Piece::T, Piece::I, Piece::I],
        hold: None,
    }
}


fn make_bad_state() -> GameState {
    GameState {
        board: Board {
            cols: [
                0b00000010000001100000,  // col 0: rows 5,6 only (the "███████ ██" overhang pair)
                0b00000001111111111111,  // col 1
                0b00011111111111111111,  // col 2 (tallest — 17 rows up to the tip)
                0b00001111111111111111,  // col 3
                0b00000111111111111111,  // col 4
                0b00000011111111111111,  // col 5
                0b00000011111111111111,  // col 6
                0b00000111111110011111,  // col 7 (gap at rows 5,6)
                0b00000111111111111111,  // col 8
                0b00000111111111111111,  // col 9
            ],
        },
        current_piece: Piece::T,
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
        queue: [Piece::I, Piece::J, Piece::L, Piece::S, Piece::T],
        hold: None,
    }
}

#[test]
fn test_losing_position() {
    let state1 = make_bad_state();
    let state2 = make_good_state();

    let features1 = extract_features(&state1);
    let features2 = extract_features(&state2);

    // should be about 0.3
    let losing_eval = eval(&features1, &features2, ModelType::LightGBM_Large);
    println!("[EVAL] position value (losing): {}", losing_eval);

    // should be about 0.7
    let winning_eval = eval(&features2, &features1, ModelType::LightGBM_Large);
    println!("[EVAL] position value (winning): {}", winning_eval);

    for depth in 1..=6 {
        let (_, losing) = solve_position(state1, state2, depth, HachiConfig::rapid());
        println!("[DEPTH {}] position value (losing): {}", depth, losing);

        let (_, winning) = solve_position(state2, state1, depth, HachiConfig::rapid());
        println!("[DEPTH {}] position value (winning): {}", depth, winning);
    }
}

use tetris::{
    moves::Move as _Move,
    state::State,
    bag::Bag,
};

use hachi::hachi::{gamestate_to_state, get_pruned_moves, u64_to_piece};

#[test]
fn test_self_play_game() {
    use std::time::{SystemTime, UNIX_EPOCH};

    // xorshift RNG seeded from time
    let mut rng: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64 | 1;
    let mut next = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    // Start with two empty boards
    let mut state2 = GameState::default(); //make_bad_state();
    let mut state1 = GameState::default(); //make_good_state();

     for queue in [&mut state1.queue, &mut state2.queue] {
        for i in (1..queue.len()).rev() {
            let j = (next() as usize) % (i + 1);
            queue.swap(i, j);
        }
    }
    
    let config1 = HachiConfig::rapid();
    let config2 = HachiConfig::beam();
    
    for turn in 0..100 {
        println!("\n=== TURN {} ===", turn);
        println!("{}", TwoBoards(&state1.board, &state2.board));
        println!("P1 meter: {}, P2 meter: {}", state1.meter, state2.meter);
        
        // Player 1 moves
        let (mv1, eval1) = solve_position(state1, state2, 2, config1);
        println!("P1 eval: {}", eval1);

        let tetris_state1 = gamestate_to_state(&state1);
        let mut lock1 = apply_move_to_gamestate(&mut state1, &mv1);
        println!("P1 clears: {}", lock1.cleared);
        
        // Player 2 moves
        let (mv2, eval2) = solve_position(state2, state1, 1, config2);
        println!("P2 eval: {}", eval2);
        
        // Apply P2's move
        let mut lock2 = apply_move_to_gamestate(&mut state2, &mv2);
        println!("P2 clears: {}", lock2.cleared);
        
        // Generate attacks
        let mut new_meter = state1.meter.saturating_sub(lock1.sent);
        state1.attack = lock1.sent - state1.meter.abs_diff(new_meter);
        state1.meter = new_meter;
        
        new_meter = state2.meter.saturating_sub(lock2.sent);
        state2.attack = lock2.sent - state2.meter.abs_diff(new_meter);
        state2.meter = new_meter;
        
        // Apply pending garbage
        if state1.meter > 0 && state1.combo == 0 {
            state1.tank_garbage(state1.meter as u32, 0);
            println!("P1 tanks {} garbage", state1.meter);
            state1.meter = 0;
        }
        if state2.meter > 0 && state2.combo == 0 {
            state2.tank_garbage(state2.meter as u32, 0);
            println!("P2 tanks {} garbage", state2.meter);
            state2.meter = 0;
        }

        // calculate trade
        if state1.attack > state2.attack {
            state1.attack -= state2.attack;
            state2.attack = 0;
        } else {
            state2.attack -= state1.attack;
            state2.attack = 0;
        }

        // Add attacks to meter
        if state1.attack > 0 {
            state2.meter += state1.attack as u8;
            state1.attack = 0;
            println!("P1 sends {} damage", lock1.sent);
        }
        if state2.attack > 0 {
            state1.meter += state2.attack as u8;
            state2.attack = 0;
            println!("P2 sends {} damage", lock2.sent);
        }
        
        // Check for game over
        if !has_valid_moves(&state1) {
            println!("\n=== GAME OVER: Player 2 wins! ===");
            break;
        }
        if !has_valid_moves(&state2) {
            println!("\n=== GAME OVER: Player 1 wins! ===");
            break;
        }
    }
    
    println!("\n=== FINAL BOARDS ===");
    println!("Player 1:\n{}", state1.board);
    println!("Player 2:\n{}", state2.board);
}

fn apply_move_to_gamestate(gamestate: &mut GameState, mv: &_Move) -> tetris::state::Lock {
    let mut state = gamestate_to_state(gamestate);
    let queue = gamestate.queue;
    
    let lock = state.make(mv, &queue);
    
    // Update gamestate from the resulting state
    gamestate.board = state.board;
    gamestate.b2b = state.b2b;
    gamestate.combo = state.combo as u8;
    gamestate.hold = state.hold;

    gamestate.queue.rotate_left(1);
    gamestate.queue[4] = u64_to_piece(gamestate.board.cols.iter().sum::<u64>() % 7);
    
    lock
}

fn has_valid_moves(state: &GameState) -> bool {
    let tetris_state = gamestate_to_state(state);
    let moves = get_pruned_moves(&tetris_state, &state.queue, 1);
    !moves.is_empty()
}

#[test]
fn test_tank_garbage() {
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

pub struct TwoBoards<'a>(pub &'a Board, pub &'a Board);

impl<'a> std::fmt::Display for TwoBoards<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in (0..20).rev() {
            for x in 0..10 {
                write!(f, "{}", if self.0.has(x, y) { "██" } else { "  " })?;
            }
            write!(f, "  ")?;
            for x in 0..10 {
                write!(f, "{}", if self.1.has(x, y) { "██" } else { "  " })?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}