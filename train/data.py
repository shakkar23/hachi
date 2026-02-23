import sqlite3
import pandas as pd
from sklearn.model_selection import train_test_split

DATABASE_PATH = "./database.db"

conn = sqlite3.connect(DATABASE_PATH)

sql_query = "SELECT * FROM training_data"

df = pd.read_sql_query(sql_query, conn)

conn.close()

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