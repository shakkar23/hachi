import xgboost as xgb 
import lightgbm as lgb
from catboost import CatBoostRegressor

device = "cpu"

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
    eval_metric=['rmse', 'mae'],
    device = device
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
    n_estimators=10000,
    subsample=0.8,
    colsample_bytree=0.8,
    gamma=0.1,
    reg_lambda=1,
    reg_alpha=0.1,
    random_state=42,
    n_jobs=-1,
    eval_metric=['rmse', 'mae'],
    tree_method="hist", device=device
)

big_lgb_model = lgb.LGBMRegressor(
    num_leaves       = 25,
    max_depth        = 8,
    n_estimators     = 500,
    learning_rate    = 0.025,
    objective         = 'regression', 
    metric            = ['rmse', 'mae'],
    random_state      = 42,
    n_jobs            = -1,
    verbosity         = 0    ,
    device = device
)

small_lgb_model = lgb.LGBMRegressor( 
    num_leaves       = 15,      
    max_depth        = 5,        
    n_estimators     = 100,     
    learning_rate    = 0.05,    
    objective        = 'regression',
    metric           = ['rmse', 'mae'],
    random_state     = 42,
    n_jobs           = -1,
    verbosity        = 0,
    device           = device,
    min_child_samples = 20,
    subsample        = 0.8,
    colsample_bytree = 0.8
)

lgb_ranker = lgb.LGBMRanker(
    objective         = 'lambdarank',
    metric            = 'ndcg',
    num_leaves        = 31,
    max_depth         = 7,
    n_estimators      = 1000,
    learning_rate     = 0.05,
    min_child_samples = 20,
    subsample         = 0.8,
    colsample_bytree  = 0.8,
    reg_lambda        = 1.0,
    reg_alpha         = 0.1,
    random_state      = 42,
    n_jobs            = -1,
    verbosity         = 0,
    device            = device,
)