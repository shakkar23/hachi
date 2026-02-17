use tetris::{board::Board, piece::Piece, piece::Rotation};
use rusqlite::{Connection, Result};
use core::panic;
use std::{env};


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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    state:State
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
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("please put in a db path for arg 1");
        return Ok(());
    }
    let conn = Connection::open(&args[1])?;
    
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
        state
        FROM Data")?;
    let person_iter = stmt.query_map([], |row| {
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
            state:to_state(&row.get::<_, String>(32)?).unwrap()
        })
    })?;

    for person in person_iter {
        println!("Found datum {:?}", person.unwrap());
    }
    Ok(())
}
