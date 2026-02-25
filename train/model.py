import xgboost as xgb 

xgb_model = xgb.XGBRegressor(
    max_depth=5,
    learning_rate=0.05,
    n_estimators=1000,
    subsample=0.8,
    colsample_bytree=0.8,
    min_child_weight=5,
    gamma=0.1,
    reg_lambda=1,
    reg_alpha=0.1,
    random_state=42,
    n_jobs=2,
    eval_metric=['rmse', 'mae']
)