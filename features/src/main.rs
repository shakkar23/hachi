use tetris::{board::Board, piece::Piece, piece::Rotation};
use std::{env, fs::exists};
use rayon::prelude::*;
use std::time::Instant;
use std::panic;

use duckdb::arrow::record_batch::RecordBatch;

use rusqlite::{Connection, Result};
use duckdb::{Connection as DuckConnection, Result as DuckResult, Error as DuckError};

use features::feature_extractor::{Features, Row};

use features::arrow::rows_to_record_batch;

use features::game::{GameState,Move,Datum,State};

fn to_piece(s:&str) -> Result<Piece, ()> {
    match s {
        "I" => Ok(Piece::I),
        "J" => Ok(Piece::J),
        "L" => Ok(Piece::L),
        "O" => Ok(Piece::O),
        "S" => Ok(Piece::S),
        "T" => Ok(Piece::T),
        "Z" => Ok(Piece::Z),
        _ =>   Err(())
    }
}
fn to_rotation(s:i32) -> Result<Rotation, ()> {
    return Ok(match s {
        0 => Rotation::North,
        1 => Rotation::East,
        2 => Rotation::South,
        3 => Rotation::West,
        _ => return Err(())
    })
}

fn to_board(bytes:Vec<u8>) -> Board {
    let mut ret = Board::new();

    for x in 0..9 {
        for y in 0..20 {
            let set = bytes[x + y * 10];
            if set != 0 {
                ret.set(x as i8, y as i8);
            }
        }
    }
    ret
}

fn to_state(s:&str) -> Result<State, ()> {
    Ok(match s {
        "PLAYING" => State::PLAYING,
        "P1_WIN" => State::P1_WIN,
        "P2_WIN" => State::P2_WIN,
        "DRAW" => State::DRAW,
        _ => return Err(())
    })
}

fn to_death_value(s:&State) -> Result<f32, ()> {
    Ok(match s {
        State::PLAYING => 0f32,
        State::P1_WIN => 1f32,
        State::P2_WIN => -1f32,
        State::DRAW => 0f32,
        _ => return Err(())
    })
}

/*
const char* sql =
		"CREATE TABLE IF NOT EXISTS Data ("
		"game_id INTEGER NOT NULL, "
		"move_index INTEGER NOT NULL, "
		"state TEXT NOT NULL, "
		"p1_board BLOB NOT NULL, p1_current_piece TEXT NOT NULL, p1_move_piece_type TEXT NOT NULL, "
		"p1_move_piece_rot INTEGER NOT NULL, p1_move_piece_x INTEGER NOT NULL, p1_move_piece_y INTEGER NOT NULL, "
		"p1_meter INTEGER NOT NULL, p1_attack INTEGER NOT NULL, p1_damage_received INTEGER NOT NULL, "
		"p1_spun INTEGER NOT NULL, "
		"p1_queue_0 TEXT NOT NULL, p1_queue_1 TEXT NOT NULL, p1_queue_2 TEXT NOT NULL, p1_queue_3 TEXT NOT NULL, p1_queue_4 TEXT NOT NULL, "
		"p1_hold TEXT NOT NULL, "
		"p2_board BLOB NOT NULL, p2_current_piece TEXT NOT NULL, p2_move_piece_type TEXT NOT NULL, "
		"p2_move_piece_rot INTEGER NOT NULL, p2_move_piece_x INTEGER NOT NULL, p2_move_piece_y INTEGER NOT NULL, "
		"p2_meter INTEGER NOT NULL, p2_attack INTEGER NOT NULL, p2_damage_received INTEGER NOT NULL, "
		"p2_spun INTEGER NOT NULL, "
		"p2_queue_0 TEXT NOT NULL, p2_queue_1 TEXT NOT NULL, p2_queue_2 TEXT NOT NULL, p2_queue_3 TEXT NOT NULL, p2_queue_4 TEXT NOT NULL, "
		"p2_hold TEXT NOT NULL"
		");";
*/

