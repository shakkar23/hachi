use tetris::{board::Board, piece::Piece, piece::Rotation};
use rusqlite::{Connection, Result};
use std::{env, fs::exists};
use itertools::izip;

mod attribute_finder;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Move {
    pub move_type:Option<Piece>,
    pub rotation:Rotation,
    pub x:u8,
    pub y:u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct GameState {
    pub board: Board,
    pub current_piece:Piece,
    pub placement:Move,
    pub meter:u8,
    pub attack:u8,
    pub damage_received:u8,
    pub spun:bool,
    pub queue:[Piece;5],
    pub hold:Option<Piece>
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
enum State {
    PLAYING,
    P1_WIN,
    P2_WIN,
    DRAW
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Datum {
    p1:GameState,
    p2:GameState,
    state:State,
    game_id:u16,
    move_index:u16
}


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
        State::P1_WIN => -1f32,
        State::P2_WIN => 1f32,
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
        move_index
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
                attack:row.get(7)?,
                damage_received:row.get(8)?,
                spun:row.get::<_, i32>(9)? == 1,
                queue:[
                    to_piece(&row.get::<_, String>(10)?).unwrap(),
                    to_piece(&row.get::<_, String>(11)?).unwrap(),
                    to_piece(&row.get::<_, String>(12)?).unwrap(),
                    to_piece(&row.get::<_, String>(13)?).unwrap(),
                    to_piece(&row.get::<_, String>(14)?).unwrap()
                ],
                hold:to_piece(&row.get::<_, String>(15)?).ok()
            },
            p2:GameState {
                board: to_board(row.get(16)?),
                current_piece: to_piece(&row.get::<_, String>(17)?).unwrap(),
                
                placement:Move{
                    move_type:to_piece(&row.get::<_, String>(18)?).ok(),
                    rotation: to_rotation(row.get(19)?).unwrap(),
                    x:row.get(20)?,
                    y:row.get(21)?
                },
                meter:row.get(22)?,
                attack:row.get(23)?,
                damage_received:row.get(24)?,
                spun:row.get::<_, i32>(25)? == 1,
                queue:[
                    to_piece(&row.get::<_, String>(26)?).unwrap(),
                    to_piece(&row.get::<_, String>(27)?).unwrap(),
                    to_piece(&row.get::<_, String>(28)?).unwrap(),
                    to_piece(&row.get::<_, String>(29)?).unwrap(),
                    to_piece(&row.get::<_, String>(30)?).unwrap()
                ],
                hold:to_piece(&row.get::<_, String>(31)?).ok()
            },
            state:to_state(&row.get::<_, String>(32)?).unwrap(),
            game_id: row.get(33)?,
            move_index: row.get(34)?
        })
    }).unwrap();
    
    return data_iter.map(|e|e.unwrap()).collect();
}

