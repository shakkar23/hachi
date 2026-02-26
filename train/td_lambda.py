from data import state, df
from model import xgb_model, mini_model, big_lgb_model
import numpy as np
import pandas as pd
from sklearn.metrics import mean_squared_error, r2_score
from sklearn.model_selection import train_test_split

iterations = 5
lmbd = 0.5

def create_tf_df(df_in, model_in):
    """
    Create temporal difference targets using TD(λ) for short trajectories
    """
    td_df = df_in.copy()
    
    # Get model predictions for current state of the data
    if 'prediction' not in td_df.columns:
        td_df['prediction'] = model_in.predict(td_df.drop(columns=['state'], errors='ignore'))
    
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

    # targets column as numpy array
    targets = np.zeros(len(td_df))

    target = 0.0
    lambda_accum = 1.0          # λ^n term
    lambda_n = lmbd             # current λ^n value
    
    # iterate df backwards
    for i in reversed(range(0, len(td_df))):
        # calculate reward from row.state
        # if reward is 1 or -1, set target = reward * (1-lmbd), set lmbd_n = lmbd
        # else do target = [target * lmbd + prediction * (1-lmbd)] * (1-lmbd_n * lmbd) / (1-lmbd_n) and lmbd_n *= lmbd
        # finally set 'ground_truth' column of targets to target

        row_state = int(state.iloc[i])
        reward = state_to_reward(row_state)
        
        if reward != 0:
            # reset at terminal state
            target = reward
            lambda_n = lmbd
        else:
            # TD(λ) forwards view
            prediction = td_df.iloc[i]['prediction']
            target = prediction + lmbd * target

            lambda_n *= lmbd
        
        targets[i] = target

        # correct for short horizon
        targets[i] *= (1-lmbd)/(1-lambda_n)
    
    td_df['ground_truth'] = targets
    
    # remove non-feature columns
    features_df = td_df.drop(columns=['ground_truth', 'prediction', 'state'], errors='ignore')

    print(td_df)
    
    return features_df, td_df['ground_truth']


def state_to_reward(state_int):
    if state_int == 1:
        return 1.0
    if state_int == 2:
        return -1.0
    return 0.0


def train(base_model, td_model, save=True):
    current_df = df.copy()
    
    # Load initial model
    base_model.load_model("models/td_model.ubj")

    # Initial targets
    current_df['prediction'] = base_model.predict(current_df.drop(columns=['state', 'prediction'], errors='ignore'))
    X, y = create_tf_df(current_df, base_model)
    
    for i in range(iterations):
        X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.1, random_state=42)

        td_model.fit(X_train, y_train)
        
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

    td_model.save_model("models/big_td_model.ubj")
    print("Training completed. Final model saved as big_td_model.ubj")

if __name__ == "__main__":
    train(xgb_model, big_lgb_model, save=False)