# Hachi

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