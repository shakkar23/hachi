from data import X_train, X_test, y_train, y_test
import pandas as pd 
import xgboost as xgb 
from sklearn.model_selection import train_test_split
from sklearn.metrics import mean_squared_error, r2_score
from sklearn.datasets import make_regression
import numpy as np 

from xgboost import plot_importance
import matplotlib.pyplot as plt

from model import xgb_model

import time

t = time.perf_counter()

# Train the model
xgb_model.fit(X_train, y_train)

# Make predictions
y_pred = xgb_model.predict(X_test)

# Evaluate
mse = mean_squared_error(y_test, y_pred)
r2 = r2_score(y_test, y_pred)

print(f"Training done in {time.perf_counter() - t:.4f} seconds")
print(f"Mean Squared Error: {mse:.4f}")
print(f"MSE Legend. 0.01 : Good. 0.05 : Fine. 0.1 : Weak. 0.25 : Worst")
print(f"R² Score: {r2:.4f}")

xgb_model.save_model("./model.ubj") 