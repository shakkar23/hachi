use features::game::{GameState};
use features::feature_extractor::{Features, extract_features};

use crate::state::{FullState, MacroState};

use tetris::state::{State, Lock};

pub fn light_eval(state: &State) -> i32 {
    0
}

pub fn lock_eval(lock: &Lock) -> i32 {
    0
}

pub fn heavy_eval(state: &MacroState) -> i32 {
    0
}