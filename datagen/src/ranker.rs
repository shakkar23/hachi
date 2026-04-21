use tetris::{board::Board, piece::Piece, piece::Rotation};
use tetris::bag::Bag;
use tetris::state::{Lock, State as TState};

use bot::bot::{BotConfigs, BotState, BotError};
use bot::eval::Weights;

use std::{env, fs::exists};
use std::time::Instant;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use rayon::prelude::*;

use duckdb::arrow::array::{
    ArrayRef, BooleanBuilder, Float32Builder, Int16Builder, UInt8Builder,
    UInt16Builder, UInt64Builder,
};
use duckdb::arrow::datatypes::{DataType, Field, Schema};
use duckdb::arrow::record_batch::RecordBatch;
use duckdb::{Connection as DuckConnection, Result as DuckResult};

use rusqlite::Connection;

use features::feature_extractor::{extract_features, Features};
use features::game::{Datum, GameState, Move as GMove, State as GState};

use std::sync::Arc;

// big beam since this is datagen
// 2500 balances quality with not taking forever (100ms per board)
// still takes 16 hours on the full 333k dataset
const BEAM_WIDTH: usize = 2500;

// ---------------------------------------------------------------------------
// Input parsing (copied from main.rs)
// ---------------------------------------------------------------------------

fn to_piece(s: &str) -> Result<Piece, ()> {
    match s {
        "I" => Ok(Piece::I),
        "J" => Ok(Piece::J),
        "L" => Ok(Piece::L),
        "O" => Ok(Piece::O),
        "S" => Ok(Piece::S),
        "T" => Ok(Piece::T),
        "Z" => Ok(Piece::Z),
        _ => Err(()),
    }
}

fn to_rotation(s: i32) -> Result<Rotation, ()> {
    Ok(match s {
        0 => Rotation::North,
        1 => Rotation::East,
        2 => Rotation::South,
        3 => Rotation::West,
        _ => return Err(()),
    })
}

fn to_board(bytes: Vec<u8>) -> Board {
    let mut ret = Board::new();
    for x in 0..9 {
        for y in 0..20 {
            if bytes[x + y * 10] != 0 {
                ret.set(x as i8, y as i8);
            }
        }
    }
    ret
}

fn to_state(s: &str) -> Result<GState, ()> {
    Ok(match s {
        "PLAYING" => GState::PLAYING,
        "P1_WIN" => GState::P1_WIN,
        "P2_WIN" => GState::P2_WIN,
        "DRAW" => GState::DRAW,
        _ => return Err(()),
    })
}

