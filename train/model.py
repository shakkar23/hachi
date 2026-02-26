import xgboost as xgb 
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

def get_base_model(model):
    model.load_model("models/base_model.ubj")
    
def save_base_model(model):
    model.save_model("models/base_model.ubj")

def get_td_model(model):
    model.load_model("models/td_model.ubj")
    
def save_td_model(model):
    model.save_model("models/td_model.ubj")
    
def get_mini_model(model):
    model.load_model("models/mini_model.ubj")
    
def save_mini_model(model):
    model.save_model("models/mini_model.ubj")
