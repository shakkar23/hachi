import xgboost as xgb
from cuml.fil import ForestInference
import data from data

X_test, X_train, y_test, y_train = data
 
fil_model = ForestInference.load("xgb_model.ubj")

preds = fil_model.predict(X_test)
probs = fil_model.predict_proba(X_test)