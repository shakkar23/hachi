# predict_last100.py   ← separate file

from data import X_train, X_test, y_train, y_test
import xgboost as xgb
import pandas as pd
import numpy as np
from sklearn.metrics import mean_squared_error, r2_score

# Load saved model
model = xgb.XGBRegressor()
model.load_model("models/model.ubj")

print("Model loaded • using last 100 rows of X_test from data module\n")

# ──── Grab last 100 ────
X_last = X_test[-100:]
y_true = y_test[-100:]

preds = model.predict(X_last)

# Nice table
df = pd.DataFrame({
    'true':  y_true.round(4),
    'pred':  preds.round(4),
    'diff': (y_true - preds).round(4)
})

print("Last 100 test predictions:")
print(df.to_string(index=True))

# Summary stats on these 100 points
print("\nQuick stats on last 100:")
print(f"MAE:   {np.mean(np.abs(y_true - preds)):.4f}")
print(f"RMSE:  {np.sqrt(mean_squared_error(y_true, preds)):.4f}")
print(f"R²:    {r2_score(y_true, preds):.4f}")