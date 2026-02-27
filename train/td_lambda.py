from data import state, df
from model import xgb_model, mini_model, big_model
import numpy as np
import pandas as pd
from sklearn.metrics import mean_squared_error, r2_score
from sklearn.model_selection import train_test_split
import numba
from numba import jit, float64, int64
import time

iterations = 10
lmbd = 0.5

@jit(nopython=True)
def compute_targets_numba(predictions, states, lmbd):
    """
    Compute TD(λ) targets using numba
    
    Args:
        predictions: array of predictions from the model
        states: array of state indices
        state_to_reward_map: array mapping state indices to rewards
        lmbd: lambda parameter
    """
    n = len(predictions)
    targets = np.zeros(n)
    
    target = 0.0
    lambda_n = lmbd
    
    for i in range(n-1, -1, -1):
        row_state = states[i]
        reward = state_to_reward(states[i])

        if reward != 0.0:  # terminal state
            target = reward
            lambda_n = lmbd
        else:
            prediction = predictions[i]
            target = prediction + lmbd * target
            lambda_n *= lmbd
        
        targets[i] = target * (1.0 - lmbd) / (1.0 - lambda_n)
    
    return targets

def create_tf_df(df_in, model_in):
    """
    Create temporal difference targets using TD(λ) for short trajectories
    """
    td_df = df_in.copy()
    
    # napkin derivation of td-lambda for reference:
    # using plain power series, our target would be
    # prediction_0 + prediction_1 * lambda + prediction_2 * lambda^2 + ... + R * lambda^t
    # geometric series formula says in the limit, weights sum to 1/(1-lambda)
    # so multiply entire target by (1-lambda) so weights sum to 1
    # (1-lmbda)(prediction_0 + prediction_1 * lambda + prediction_2 * lambda^2 + ... + R * lambda^t)
    # this is traditional td lambda formula
    # since our trajectories are quite short, we should not use 1/(1-lambda) approximation as is
    # finite geometric series is (1-lambda^t)/(1-lambda)
    # so additionally divide by (1-lambda^t) to correct for smallness of t.

    # Extract arrays for numba
    predictions = td_df['prediction'].values.astype(np.float64)
    states = state.values.astype(np.int64)
    
    # Compute targets with numba
    targets = compute_targets_numba(predictions, states, lmbd)
    
    # Assign back to dataframe
    td_df['ground_truth'] = targets
    
    # remove non-feature columns
    features_df = td_df.drop(columns=['ground_truth', 'prediction', 'state'], errors='ignore')

    print(td_df)
    
    return features_df, td_df['ground_truth']


@jit(nopython=True)
def state_to_reward(state_int):
    if state_int == 1:
        return 1.0
    if state_int == 2:
        return -1.0
    return 0.0


def train(base_model, td_model, save=True):
    current_df = df.copy()
    
    # Load initial model
    base_model.load_model("models/model.ubj")

    # Initial targets
    current_df['prediction'] = base_model.predict(current_df.drop(columns=['state', 'prediction'], errors='ignore'))
    X, y = create_tf_df(current_df, base_model)
    
    for i in range(iterations):
        X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.1, random_state=42)

        t = time.perf_counter()

        td_model.fit(X_train, y_train)

        print(f"Fitting done in {time.perf_counter() - t:.4f} seconds")
        
        y_pred = td_model.predict(X_test)
        mse = mean_squared_error(y_test, y_pred)
        r2 = r2_score(y_test, y_pred)
        
        print(f"Iteration {i+1}/{iterations} done")
        print(f"Mean Squared Error: {mse:.4f}")
        print(f"R² Score:       {r2:.4f}\n")
        
        # Boostrapped targets
        current_df['prediction'] = td_model.predict(current_df.drop(columns=['state', 'prediction'], errors='ignore'))
        X, y = create_tf_df(current_df, td_model)
    
    if not save:
        return

    td_model.save_model("models/td_model.ubj")
    print("Training completed. Final model saved as td_model.ubj")

if __name__ == "__main__":
    train(xgb_model, xgb_model, save=True)