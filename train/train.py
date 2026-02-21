from data import X_train, X_test, y_train, y_test
import pandas as pd 
import xgboost as xgb 
from sklearn.model_selection import train_test_split
from sklearn.metrics import mean_squared_error, r2_score
from sklearn.datasets import make_regression
import numpy as np 

import time

t = time.perf_counter()

# Create XGBoost regressor
xgb_model = xgb.XGBRegressor(
    n_estimators=150,
    max_depth=15,
    learning_rate=0.1,
    random_state=42
)

# Train the model
xgb_model.fit(X_train, y_train)

# Make predictions
y_pred = xgb_model.predict(X_test)

# Evaluate
mse = mean_squared_error(y_test, y_pred)
r2 = r2_score(y_test, y_pred)

print(f"Training done in {time.perf_counter() - t} seconds")
print(f"Mean Squared Error: {mse:.4f}")
print(f"MSE Legend. 0.01 : Good. 0.05 : Fine. 0.1 : Weak. 0.25 : Worst")
print(f"R² Score: {r2:.4f}")

