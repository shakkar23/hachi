from model import big_model as model
import onnxmltools
from onnxconverter_common.data_types import FloatTensorType

model.load_model("models/td_model.ubj")

booster = model.get_booster()
original_feature_names = booster.feature_names
if original_feature_names is not None:
    onnx_converter_conform_feature_names = [f"f{num}" for num in range(len(original_feature_names))]
    booster.feature_names = onnx_converter_conform_feature_names

# Convert the XGBoost model to ONNX format
onnx_model = onnxmltools.convert_xgboost(
    model,
    initial_types=[('input', FloatTensorType([None, 1650]))]
)

# Save the ONNX model
with open("models/big_model.onnx", "wb") as f:
    f.write(onnx_model.SerializeToString())