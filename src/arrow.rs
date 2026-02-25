use duckdb::arrow::array::{
    ArrayBuilder, ArrayRef, BooleanBuilder, Float32Builder, UInt16Builder, Int16Builder,
};
use duckdb::arrow::datatypes::{DataType, Field, Schema};
use duckdb::arrow::record_batch::RecordBatch;
use duckdb::arrow::error::ArrowError;

use std::sync::Arc;

use crate::feature_extractor::{Features, Row};

pub fn rows_to_record_batch(rows: &[Row]) -> Result<RecordBatch, ArrowError> {
    let n_rows = rows.len();

    let mut game_id_builder     = UInt16Builder::with_capacity(n_rows);
    let mut move_index_builder  = UInt16Builder::with_capacity(n_rows);
    let mut state_builder       = UInt16Builder::with_capacity(n_rows);
    let mut ground_truth_builder = Float32Builder::with_capacity(n_rows);

    let n_feat = Features::count;

    let mut feat0_builders: Vec<Int16Builder> = (0..n_feat)
        .map(|_| Int16Builder::with_capacity(n_rows))
        .collect();

    let mut feat1_builders: Vec<Int16Builder> = (0..n_feat)
        .map(|_| Int16Builder::with_capacity(n_rows))
        .collect();

    for row in rows {
        game_id_builder.append_value(row.game_id);
        move_index_builder.append_value(row.move_index);
        state_builder.append_value(row.state as u16);
        ground_truth_builder.append_value(row.ground_truth as f32);
        
        let values0 = row.features.0.values();
        let values1 = row.features.1.values();

        for i in 0..n_feat {
            feat0_builders[i].append_value(values0[i]);
            feat1_builders[i].append_value(values1[i]);
        }
    }

    let game_id     = Arc::new(game_id_builder.finish())     as ArrayRef;
    let move_index  = Arc::new(move_index_builder.finish())  as ArrayRef;
    let state  = Arc::new(state_builder.finish())  as ArrayRef;
    let ground_truth = Arc::new(ground_truth_builder.finish()) as ArrayRef;

    let mut feature_arrays: Vec<ArrayRef> = Vec::with_capacity(n_feat * 2);

    for mut b in feat0_builders {
        feature_arrays.push(Arc::new(b.finish()));
    }
    for mut b in feat1_builders {
        feature_arrays.push(Arc::new(b.finish()));
    }

    let mut fields = vec![
        Field::new("game_id", DataType::UInt16, false),
        Field::new("move_index", DataType::UInt16, false),
        Field::new("state", DataType::UInt16, false),
        Field::new("ground_truth", DataType::Float32, false),
    ];

    for i in 0..n_feat {
        fields.push(Field::new(format!("f0_{i}"), DataType::Int16, false));
    }

    for i in 0..n_feat {
        fields.push(Field::new(format!("f1_{i}"), DataType::Int16, false));
    }

    let schema = Arc::new(Schema::new(fields));

    let mut columns = vec![game_id, move_index, state, ground_truth];
    columns.extend(feature_arrays);

    RecordBatch::try_new(schema, columns)
}