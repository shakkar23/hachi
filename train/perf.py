from model import xgb_model, mini_model
from data import X_train, X_test, y_train, y_test
import xgboost as xgb
import pandas as pd
import numpy as np
from sklearn.metrics import mean_squared_error, r2_score

import time

def bench(model):
    AMOUNT = 100000

    X_last = X_train[-AMOUNT:]
    y_true = y_train[-AMOUNT:]

    model.fit(X_test, y_test)

    t = time.perf_counter()
    preds = model.predict(X_last)
    print(f"{AMOUNT} predictions in {time.perf_counter() - t:.4f}s")

if __name__ == "__main__":
    bench(xgb_model)
    bench(mini_model) 