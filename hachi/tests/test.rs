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

use tetris::{
    moves::Move as _Move,
    state::State,
    bag::Bag,
};

use hachi::hachi::{gamestate_to_state, get_pruned_moves, u64_to_piece, drain_tree_dump};

fn make_good_state() -> GameState {
    GameState {
        board: Board {
            cols: [
                0b00000000011111111111,
                0b00000000011111111111,
                0b00000000011111111111,
                0b00000000011111111111,
                0b00000000001111111111,
                0b00000000000000000000,
                0b00000000001111111111,
                0b00000000011111111111,
                0b00000000011111111111,
                0b00000000011111111111,
            ],
        },
        current_piece: Piece::Z,
        placement: Move { move_type: None, rotation: Rotation::North, x: 0, y: 0 },
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
                0b00000010000001100000,
                0b00000001111111111111,
                0b00011111111111111111,
                0b00001111111111111111,
                0b00000111111111111111,
                0b00000011111111111111,
                0b00000011111111111111,
                0b00000111111110011111,
                0b00000111111111111111,
                0b00000111111111111111,
            ],
        },
        current_piece: Piece::T,
        placement: Move { move_type: None, rotation: Rotation::North, x: 0, y: 0 },
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

fn make_ok_state1() -> GameState {
    GameState {
        board: Board {
            cols: [
                0b00000000011111111111,
                0b00000000011111111111,
                0b00000000011111111111,
                0b00000000011111110111,
                0b00000000001111111111,
                0b00000000000000001000,
                0b00000000001111111111,
                0b00000000011111111111,
                0b00000000111111111111,
                0b00000000111111111111,
            ],
        },
        current_piece: Piece::Z,
        placement: Move { move_type: None, rotation: Rotation::North, x: 0, y: 0 },
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

fn make_ok_state2() -> GameState {
    GameState {
        board: Board {
            cols: [
                0b00000000011111111111,
                0b00000000011111111111,
                0b00000000000001110111,
                0b00000000011111111111,
                0b00000000001111111111,
                0b00000000000000001100,
                0b00000000101111111111,
                0b00000000111111111011,
                0b00000000111111111111,
                0b00000000111111111111,
            ],
        },
        current_piece: Piece::I,
        placement: Move { move_type: None, rotation: Rotation::North, x: 0, y: 0 },
        meter: 0,
        combo: 0,
        attack: 0,
        b2b: 0,
        damage_received: 0,
        spun: false,
        queue: [Piece::I, Piece::J, Piece::Z, Piece::S, Piece::T],
        hold: None,
    }
}

#[test]
fn test_losing_position() {
    let state1 = make_bad_state();
    let state2 = make_good_state();

    let features1 = extract_features(&state1);
    let features2 = extract_features(&state2);

    let losing_eval = eval(&features1, &features2, ModelType::LightGBM_Large);
    println!("[EVAL] position value (losing): {}", losing_eval);

    let winning_eval = eval(&features2, &features1, ModelType::LightGBM_Large);
    println!("[EVAL] position value (winning): {}", winning_eval);

    for depth in 1..=6 {
        let (_, losing) = solve_position(state1, state2, depth, HachiConfig::rapid());
        println!("[DEPTH {}] position value (losing): {}", depth, losing);

        let (_, winning) = solve_position(state2, state1, depth, HachiConfig::rapid());
        println!("[DEPTH {}] position value (winning): {}", depth, winning);
    }
}


#[test]
fn generate_dump() {
    let mut state1 = make_ok_state1();
    let mut state2 = make_ok_state2();
    
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    
    let mut rng: u64 = seed | 1;
    let mut next = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    
    for queue in [&mut state1.queue, &mut state2.queue] {
        for i in (1..queue.len()).rev() {
            let j = (next() as usize) % (i + 1);
            queue.swap(i, j);
        }
    }

    let (_, _) = solve_position(state1, state2, 5, HachiConfig::rapid());
    std::fs::write("dump.json", drain_tree_dump()).unwrap();
}

enum GameOutcome {
    Win,
    Draw,
    Loss
}

fn self_play(seed: u64, config1: HachiConfig, config2: HachiConfig, depth: usize) -> GameOutcome {
    let mut rng: u64 = seed | 1;
    let mut next = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };

    let mut state1 = GameState::default();
    let mut state2 = GameState::default();

    for queue in [&mut state1.queue, &mut state2.queue] {
        for i in (1..queue.len()).rev() {
            let j = (next() as usize) % (i + 1);
            queue.swap(i, j);
        }
    }

    let mut turn = 0;

    let mut ret = GameOutcome::Draw;

    for turn in 0..500 {
        //println!("\n=== TURN {} ===", turn);
        println!("{}", TwoBoards(&state1.board, &state2.board));
        //println!("P1 meter: {}, P2 meter: {}", state1.meter, state2.meter);

        let (mv1, eval1) = solve_position(state1, state2, depth, config1);
        let mut lock1 = apply_move_to_gamestate(&mut state1, &mv1);
        //println!("P1 clears: {}", lock1.cleared);

        let (mv2, eval2) = solve_position(state2, state1, depth, config2);
        println!("P1 eval: {}", eval1);
        //println!("P2 eval: {}", eval2);
        let mut lock2 = apply_move_to_gamestate(&mut state2, &mv2);
        //println!("P2 clears: {}", lock2.cleared);

        let mut new_meter = state1.meter.saturating_sub(lock1.sent);
        state1.attack = lock1.sent - state1.meter.abs_diff(new_meter);
        state1.meter = new_meter;

        new_meter = state2.meter.saturating_sub(lock2.sent);
        state2.attack = lock2.sent - state2.meter.abs_diff(new_meter);
        state2.meter = new_meter;

        if state1.meter > 0 && state1.combo == 0 {
            state1.tank_garbage(state1.meter as u32, next() as usize % 10);
            //println!("P1 tanks {} garbage", state1.meter);
            state1.meter = 0;
        }
        if state2.meter > 0 && state2.combo == 0 {
            state2.tank_garbage(state2.meter as u32, next() as usize % 10);
            //println!("P2 tanks {} garbage", state2.meter);
            state2.meter = 0;
        }

        if state1.attack > state2.attack {
            state1.attack -= state2.attack;
            state1.attack = 0;
        } else {
            state2.attack -= state1.attack;
            state1.attack = 0;
        }

        if state1.attack > 0 {
            state2.meter += state1.attack as u8;
            state1.attack = 0;
            //println!("P1 sends {} damage", lock1.sent);
        }
        if state2.attack > 0 {
            state1.meter += state2.attack as u8;
            state2.attack = 0;
            //println!("P2 sends {} damage", lock2.sent);
        }

        if !has_valid_moves(&state1) {
            println!("\n=== GAME OVER: Player 2 wins! ===");
            ret = GameOutcome::Loss;
            break;
        }
        if !has_valid_moves(&state2) {
            println!("\n=== GAME OVER: Player 1 wins! ===");
            ret = GameOutcome::Win;
            break;
        }
    }


    println!("\n=== FINAL BOARDS ===");
    println!("Player 1:\n{}", state1.board);
    println!("Player 2:\n{}", state2.board);
    ret
}

#[test]
fn test_play_self() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    self_play(seed, HachiConfig::rapid(), HachiConfig::rapid(), 4);
}

