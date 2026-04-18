use features::feature_extractor::Features;
use lightgbm_rust::{predict_type, Booster};
use std::cell::RefCell;

const MODEL_PATH: &str = "models/td_model.txt";
const FEATURES_PER_ROW: usize = Features::COUNT * 2;

thread_local! {
    static BOOSTER: RefCell<Option<Booster>> = const { RefCell::new(None) };
}

fn with_booster<T>(f: impl FnOnce(&Booster) -> T) -> T {
    BOOSTER.with(|cell| {
        let mut opt = cell.borrow_mut();
        let booster = opt.get_or_insert_with(|| {
            Booster::load(MODEL_PATH).expect("failed to load lightgbm model")
        });
        f(booster)
    })
}

pub fn eval(f1: &Features, f2: &Features) -> f64 {
    eval_batched(&[(f1, f2)])[0]
}

pub fn eval_batched(pairs: &[(&Features, &Features)]) -> Vec<f64> {
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

    with_booster(|booster| {
        booster
            .predict(&data, num_rows as i32, FEATURES_PER_ROW as i32, predict_type::NORMAL)
            .expect("lightgbm prediction failed")
    })
}