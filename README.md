# Hachi

## C++ Board Features

Link the `bot_features` CMake target and include `feature_extractor.hpp`:

```cpp
const auto features = moe::extract_features(board);
const std::array<float, moe::BoardFeatures::COUNT> inputs = features.values();
const auto& names = moe::BoardFeatures::names(); // Same order as inputs.
```

The 37 raw inputs are 10 column heights, 9 signed adjacent height differences,
10 first-hole depths, then maximum height, aggregate height, hole count,
coveredness, row transitions, bumpiness, well column, and well depth. All 32
board rows are included, with y=0 at the floor. First-hole depth is the number
of filled cells from the surface to the first empty cell below it (zero for a
column with no holes). Coveredness sums `height - y` for each hole.

As in the Rust static features, the well is the leftmost shortest column,
well depth counts rows above its height filled in all other columns, and
bumpiness sums squared height differences between columns after skipping the
well. Row transitions count changes between adjacent cells, including filled
side walls, over the C++ board's 32 rows. These are board-only inputs; expert
search scores are not exposed. No normalization is applied, so any scaling
used during PSO must also be applied during inference.

Run the feature checks with:

```sh
cmake -S . -B build
cmake --build build --target feature_extractor_test
./build/bots/feature_extractor_test
```

### Install Dependencies
```
pip install -r requirements.txt
```
### Run Feature Extractor
```
cargo run -p features --release -- .\database.db .\training.duckdb
```
### Train Base Model
```
python ./train/train.py
```
### Refine using TD-Lambda
```
python ./train/td_lambda.py
```
### Distill Child Model
```
python ./train/distill.py
```

## Analysis Tools

### View win probability predictions on test data
```
python ./train/view_predictions.py
```

### View most important features
```
python ./train/importance.py
cat importances.txt
```

### View most significant 3x3 XOR filters
```
python ./train/visualise.py
```

### Benchmark predictions per second
```
python ./train/perf.py
```
