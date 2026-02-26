from data import state, df
from model import xgb_model, big_model
import numpy as np
import pandas as pd
from sklearn.metrics import mean_squared_error, r2_score
from sklearn.model_selection import train_test_split
from perf import bench

def train():
    global xgb_model, big_model, df
    
    df = df.copy()
    
    # Load parent model
    xgb_model.load_model("models/td_model.ubj")
    
    df['prediction'] = xgb_model.predict(df)

    X = df.drop('prediction', axis=1)
    y = df['prediction']
  
    X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.1, random_state=42)

    # 3. Refit model
    big_model.fit(X_train, y_train)

    y_pred = big_model.predict(X_test)

    # Evaluate
    mse = mean_squared_error(y_test, y_pred)
    r2 = r2_score(y_test, y_pred)
    
    print(f"Mean Squared Error: {mse:.4f}")
    print(f"R² Score:       {r2:.4f}\n")
    
    big_model.save_model("models/big_model.ubj")
    print("Training completed. Final model saved as big_model.ubj")

if __name__ == "__main__":
    train()
    bench(big_model)