fn create_dataset(data: &[Datum], output_db_path: &str) -> Result<(), rusqlite::Error> {
    let mut conn = Connection::open(output_db_path)?;
    
    let tx = conn.transaction()?;
    
    // Create the training data table
    tx.execute(
        "CREATE TABLE IF NOT EXISTS training_data (
            game_id INTEGER NOT NULL,
            move_index INTEGER NOT NULL,
            state TEXT NOT NULL,
            ground_truth REAL NOT NULL,
            -- P1 features
            p1_bumpiness REAL NOT NULL,
            p1_n_donations REAL NOT NULL,
            p1_well_depth REAL NOT NULL,
            p1_max_donated_height REAL NOT NULL,
            p1_max_height REAL NOT NULL,
            p1_t_clear_0 REAL NOT NULL,
            p1_t_clear_1 REAL NOT NULL,
            p1_t_clear_2 REAL NOT NULL,
            p1_t_clear_3 REAL NOT NULL,
            p1_well_x REAL NOT NULL,
            -- P2 features
            p2_bumpiness REAL NOT NULL,
            p2_n_donations REAL NOT NULL,
            p2_well_depth REAL NOT NULL,
            p2_max_donated_height REAL NOT NULL,
            p2_max_height REAL NOT NULL,
            p2_t_clear_0 REAL NOT NULL,
            p2_t_clear_1 REAL NOT NULL,
            p2_t_clear_2 REAL NOT NULL,
            p2_t_clear_3 REAL NOT NULL,
            p2_well_x REAL NOT NULL,
            PRIMARY KEY (game_id, move_index)
        )",
        [],
    )?;

    {
        // Prepare the insert statement
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO training_data (
                game_id, move_index, state, ground_truth,
                p1_bumpiness, p1_n_donations, p1_well_depth, p1_max_donated_height,
                p1_max_height, p1_t_clear_0, p1_t_clear_1, p1_t_clear_2, p1_t_clear_3,
                p1_well_x,
                p2_bumpiness, p2_n_donations, p2_well_depth, p2_max_donated_height,
                p2_max_height, p2_t_clear_0, p2_t_clear_1, p2_t_clear_2, p2_t_clear_3,
                p2_well_x
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )?;

        let mut training_data = Vec::new();
        let mut states = Vec::new();
        let mut game_ids = Vec::new();
        let mut move_indexes = Vec::new();
        let mut ground_truths = Vec::new();
        
        // First pass: collect features and compute initial ground truths
        for d in data {
            let p1_attrs = attribute_finder::get_attributes(d.p1.board);
            let p2_attrs = attribute_finder::get_attributes(d.p2.board);

            training_data.push((
                p1_attrs.sunbeam_bumpiness as f32,
                p1_attrs.sunbeam_n_donations as f32,
                p1_attrs.sunbeam_well_depth as f32,
                p1_attrs.sunbeam_max_donated_height as f32,
                p1_attrs.sunbeam_max_height as f32,
                p1_attrs.sunbeam_t_clears[0] as f32,
                p1_attrs.sunbeam_t_clears[1] as f32,
                p1_attrs.sunbeam_t_clears[2] as f32,
                p1_attrs.sunbeam_t_clears[3] as f32,
                p1_attrs.sunbeam_well_x as f32,
                p2_attrs.sunbeam_bumpiness as f32,
                p2_attrs.sunbeam_n_donations as f32,
                p2_attrs.sunbeam_well_depth as f32,
                p2_attrs.sunbeam_max_donated_height as f32,
                p2_attrs.sunbeam_max_height as f32,
                p2_attrs.sunbeam_t_clears[0] as f32,
                p2_attrs.sunbeam_t_clears[1] as f32,
                p2_attrs.sunbeam_t_clears[2] as f32,
                p2_attrs.sunbeam_t_clears[3] as f32,
                p2_attrs.sunbeam_well_x as f32
            ));
            
            ground_truths.push(to_death_value(&d.state).unwrap());
            states.push(d.state);
            game_ids.push(d.game_id);
            move_indexes.push(d.move_index);
        }

        let mut loss = 1f32;
        for gt in ground_truths.iter_mut().rev() {
            if *gt != 0f32 {
                loss = *gt;
            } else {
                *gt = (55f32/60f32) * loss;
                loss = *gt;
            }
        }

        // Insert all rows into the database
        for i in 0..data.len() {
            let feats = training_data[i];
            
            stmt.execute(rusqlite::params![
                game_ids[i],
                move_indexes[i],
                format!("{:?}", states[i]),  // Convert enum to string
                ground_truths[i],
                // P1 features
                feats.0, feats.1, feats.2, feats.3, feats.4,
                feats.5, feats.6, feats.7, feats.8, feats.9,
                // P2 features
                feats.10, feats.11, feats.12, feats.13, feats.14, 
                feats.15, feats.16, feats.17, feats.18, feats.19
            ])?;
        }

    }

    tx.commit()?;

    println!("Wrote {} training records to {}", data.len(), output_db_path);
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("please put in a db path for arg 1");
        return;
    }
    if !exists(args[1].to_string()).unwrap() {
        println!("your file does not exist!");
        return;
    }
    let data = extract_data(args[1].to_string());
    if let Err(e) = create_dataset(&data, &args[1].to_string()) {
        println!("Error creating dataset: {}", e);
    }
}