fn extract_data(db_path:String) -> Vec<Datum> {

    let conn = Connection::open(&db_path).unwrap();
    
    let mut stmt = conn.prepare("SELECT 
        p1_board,
        p1_current_piece,
        p1_move_piece_type,
        p1_move_piece_rot,
        p1_move_piece_x,
        p1_move_piece_y,
        p1_meter,
        p1_combo,
        p1_attack,
        p1_damage_received,
        p1_spun,
        p1_queue_0,
        p1_queue_1,
        p1_queue_2,
        p1_queue_3,
        p1_queue_4,
        p1_hold,
        p2_board,
        p2_current_piece,
        p2_move_piece_type,
        p2_move_piece_rot,
        p2_move_piece_x,
        p2_move_piece_y,
        p2_meter,
        p2_combo,
        p2_attack,
        p2_damage_received,
        p2_spun,
        p2_queue_0,
        p2_queue_1,
        p2_queue_2,
        p2_queue_3,
        p2_queue_4,
        p2_hold,
        state,
        game_id,
        move_index,
        p1_b2b,
        p2_b2b
        FROM Data ORDER BY game_id ASC, move_index ASC").unwrap();
    let data_iter = stmt.query_map([], |row| {
        Ok(Datum{
            p1:GameState {
                board: to_board(row.get(0)?),
                current_piece: to_piece(&row.get::<_, String>(1)?).unwrap(),
                
                placement:Move{
                    move_type:to_piece(&row.get::<_, String>(2)?).ok(),
                    rotation: to_rotation(row.get(3)?).unwrap(),
                    x:row.get(4)?,
                    y:row.get(5)?
                },
                meter:row.get(6)?,
                combo:row.get(7)?,
                b2b:row.get(37)?,
                attack:row.get(8)?,
                damage_received:row.get(9)?,
                spun:row.get::<_, i32>(10)? == 1,
                queue:[
                    to_piece(&row.get::<_, String>(11)?).unwrap(),
                    to_piece(&row.get::<_, String>(12)?).unwrap(),
                    to_piece(&row.get::<_, String>(13)?).unwrap(),
                    to_piece(&row.get::<_, String>(14)?).unwrap(),
                    to_piece(&row.get::<_, String>(15)?).unwrap(),
                ],
                hold:to_piece(&row.get::<_, String>(16)?).ok()
            },
            p2:GameState {
                board: to_board(row.get(17)?),
                current_piece: to_piece(&row.get::<_, String>(18)?).unwrap(),
                
                placement:Move{
                    move_type:to_piece(&row.get::<_, String>(19)?).ok(),
                    rotation: to_rotation(row.get(20)?).unwrap(),
                    x:row.get(21)?,
                    y:row.get(22)?
                },
                meter:row.get(23)?,
                combo:row.get(24)?,
                b2b:row.get(38)?,
                attack:row.get(25)?,
                damage_received:row.get(26)?,
                spun:row.get::<_, i32>(27)? == 1,
                queue:[
                    to_piece(&row.get::<_, String>(28)?).unwrap(),
                    to_piece(&row.get::<_, String>(29)?).unwrap(),
                    to_piece(&row.get::<_, String>(30)?).unwrap(),
                    to_piece(&row.get::<_, String>(31)?).unwrap(),
                    to_piece(&row.get::<_, String>(32)?).unwrap(),
                ],
                hold:to_piece(&row.get::<_, String>(33)?).ok()
            },
            state:to_state(&row.get::<_, String>(34)?).unwrap(),
            game_id: row.get(35)?,
            move_index: row.get(36)?
        })
    }).unwrap();
    
    return data_iter.map(|e|e.unwrap()).collect();
}

fn create_dataset(data: &[Datum], output_db_path: &str) -> DuckResult<()> {
    let start = Instant::now();

    let conn = DuckConnection::open(output_db_path)?;

    conn.execute("DROP TABLE IF EXISTS training_data", [])?;

    conn.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS training_data (
                game_id       INTEGER NOT NULL,
                move_index    INTEGER NOT NULL,
                state         SMALLINT NOT NULL,
                ground_truth  REAL NOT NULL,
                {},
                {},
                PRIMARY KEY (game_id, move_index)
            )",
            Features::sql_columns_with_types("p1"),
            Features::sql_columns_with_types("p2"),
        ),
        [],
    )?;

    let mut rows: Vec<Row> = data.par_iter()
        .map(|d| {
            let p1_attrs = features::feature_extractor::extract_features(&d.p1);
            let p2_attrs = features::feature_extractor::extract_features(&d.p2);

            Row {
                features: (p1_attrs, p2_attrs),
                state: d.state,
                game_id: d.game_id,
                move_index: d.move_index,
                ground_truth: to_death_value(&d.state).unwrap(),
            }
        })
        .collect();

    let mut loss = 1f32;
    for row in rows.iter_mut().rev() {
        if row.ground_truth != 0f32 {
            loss = row.ground_truth;
        } else {
            row.ground_truth = (50f32 / 60f32) * loss;
            loss = row.ground_truth;
        }
    }

    let mut duration = start.elapsed().as_secs_f64();

    println!(
        "Feature extraction took {:.1}s",
        duration
    );
    
    let mut record_batch = rows_to_record_batch(&rows).unwrap();

    duration = start.elapsed().as_secs_f64();
    
    println!(
        "Record Batch preparation took {:.1}s",
        duration
    );

    let mut appender = conn.appender("training_data")?;

    appender.append_record_batch(record_batch)?;
    appender.flush()?;

    duration = start.elapsed().as_secs_f64();

    println!(
        "Wrote {} training records to {} in {:.1}s",
        data.len(),
        output_db_path,
        duration
    );

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("Please provide an input database path.");
        return;
    }
    if args.len() == 2 {
        println!("Please provide an output database path.");
        return;
    }
    if !exists(args[1].to_string()).unwrap() {
        println!("Input database not found.");
        return;
    }
    let data = extract_data(args[1].to_string());
    if let Err(e) = create_dataset(&data, &args[2].to_string()) {
        println!("Error creating dataset: {}", e);
    }
}
