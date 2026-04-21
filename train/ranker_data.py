import duckdb
import pandas as pd
from sklearn.model_selection import GroupShuffleSplit
import numpy as np
import time

DATABASE_PATH = "./rankings.duckdb"

conn = duckdb.connect(DATABASE_PATH, read_only=True)

t = time.perf_counter()

df = conn.execute("SELECT * FROM move_rankings").fetchdf()

print(f"Loaded ranking data in {time.perf_counter() - t:.4f}s")

conn.close()

# Relevance = N - rank (rank 1 -> highest label, rank N -> 0).
group_sizes = df.groupby("group_id")["rank"].transform("size")
df["relevance"] = (group_sizes - df["rank"]).astype(np.int32)

# Group-aware split so candidates from the same position don't get mixed across train and test
gss = GroupShuffleSplit(n_splits=1, test_size=0.1, random_state=42)
(train_idx, test_idx), = gss.split(df, groups=df["group_id"])

train_df = df.iloc[train_idx].sort_values(["group_id", "rank"]).reset_index(drop=True)
test_df  = df.iloc[test_idx].sort_values(["group_id", "rank"]).reset_index(drop=True)

drop_cols = ["group_id", "rank", "relevance", "game_id", "move_index", "player"]

y_train = train_df["relevance"].values
y_test  = test_df["relevance"].values

X_train = train_df.drop(columns=drop_cols)
X_test  = test_df.drop(columns=drop_cols)

# Per-group candidate counts, in order. LightGBM's `group=` wants these.
group_train = train_df.groupby("group_id", sort=False).size().values
group_test  = test_df.groupby("group_id", sort=False).size().values

print(f"train: {len(X_train)} rows across {len(group_train)} groups")
print(f"test:  {len(X_test)} rows across {len(group_test)} groups")

data = {
    "X_train": X_train,
    "X_test": X_test,
    "y_train": y_train,
    "y_test": y_test,
    "group_train": group_train,
    "group_test": group_test,
}