fn extract_data(db_path: &str) -> Vec<Datum> {
    let conn = Connection::open(db_path).unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT
                p1_board, p1_current_piece, p1_move_piece_type, p1_move_piece_rot,
                p1_move_piece_x, p1_move_piece_y, p1_meter, p1_combo, p1_attack,
                p1_damage_received, p1_spun,
                p1_queue_0, p1_queue_1, p1_queue_2, p1_queue_3, p1_queue_4,
                p1_hold,
                p2_board, p2_current_piece, p2_move_piece_type, p2_move_piece_rot,
                p2_move_piece_x, p2_move_piece_y, p2_meter, p2_combo, p2_attack,
                p2_damage_received, p2_spun,
                p2_queue_0, p2_queue_1, p2_queue_2, p2_queue_3, p2_queue_4,
                p2_hold,
                state, game_id, move_index, p1_b2b, p2_b2b
             FROM Data ORDER BY game_id ASC, move_index ASC",
        )
        .unwrap();

    let iter = stmt
        .query_map([], |row| {
            Ok(Datum {
                p1: GameState {
                    board: to_board(row.get(0)?),
                    current_piece: to_piece(&row.get::<_, String>(1)?).unwrap(),
                    placement: GMove {
                        move_type: to_piece(&row.get::<_, String>(2)?).ok(),
                        rotation: to_rotation(row.get(3)?).unwrap(),
                        x: row.get(4)?,
                        y: row.get(5)?,
                    },
                    meter: row.get(6)?,
                    combo: row.get(7)?,
                    b2b: row.get(37)?,
                    attack: row.get(8)?,
                    damage_received: row.get(9)?,
                    spun: row.get::<_, i32>(10)? == 1,
                    queue: [
                        to_piece(&row.get::<_, String>(11)?).unwrap(),
                        to_piece(&row.get::<_, String>(12)?).unwrap(),
                        to_piece(&row.get::<_, String>(13)?).unwrap(),
                        to_piece(&row.get::<_, String>(14)?).unwrap(),
                        to_piece(&row.get::<_, String>(15)?).unwrap(),
                    ],
                    hold: to_piece(&row.get::<_, String>(16)?).ok(),
                },
                p2: GameState {
                    board: to_board(row.get(17)?),
                    current_piece: to_piece(&row.get::<_, String>(18)?).unwrap(),
                    placement: GMove {
                        move_type: to_piece(&row.get::<_, String>(19)?).ok(),
                        rotation: to_rotation(row.get(20)?).unwrap(),
                        x: row.get(21)?,
                        y: row.get(22)?,
                    },
                    meter: row.get(23)?,
                    combo: row.get(24)?,
                    b2b: row.get(38)?,
                    attack: row.get(25)?,
                    damage_received: row.get(26)?,
                    spun: row.get::<_, i32>(27)? == 1,
                    queue: [
                        to_piece(&row.get::<_, String>(28)?).unwrap(),
                        to_piece(&row.get::<_, String>(29)?).unwrap(),
                        to_piece(&row.get::<_, String>(30)?).unwrap(),
                        to_piece(&row.get::<_, String>(31)?).unwrap(),
                        to_piece(&row.get::<_, String>(32)?).unwrap(),
                    ],
                    hold: to_piece(&row.get::<_, String>(33)?).ok(),
                },
                state: to_state(&row.get::<_, String>(34)?).unwrap(),
                game_id: row.get(35)?,
                move_index: row.get(36)?,
            })
        })
        .unwrap();

    iter.map(|e| e.unwrap()).collect()
}

// ---------------------------------------------------------------------------
// Group ID: stable hash of the pre-move root state
// ---------------------------------------------------------------------------

fn group_id(gs: &GameState) -> u64 {
    let mut h = DefaultHasher::new();
    gs.board.hash(&mut h);
    gs.current_piece.hash(&mut h);
    gs.queue.hash(&mut h);
    gs.hold.hash(&mut h);
    gs.b2b.hash(&mut h);
    gs.combo.hash(&mut h);
    h.finish()
}

// ---------------------------------------------------------------------------
// Row produced per ranked candidate
// Includes static board features (Features) of the outcome board +
// non-static information such as whether we sent attack or sent a t-spin
// Exact move (x, y, etc) not a feature, but recoverable if needed
// ---------------------------------------------------------------------------

struct RankingRow {
    group_id: u64,
    rank: i16, // oracle ranking of this move
    game_id: u16, // group identifier
    move_index: u16, // can be used to retrieve exact move later 
    player: u8, // not important
    attack_sent: u8, // tells us if the move just made sent attack
    lines_cleared: u8, // see above
    was_tspin: bool, // if we did a t spin
    was_softdrop: bool, // if it was a softdrop (could be important for ppt)
    piece_placed: u8, // piece type
    features: Features,
}

// ---------------------------------------------------------------------------
// Per-player ranking: build BotState, run beam, simulate each candidate,
// extract features on the post-move GameState.
// ---------------------------------------------------------------------------

