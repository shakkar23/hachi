import sqlite3
import pandas as pd 

DATABASE_PATH = "./database.db"

conn = sqlite3.connect('my_database.db')

sql_query = "SELECT * FROM training_data"

training_df = pd.read_sql_query(sql_query, conn)

conn.close()
