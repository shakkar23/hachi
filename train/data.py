import sqlite3
import pandas as pd
from sklearn.model_selection import train_test_split

DATABASE_PATH = "./database.db"

conn = sqlite3.connect(DATABASE_PATH)

sql_query = "SELECT * FROM training_data"

df = pd.read_sql_query(sql_query, conn)

conn.close()

"""
CREATE TABLE training_data (
    game_id INTEGER NOT NULL,
    move_index INTEGER NOT NULL,
    state TEXT NOT NULL,
    ground_truth REAL NOT NULL,
    -- P1 features
    p1_bumpiness REAL NOT NULL,
    p1_n_donations REAL NOT NULL,
    p1_well_depth REAL NOT NULL,
    p1_max_donated_height REAL NOT NULL,
    p1_max_height REAL NOT NULL,
    p1_t_clear_0 REAL NOT NULL,
    p1_t_clear_1 REAL NOT NULL,
    p1_t_clear_2 REAL NOT NULL,
    p1_t_clear_3 REAL NOT NULL,
    p1_well_x REAL NOT NULL,
    -- P2 features
    p2_bumpiness REAL NOT NULL,
    p2_n_donations REAL NOT NULL,
    p2_well_depth REAL NOT NULL,
    p2_max_donated_height REAL NOT NULL,
    p2_max_height REAL NOT NULL,
    p2_t_clear_0 REAL NOT NULL,
    p2_t_clear_1 REAL NOT NULL,
    p2_t_clear_2 REAL NOT NULL,
    p2_t_clear_3 REAL NOT NULL,
    p2_well_x REAL NOT NULL,
    PRIMARY KEY (game_id, move_index)
);
"""

df = df.drop(columns=[
    "game_id",
    "move_index",
    "state",
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