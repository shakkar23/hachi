use std::sync::OnceLock;

use crate::state::MacroState;

use features::feature_extractor::{extract_features, Features};

use xgboost_rust::{Booster, XGBoostResult};

const MODEL_PATH: &str = "models/td_model.json";

const FEATURES_PER_ROW: usize = Features::count * 2;

static BOOSTER: OnceLock<Booster> = OnceLock::new();

fn booster() -> &'static Booster {
    BOOSTER.get_or_init(|| {
        Booster::load(MODEL_PATH).expect("failed to load xgboost model")
    })
}

/// Flatten a MacroState into a row of f32 features (p1 then p2).
fn macro_state_to_row(state: &MacroState) -> Vec<f32> {
    let f1 = extract_features(&state.p1.state);
    let f2 = extract_features(&state.p2.state);

    let v1 = f1.values();
    let v2 = f2.values();

    debug_assert_eq!(v1.len(), Features::count);
    debug_assert_eq!(v2.len(), Features::count);

    let mut row = Vec::with_capacity(FEATURES_PER_ROW);
    row.extend(v1.into_iter().map(|x| x as f32));
    row.extend(v2.into_iter().map(|x| x as f32));
    row
}

pub fn heavy_eval(state: &MacroState) -> f64 {
    let row = macro_state_to_row(state);

    let predictions = booster()
        .predict(&row, 1, FEATURES_PER_ROW, 0, false)
        .expect("xgboost prediction failed");

    predictions[0] as f64
}

pub fn heavy_eval_batched(states: &Vec<MacroState>) -> Vec<f64> {
    if states.is_empty() {
        return Vec::new();
    }

    let num_rows = states.len();
    let mut data = Vec::with_capacity(num_rows * FEATURES_PER_ROW);

    for state in states {
        let f1 = extract_features(&state.p1.state);
        let f2 = extract_features(&state.p2.state);

        for v in f1.values() {
            data.push(v as f32);
        }
        for v in f2.values() {
            data.push(v as f32);
        }
    }

    debug_assert_eq!(data.len(), num_rows * FEATURES_PER_ROW);

    let predictions = booster()
        .predict(&data, num_rows, FEATURES_PER_ROW, 0, false)
        .expect("xgboost prediction failed");

    predictions.into_iter().map(|p| p as f64).collect()
}