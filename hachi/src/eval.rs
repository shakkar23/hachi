use features::feature_extractor::Features;

#[derive(Debug, Clone, Copy)]
pub enum ModelType {
    LightGBM_Large,
}

use lightgbm3::Booster;
use std::cell::RefCell;

const MODEL_PATH: &str = "../models/td_model.txt";
const FEATURES_PER_ROW: usize = Features::COUNT * 2;

thread_local! {
    static BOOSTER: RefCell<Option<Booster>> = const { RefCell::new(None) };
}

fn with_booster<T>(f: impl FnOnce(&Booster) -> T) -> T {
    BOOSTER.with(|cell| {
        let mut opt = cell.borrow_mut();
        let booster = opt.get_or_insert_with(|| {
            Booster::from_file(MODEL_PATH).expect("failed to load lightgbm model")
        });
        let expected = booster.num_features();
        f(booster)
    })
}

// scale from expectation of sink states {-1,1} to probability of winning [0,1].
pub fn scale(raw_eval: &f64) -> f64 {
    (raw_eval + 1.0) / 2.0
}

pub fn eval(f1: &Features, f2: &Features, config: ModelType) -> f64 {
    eval_batched(&[(f1, f2)], config)[0]
}

pub fn eval_batched(pairs: &[(&Features, &Features)], _config: ModelType) -> Vec<f64> {
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
        let result = booster
            .predict(&data, FEATURES_PER_ROW as i32, true)
            .expect("lightgbm prediction failed");
        result
    }).iter().map(|v| scale(v)).collect()
}

#[test]
fn eval_zeros_test() {
    let f1 = Features::default();
    let f2 = Features::default();
    let result = eval(&f1, &f2, ModelType::LightGBM_Large);
    println!("result: {}", result);
}