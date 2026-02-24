import duckdb
import pandas as pd
from sklearn.model_selection import train_test_split
import time

DATABASE_PATH = "./training.duckdb"

conn = duckdb.connect(DATABASE_PATH)

sql_query = "SELECT * FROM training_data"

t = time.perf_counter()

df = conn.execute(sql_query).fetchdf()

print(f"Loaded training data in {time.perf_counter() - t:.4f}s")

conn.close()

df = df.drop(columns=[
    "game_id",
    "move_index",
])

print(df)

y = df['ground_truth']

df = df.drop('ground_truth', axis=1)

X = df


X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.1, random_state=42)

data = {
    "X_train": X_train,
    "X_test": X_test,
    "y_train": y_train,
    "y_test": y_test,
}