#[test]
fn test_play_sunbeam() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    self_play(seed, HachiConfig::rapid(), HachiConfig::beam(), 2);
}

#[test]
fn test_sunbeam_winrate() {
    use std::time::{SystemTime, UNIX_EPOCH};

    const NUM_GAMES: usize = 100;
    let base_seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    let mut wins = 0;
    let mut draws = 0;
    let mut losses = 0;

    for i in 0..NUM_GAMES {
        let seed = base_seed.wrapping_add(i as u64);
        match self_play(seed, HachiConfig::beam(), HachiConfig::rapid(), 2) {
            GameOutcome::Win => wins += 1,
            GameOutcome::Draw => draws += 1,
            GameOutcome::Loss => losses += 1,
        }
    }

    let n = NUM_GAMES as f64;
    let winrate = wins as f64 / n;
    let drawrate = draws as f64 / n;
    let lossrate = losses as f64 / n;

    // Wilson-ish SE for winrate
    let se = (winrate * (1.0 - winrate) / n).sqrt();

    println!("\n=== SUNBEAM WINRATE ({} games) ===", NUM_GAMES);
    println!("P1 (hachi) wins:   {} ({:.1}%)", wins, winrate * 100.0);
    println!("Draws:             {} ({:.1}%)", draws, drawrate * 100.0);
    println!("P2 (sunbeam) wins:    {} ({:.1}%)", losses, lossrate * 100.0);
    println!("P1 winrate: {:.3} ± {:.3} (1σ)", winrate, se);
}

fn apply_move_to_gamestate(gamestate: &mut GameState, mv: &_Move) -> tetris::state::Lock {
    let mut state = gamestate_to_state(gamestate);
    let queue = gamestate.queue;

    let lock = state.make(mv, &queue);

    gamestate.board = state.board;
    gamestate.b2b = state.b2b;
    gamestate.combo = state.combo as u8;
    gamestate.hold = state.hold;

    gamestate.queue.rotate_left(1);
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    gamestate.queue[4] = u64_to_piece(seed % 7);

    lock
}

fn has_valid_moves(state: &GameState) -> bool {
    let tetris_state = gamestate_to_state(state);
    let moves = get_pruned_moves(&tetris_state, &state.queue, 1, 1);
    !moves.is_empty()
}

#[test]
fn test_tank_garbage() {
    let mut game = GameState::default();
    game.board = Board { cols: [1u64; 10] };

    game.tank_garbage(3, 7);

    for i in 0..10 {
        if i == 7 {
            assert_eq!(game.board.cols[i], 1 << 3);
        } else {
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