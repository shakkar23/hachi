use std::sync::OnceLock;

use features::feature_extractor::{extract_features, Features};

use lightgbm_rust::{predict_type, Booster};

const MODEL_PATH: &str = "models/td_model.txt";

const FEATURES_PER_ROW: usize = Features::count * 2;

struct SyncBooster(Booster);
// For predictions, booster should be threadsafe. 
// We Send + Sync to handle the underlying raw pointer.
unsafe impl Send for SyncBooster {}
unsafe impl Sync for SyncBooster {}

static BOOSTER: OnceLock<SyncBooster> = OnceLock::new();

fn booster() -> &'static Booster {
    &BOOSTER
        .get_or_init(|| {
            SyncBooster(Booster::load(MODEL_PATH).expect("failed load lightgbm model"))
        })
        .0
}

pub fn eval(f1: &Features, f2: &Features) -> f64 {
    let v1 = f1.values();
    let v2 = f2.values();

    debug_assert_eq!(v1.len(), Features::count);
    debug_assert_eq!(v2.len(), Features::count);

    let mut row = Vec::with_capacity(FEATURES_PER_ROW);
    row.extend(v1.into_iter().map(|x| x as f64));
    row.extend(v2.into_iter().map(|x| x as f64));

    let predictions = booster()
        .predict(&row, 1, FEATURES_PER_ROW as i32, predict_type::NORMAL)
        .expect("lightgbm prediction failed");

    predictions[0]
}

pub fn eval_batched(pairs: &[(Features, Features)]) -> Vec<f64> {
    if pairs.is_empty() {
        return Vec::new();
    }

    let num_rows = pairs.len();
    let mut data = Vec::with_capacity(num_rows * FEATURES_PER_ROW);

    for (f1, f2) in pairs {
        for v in f1.values() {
            data.push(v as f64);
        }
        for v in f2.values() {
            data.push(v as f64);
        }
    }

    booster()
        .predict(&data, num_rows as i32, FEATURES_PER_ROW as i32, predict_type::NORMAL)
        .expect("lightgbm prediction failed")
}