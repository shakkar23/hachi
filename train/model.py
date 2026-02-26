import xgboost as xgb 
import lightgbm as lgb
from catboost import CatBoostRegressor

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
    n_jobs=-1,
    eval_metric=['rmse', 'mae']
)

mini_model = CatBoostRegressor(
    max_depth=5,
    learning_rate=0.05,
    n_estimators=125,
    verbose=0
)

big_model = xgb.XGBRegressor(
    max_depth=7,
    learning_rate=0.025,
    n_estimators=5000,
    subsample=0.8,
    colsample_bytree=0.8,
    min_child_weight=5,
    gamma=0.1,
    reg_lambda=1,
    reg_alpha=0.1,
    random_state=42,
    n_jobs=-1,
    eval_metric=['rmse', 'mae']
)

big_lgb_model = lgb.LGBMRegressor(
    num_leaves       = 255,
    max_depth        = 12,
    n_estimators     = 5000,
    learning_rate    = 0.025,
    min_child_samples = 20,
    min_child_weight  = 0.001,
    lambda_l1         = 0.1,
    lambda_l2         = 1.0,
    min_gain_to_split = 0.0,
    subsample         = 0.8,      
    colsample_bytree  = 0.8,
    bagging_fraction  = 0.8,
    bagging_freq      = 5,
    objective         = 'regression', 
    metric            = ['rmse', 'mae'],
    random_state      = 42,
    n_jobs            = -1,
    verbosity         = -1    
)