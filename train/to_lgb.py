from data import state, df
from model import big_model, small_lgb_model
import lightgbm as lgb
import numpy as np
import pandas as pd
from sklearn.metrics import mean_squared_error, r2_score
from sklearn.model_selection import train_test_split
from perf import bench

def train():
    global big_model, small_lgb_model, df
    
    df = df.copy()
    
    # Load parent model
    big_model.load_model("models/td_model.ubj")
    
    df['prediction'] = big_model.predict(df)

    X = df.drop('prediction', axis=1)
    y = df['prediction']
  
    X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.1, random_state=42)

    # 3. Refit model
    small_lgb_model.fit(X_train, y_train)

    y_pred = small_lgb_model.predict(X_test)

    # Evaluate
    mse = mean_squared_error(y_test, y_pred)
    r2 = r2_score(y_test, y_pred)
    
    print(f"Mean Squared Error: {mse:.4f}")
    print(f"R² Score:       {r2:.4f}\n")
    
    small_lgb_model.booster_.save_model("models/small_lgb_model.txt")
    print("Training completed. Final model saved as models/small_lgb_model.txt")

if __name__ == "__main__":
    train()
    bench(small_lgb_model)