fn rank_player(gs: &GameState, game_id: u16, move_index: u16, player: u8) -> Vec<RankingRow> {
    // Bot queue = current piece + next 5. BotState wants queue.len() >= 2.
    let mut queue: Vec<Piece> = Vec::with_capacity(6);
    queue.push(gs.current_piece);
    queue.extend_from_slice(&gs.queue);

    let root = TState {
        board: gs.board,
        hold: gs.hold,
        bag: Bag::all(),
        next: 0,
        b2b: gs.b2b,
        combo: gs.combo,
    };

    let lock_seed = Lock {
        cleared: 0,
        sent: 0,
        softdrop: false,
    };

    let bot = match BotState::new(root.clone(), lock_seed, queue.clone(), Weights::default()) {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };

    let result = match bot.get_full_rankings(BotConfigs { width: BEAM_WIDTH }) {
        Ok(r) => r,
        Err(BotError::Death) | Err(BotError::InvalidQueue) => return Vec::new(),
    };

    let gid = group_id(gs);
    let mut rows = Vec::with_capacity(result.candidates.len());

    // candidates are sorted best-first by get_full_rankings
    for (i, (mv, _score)) in result.candidates.iter().enumerate() {
        // Simulate the move on a fresh clone to get the post-state + Lock.
        let mut sim_state = root.clone();
        let lock = sim_state.make(mv, &queue);
 
        // Post-move view needs current_piece + 5 next pieces = 6 total.
        // The bot's queue had 6 pieces and .make() consumed 1 or 2, so we're
        // always short 1 or 2 pieces. Pad the tail with uniform-random pieces.
        let remaining = &queue[sim_state.next..];

        debug_assert!(remaining.len() == 4);

        const ALL_PIECES: [Piece; 7] = [
            Piece::I, Piece::J, Piece::L, Piece::O, Piece::S, Piece::T, Piece::Z,
        ];
        let mut full = [Piece::I; 6];
        for j in 0..6 {
            full[j] = if j < remaining.len() {
                remaining[j]
            } else {
                ALL_PIECES[rand::random::<usize>() % 7]
            };
        }
        let new_current = full[0];
        let mut new_queue = [Piece::I; 5];
        new_queue.copy_from_slice(&full[1..6]);

        let post = GameState {
            board: sim_state.board,
            current_piece: new_current,
            placement: GMove {
                move_type: Some(mv.kind),
                rotation: mv.r,
                x: mv.x.max(0) as u8,
                y: mv.y.max(0) as u8,
            },
            meter: 0, // beam search doesn't use this (for now)
            combo: sim_state.combo,
            b2b: sim_state.b2b,
            attack: lock.sent,
            damage_received: gs.damage_received, // doesn't use this either
            spun: mv.tspin.is_some(),
            queue: new_queue,
            hold: sim_state.hold,
        };

        let feats = extract_features(&post);

        rows.push(RankingRow {
            group_id: gid,
            rank: (i + 1) as i16,
            game_id,
            move_index,
            player,
            attack_sent: lock.sent,
            lines_cleared: lock.cleared,
            was_tspin: mv.tspin.is_some(),
            was_softdrop: lock.softdrop,
            piece_placed: mv.kind as u8,
            features: feats,
        });
    }

    rows
}

// ---------------------------------------------------------------------------
// Arrow RecordBatch construction
// ---------------------------------------------------------------------------

fn rows_to_record_batch(rows: &[RankingRow]) -> RecordBatch {
    let n = rows.len();

    let mut group_id_b = UInt64Builder::with_capacity(n);
    let mut rank_b = Int16Builder::with_capacity(n);
    let mut game_id_b = UInt16Builder::with_capacity(n);
    let mut move_index_b = UInt16Builder::with_capacity(n);
    let mut player_b = UInt8Builder::with_capacity(n);
    let mut attack_sent_b = UInt8Builder::with_capacity(n);
    let mut lines_cleared_b = UInt8Builder::with_capacity(n);
    let mut was_tspin_b = BooleanBuilder::with_capacity(n);
    let mut was_softdrop_b = BooleanBuilder::with_capacity(n);
    let mut piece_placed_b = UInt8Builder::with_capacity(n);

    let n_feat = Features::COUNT;
    let mut feat_b: Vec<Int16Builder> = (0..n_feat)
        .map(|_| Int16Builder::with_capacity(n))
        .collect();

    for row in rows {
        group_id_b.append_value(row.group_id);
        rank_b.append_value(row.rank);
        game_id_b.append_value(row.game_id);
        move_index_b.append_value(row.move_index);
        player_b.append_value(row.player);
        attack_sent_b.append_value(row.attack_sent);
        lines_cleared_b.append_value(row.lines_cleared);
        was_tspin_b.append_value(row.was_tspin);
        was_softdrop_b.append_value(row.was_softdrop);
        piece_placed_b.append_value(row.piece_placed);

        let vals = row.features.values();
        for i in 0..n_feat {
            feat_b[i].append_value(vals[i]);
        }
    }

    let mut fields = vec![
        Field::new("group_id", DataType::UInt64, false),
        Field::new("rank", DataType::Int16, false),
        Field::new("game_id", DataType::UInt16, false),
        Field::new("move_index", DataType::UInt16, false),
        Field::new("player", DataType::UInt8, false),
        Field::new("attack_sent", DataType::UInt8, false),
        Field::new("lines_cleared", DataType::UInt8, false),
        Field::new("was_tspin", DataType::Boolean, false),
        Field::new("was_softdrop", DataType::Boolean, false),
        Field::new("piece_placed", DataType::UInt8, false),
    ];
    for i in 0..n_feat {
        fields.push(Field::new(format!("f_{i}"), DataType::Int16, false));
    }

    let mut columns: Vec<ArrayRef> = vec![
        Arc::new(group_id_b.finish()),
        Arc::new(rank_b.finish()),
        Arc::new(game_id_b.finish()),
        Arc::new(move_index_b.finish()),
        Arc::new(player_b.finish()),
        Arc::new(attack_sent_b.finish()),
        Arc::new(lines_cleared_b.finish()),
        Arc::new(was_tspin_b.finish()),
        Arc::new(was_softdrop_b.finish()),
        Arc::new(piece_placed_b.finish()),
    ];
    for mut b in feat_b {
        columns.push(Arc::new(b.finish()));
    }

    let schema = Arc::new(Schema::new(fields));
    RecordBatch::try_new(schema, columns).expect("record batch build failed")
}

