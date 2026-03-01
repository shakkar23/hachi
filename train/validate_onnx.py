"""
Test script: Compare original XGBoost predictions vs ONNX Runtime predictions

Assumes:
- models/big_model.onnx       exists (the exported model)
- models/model.ubj or models/td_model.ubj exists (original xgboost model)
- You have X_test, y_test available from your data module
"""

import numpy as np
import pandas as pd
import xgboost as xgb
from sklearn.metrics import mean_squared_error, r2_score

# ─── ONNX Runtime ────────────────────────────────────────────────
try:
    import onnxruntime as ort
except ImportError:
    print("onnxruntime not found. Install with:")
    print("pip install onnxruntime onnxruntime-tools")
    exit(1)

# ──── Configuration ──────────────────────────────────────────────
ONNX_MODEL_PATH = "models/big_model.onnx"
XGB_MODEL_PATH  = "models/td_model.ubj"          # ← change if name is different
N_SAMPLES       = 300                         # how many points to compare

# ──── Load original XGBoost model ────────────────────────────────
print("Loading original XGBoost model...")
xgb_model = xgb.XGBRegressor()
xgb_model.load_model(XGB_MODEL_PATH)
print("XGBoost model loaded.")

# ──── Load ONNX model ────────────────────────────────────────────
print("Loading ONNX model...")
session = ort.InferenceSession(ONNX_MODEL_PATH)
input_name = session.get_inputs()[0].name
print(f"ONNX input name: {input_name}")
print("ONNX model loaded.")

# ──── Prepare test data ──────────────────────────────────────────
from data import X_test, y_test   # ← your data module

# Take last N samples (or random – your choice)
X_test_np = X_test[-N_SAMPLES:].astype(np.float32)   # ONNX usually wants float32
y_true    = y_test[-N_SAMPLES:]

X_test_np = np.asarray(X_test_np, dtype=np.float32)

print(f"\nEvaluating on {len(X_test_np)} samples...")

# ──── Predict with original XGBoost ──────────────────────────────
print("→ Predicting with XGBoost...")
preds_xgb = xgb_model.predict(X_test_np)

# ──── Predict with ONNX ──────────────────────────────────────────
print("→ Predicting with ONNX Runtime...")
print(X_test_np)
input_feed = {input_name: X_test_np}
preds_onnx_raw = session.run(None, input_feed)
preds_onnx = preds_onnx_raw[0].ravel()          # usually [batch, 1] → flatten

# ──── Compare ────────────────────────────────────────────────────
diff = preds_xgb - preds_onnx
abs_diff = np.abs(diff)

df_compare = pd.DataFrame({
    'true':   y_true.round(4),
    'xgb':    preds_xgb.round(4),
    'onnx':   preds_onnx.round(4),
    'diff':   diff.round(6),
    '|diff|': abs_diff.round(6),
})

print("\n" + "="*70)
print("First 15 predictions (sorted by largest |diff|)")
print("="*70)
print(df_compare.nlargest(15, '|diff|').to_string(index=True))

# ──── Summary statistics ─────────────────────────────────────────
print("\n" + "-"*60)
print("Agreement statistics (last", N_SAMPLES, "samples)")
print("-"*60)
print(f"Max absolute difference    : {abs_diff.max():.6f}")
print(f"Mean absolute difference   : {abs_diff.mean():.6f}")
print(f"Median absolute difference : {np.median(abs_diff):.6f}")
print(f"90th percentile |diff|     : {np.percentile(abs_diff, 90):.6f}")
print(f"Std of differences         : {diff.std():.6f}")

mae_xgb  = np.mean(np.abs(y_true - preds_xgb))
mae_onnx = np.mean(np.abs(y_true - preds_onnx))

print(f"\nXGBoost  MAE  : {mae_xgb:.4f}")
print(f"ONNX     MAE  : {mae_onnx:.4f}")
print(f"R² (XGBoost)  : {r2_score(y_true, preds_xgb):.4f}")
print(f"R² (ONNX)     : {r2_score(y_true, preds_onnx):.4f}")

tol = 1e-4
if abs_diff.max() < tol:
    print(f"\n→ Excellent agreement! (max diff < {tol})")
elif abs_diff.max() < 5e-3:
    print(f"\n→ Very good agreement (max diff < 0.005)")
elif abs_diff.max() < 0.05:
    print("\n→ Acceptable — but check why differences exist")
else:
    print("\nLarge differences detected.")

# Optional: save detailed comparison
# df_compare.to_csv("xgb_vs_onnx_comparison.csv", index=True)