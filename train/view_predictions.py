import sqlite3
import duckdb
import numpy as np
import pandas as pd
from model import xgb_model

def get_feature_data():
    DATABASE_PATH = "./15.duckdb"

    conn = duckdb.connect(DATABASE_PATH)

    sql_query = "SELECT * FROM training_data LIMIT 1000"

    df = conn.execute(sql_query).fetchdf()

    df = df.drop(columns=[
        "game_id",
        "state",
        "move_index",
        "ground_truth"
    ])

    conn.close()

    return df

def get_raw_data():
    DATABASE_PATH = "./15.db"

    conn = sqlite3.connect(DATABASE_PATH)

    sql_query = "SELECT p1_board, p2_board FROM Data LIMIT 1000"

    df = pd.read_sql(sql_query, conn)

    conn.close()

    return df

def to_board(blob_data):
    """Convert BLOB data to 10x20 tetris board representation"""
    # Convert bytes to numpy array
    bytes_data = np.frombuffer(blob_data, dtype=np.uint8)
    
    # Create 10x20 board
    board = np.zeros((20, 10), dtype=bool)
    
    for x in range(10):
        for y in range(20):
            idx = x + y * 10
            if idx < len(bytes_data) and bytes_data[idx] != 0:
                board[y, x] = True
    
    return board

def print_board(board, title=""):
    """Print a tetris board to console"""
    if title:
        print(f"{title}:")
    
    # Print top border
    print("+" + "-" * 10 + "+")
    
    # Print board cells
    for y in range(20):
        line = "|"
        for x in range(10):
            if board[y, x]:
                line += "█"
            else:
                line += " "
        line += "|"
        print(line)
    
    # Print bottom border
    print("+" + "-" * 10 + "+")

def print_boards_side_by_side(board1, board2, prediction):
    """Print two boards side by side with prediction"""
    
    # Print headers
    print(f"{'P1 Board':^25} {'P2 Board':^25}")
    print("-" * 50)
    
    # Print boards side by side
    for y in reversed(range(20)):
        # Left board row
        line = "|"
        for x in range(10):
            if board1[y, x]:
                line += "█"
            else:
                line += " "
        line += "|"
        
        line += " " * 5  # Spacing between boards
        
        # Right board row
        line += "|"
        for x in range(10):
            if board2[y, x]:
                line += "█"
            else:
                line += " "
        line += "|"
        
        print(line)
    
    # Print bottom borders
    print("+" + "-" * 10 + "+" + " " * 5 + "+" + "-" * 10 + "+")
    
    # Print prediction
    print(f"\nPrediction: {prediction:.4f} ({"P1" if prediction > 0 else "P2"} winning)")
    print("=" * 50 + "\n")

def view_predictions():
    # Load the model
    xgb_model.load_model("models/td_model.ubj")
    
    # Get feature data for predictions
    fdf = get_feature_data()
    
    # Get raw board data
    rdf = get_raw_data()
    
    # Make predictions
    fdf['prediction'] = xgb_model.predict(fdf)
    
    # Step through each game state
    for idx in range(len(fdf)):
        print(f"\nGame State {idx + 1}/{len(fdf)}")
        print("-" * 50)
        
        # Get board data
        p1_board_blob = rdf.iloc[idx]['p1_board']
        p2_board_blob = rdf.iloc[idx]['p2_board']
        
        # Convert to board representations
        p1_board = to_board(p1_board_blob)
        p2_board = to_board(p2_board_blob)
        
        # Get prediction
        prediction = fdf.iloc[idx]['prediction']
        
        # Display boards side by side
        print_boards_side_by_side(p1_board, p2_board, prediction)
        
        # Wait for user input to continue
        if idx < len(fdf) - 1:
            input("Press Enter to see next game state...")

def view_predictions_simple():
    """Alternative version that just prints boards and predictions sequentially"""
    # Load the model
    xgb_model.load("models/td_model.ubj")
    
    # Get data
    fdf = get_feature_data()
    rdf = get_raw_data()
    
    # Make predictions
    fdf['prediction'] = xgb_model.predict(fdf)
    
    # Display each game state
    for idx in range(min(len(fdf), len(rdf))):
        print(f"\n{'='*60}")
        print(f"GAME STATE {idx + 1}")
        print(f"{'='*60}\n")
        
        # Convert and display P1 board
        p1_board = to_board(rdf.iloc[idx]['p1_board'])
        print_board(p1_board, "P1 BOARD")
        
        print("\n" + " " * 10 + "VS" + "\n")
        
        # Convert and display P2 board
        p2_board = to_board(rdf.iloc[idx]['p2_board'])
        print_board(p2_board, "P2 BOARD")
        
        # Display prediction
        prediction = fdf.iloc[idx]['prediction']
        print(f"\nPREDICTION: {prediction:.4f}")
        print(f"{'='*60}\n")

if __name__ == "__main__":
    # You can choose which visualization to use
    view_predictions()  # Side-by-side view
    # view_predictions_simple()  # Sequential view