// ---------------------------------------------------------------------------
// DuckDB table creation + append
// ---------------------------------------------------------------------------

fn create_and_write(rows: &[RankingRow], output_path: &str) -> DuckResult<()> {
    let conn = DuckConnection::open(output_path)?;
    conn.execute("DROP TABLE IF EXISTS move_rankings", [])?;

    let mut cols = String::from(
        "group_id UBIGINT NOT NULL,
         rank SMALLINT NOT NULL,
         game_id USMALLINT NOT NULL,
         move_index USMALLINT NOT NULL,
         player UTINYINT NOT NULL,
         attack_sent UTINYINT NOT NULL,
         lines_cleared UTINYINT NOT NULL,
         was_tspin BOOLEAN NOT NULL,
         was_softdrop BOOLEAN NOT NULL,
         piece_placed UTINYINT NOT NULL",
    );
    for i in 0..Features::COUNT {
        cols.push_str(&format!(", f_{i} SMALLINT NOT NULL"));
    }

    conn.execute(&format!("CREATE TABLE move_rankings ({})", cols), [])?;

    let batch = rows_to_record_batch(rows);
    let mut appender = conn.appender("move_rankings")?;
    appender.append_record_batch(batch)?;
    appender.flush()?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("usage: ranker <input_sqlite> <output_duckdb>");
        return;
    }
    if !exists(&args[1]).unwrap() {
        println!("input database not found: {}", args[1]);
        return;
    }

    println!("loading data from {}...", args[1]);
    let t0 = Instant::now();
    let data = extract_data(&args[1]);
    println!("loaded {} datums in {:.1}s", data.len(), t0.elapsed().as_secs_f64());

    // Filter to PLAYING only.
    let playing: Vec<&Datum> = data.iter().filter(|d| d.state == GState::PLAYING).collect();
    println!("{} datums in PLAYING state", playing.len());

    println!("running beam search (width={}) on both players...", BEAM_WIDTH);
    let t1 = Instant::now();

    let rows: Vec<RankingRow> = playing
        .par_iter()
        .flat_map(|d| {
            let mut r = rank_player(&d.p1, d.game_id, d.move_index, 0);
            r.extend(rank_player(&d.p2, d.game_id, d.move_index, 1));
            r
        })
        .collect();

    println!(
        "ranked {} candidate moves across {} positions in {:.1}s",
        rows.len(),
        playing.len() * 2,
        t1.elapsed().as_secs_f64()
    );

    println!("writing to {}...", args[2]);
    let t2 = Instant::now();
    if let Err(e) = create_and_write(&rows, &args[2]) {
        println!("duckdb error: {}", e);
        return;
    }
    println!("wrote in {:.1}s", t2.elapsed().as_secs_f64());
    println!("done. total: {:.1}s", t0.elapsed().as_secs_f64());
}