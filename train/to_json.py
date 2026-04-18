from model import big_model as model

model.load_model("models/td_model.ubj")
model.save_model("models/td_model.json")

from model import mini_model

mini_model.load_model("models/mini_model.ubj")
mini_model.save_model("models/mini_model.json", format="json")