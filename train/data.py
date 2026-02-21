import sqlite3
import pandas as pd
from sklearn.model_selection import train_test_split

DATABASE_PATH = "./database.db"

conn = sqlite3.connect(DATABASE_PATH)

sql_query = "SELECT * FROM denormalized_data"

df = pd.read_sql_query(sql_query, conn)

conn.close()

df = df.drop(columns=[])

y = df['player1_score']

df = df.drop('player2_score', axis=1)
df = df.drop('player1_score', axis=1)
X = df

X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.1, random_state=42)

data = {
    "X_train": X_train,
    "X_test": X_test,
    "y_train": y_train,
    "y_test": y_test,
}