
mod eval;
mod state;
mod solver;
mod hachi;
mod table;

use tetris::{board::Board, piece::{Piece, Rotation}};
use state::{MacroState};
use features::game::{GameState};
use json::{JsonValue};
use features::game::{Move};
use tetris::moves::{Tspin};
use hachi::solve_position;

use crate::hachi::HachiConfig;
fn sbp_piece_to_piece(s:String) -> Piece {
    match s.as_str() {
        "S" => Piece::S,
        "Z" => Piece::Z,
        "L" => Piece::L,
        "J" => Piece::J,
        "T" => Piece::T,
        "I" => Piece::I,
        "O" => Piece::O,
        _ => unreachable!()
    }
}
fn sbp_board_to_board(board_json:&JsonValue) -> Board {
    let mut b = Board::new();
    for x in 0..board_json.len() {
        for y in 0..board_json["0"].len() {
            let filled_cell = !board_json[x][y].is_null();
            if filled_cell {
                b.set(x as i8, y as i8);
            }
        }    
    }
    b
}
fn piece_to_str(p:Piece) -> &'static str {
    match p {
        Piece::S => "S",
        Piece::Z => "Z",
        Piece::L => "L",
        Piece::J => "J",
        Piece::T => "T",
        Piece::I => "I",
        Piece::O => "O",
        _ => unreachable!()
    }
}
fn rotation_to_str(r:Rotation) -> &'static str {
    match r {
        Rotation::North => "north",
        Rotation::East => "east",
        Rotation::South => "south",
        Rotation::West => "west",
        _ => unreachable!()
    }
}

fn spin_to_str(s:Tspin) -> &'static str {
    match s {
        Tspin::Full => "full",
        Tspin::Mini => "mini",
        _ => unreachable!()
    }
}
fn main() {
    println!("{}", r#"{"type": "info","version": "v3","author": "Awyzza + Shakkar","features": ["SBP"]}"#);
    // rules message
    {
        let mut buffer:String = String::new();
        let _ = std::io::stdin().read_line(&mut buffer);
        // let rules_json = json::parse(&buffer);
    }
    // ready immediately
    println!("{}", r#"{"type": "ready"}"#);
    
    while true {
        let mut buffer:String = String::new();
        let _ = std::io::stdin().read_line(&mut buffer);
        let frontend_message = json::parse(&buffer).unwrap_or(
            json::parse(r#"{"type": "quit"}"#).unwrap()
        );
        let message_type = frontend_message["type"].clone();
        match message_type.to_string().as_str() {
            "quit" => {return;},
            "play" => {
                let mut state = MacroState{
                    p1: GameState {
                        board: sbp_board_to_board(&frontend_message["board"]),
                        current_piece: sbp_piece_to_piece(frontend_message["queue"][0].to_string()),
                        placement: Move {
                            move_type:None,
                            rotation:tetris::piece::Rotation::North,
                            x:0,
                            y:0
                        },
                        meter: frontend_message["meter"].as_u8().unwrap(),
                        combo: frontend_message["combo"].as_u8().unwrap(),
                        attack: 0,
                        b2b: frontend_message["back_to_back"].as_u8().unwrap(),
                        damage_received:0,
                        spun:false,
                        queue:[
                            sbp_piece_to_piece(frontend_message["queue"][1].to_string()),
                            sbp_piece_to_piece(frontend_message["queue"][2].to_string()),
                            sbp_piece_to_piece(frontend_message["queue"][3].to_string()),
                            sbp_piece_to_piece(frontend_message["queue"][4].to_string()),
                            sbp_piece_to_piece(frontend_message["queue"][5].to_string()),
                            ],
                        hold: if !frontend_message["hold"].is_null() {
                                Some(sbp_piece_to_piece(frontend_message["hold"].to_string()))
                            } else {None},
                    },
                    p2: GameState {
                        board: sbp_board_to_board(&frontend_message["opponents"][0]["board"]),
                        current_piece: sbp_piece_to_piece(frontend_message["opponents"][0]["queue"][0].to_string()),
                        placement: Move {
                            move_type:None,
                            rotation:tetris::piece::Rotation::North,
                            x:0,
                            y:0
                        },
                        meter: frontend_message["opponents"][0]["meter"].as_u8().unwrap(),
                        combo: frontend_message["opponents"][0]["combo"].as_u8().unwrap(),
                        attack: 0,
                        b2b: frontend_message["opponents"][0]["back_to_back"].as_u8().unwrap(),
                        damage_received:0,
                        spun:false,
                        queue:[
                            sbp_piece_to_piece(frontend_message["opponents"][0]["queue"][1].to_string()),
                            sbp_piece_to_piece(frontend_message["opponents"][0]["queue"][2].to_string()),
                            sbp_piece_to_piece(frontend_message["opponents"][0]["queue"][3].to_string()),
                            sbp_piece_to_piece(frontend_message["opponents"][0]["queue"][4].to_string()),
                            sbp_piece_to_piece(frontend_message["opponents"][0]["queue"][5].to_string()),
                            ],
                        hold: if !frontend_message["opponents"][0]["hold"].is_null() {
                                Some(sbp_piece_to_piece(frontend_message["opponents"][0]["hold"].to_string()))
                            } else {None},
                    }
                };
                let hachi_move = solve_position(state.p1, state.p2, 3, HachiConfig::rapid());
                let mut move_json = JsonValue::new_object();
                move_json["location"]["type"] = piece_to_str(hachi_move.0.kind).into();
                move_json["location"]["orientation"] = rotation_to_str(hachi_move.0.r).into();
                move_json["location"]["x"] = hachi_move.0.x.into();
                move_json["location"]["y"] = hachi_move.0.y.into();

                move_json["spin"] = if hachi_move.0.tspin.is_some() 
                    { spin_to_str(hachi_move.0.tspin.unwrap()).into() } else 
                    { "none".into() };
                let mut moves_output = JsonValue::new_object();
                moves_output["moves"] = JsonValue::new_array();
                let _ = moves_output["moves"].push(move_json);
                println!("{}", moves_output.to_string());
            }
            _ => {return;}
        }
    }

    return;
}