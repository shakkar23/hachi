from data_ranker import data
from model import lgb_ranker
import lightgbm as lgb
import numpy as np
import time


def train():
    X_train = data["X_train"]
    X_test  = data["X_test"]
    y_train = data["y_train"]
    y_test  = data["y_test"]
    group_train = data["group_train"]
    group_test  = data["group_test"]

    t = time.perf_counter()
    lgb_ranker.fit(
        X_train, y_train,
        group=group_train,
        eval_set=[(X_test, y_test)],
        eval_group=[group_test],
        eval_at=[1, 3, 5, 10],
        callbacks=[
            lgb.early_stopping(stopping_rounds=50),
            lgb.log_evaluation(period=25),
        ],
    )
    print(f"Trained in {time.perf_counter() - t:.1f}s")

    # Top-1 accuracy: how often the predicted-best candidate was actually rank 1.
    preds = lgb_ranker.predict(X_test)
    top1_hits = 0
    offset = 0
    for gsz in group_test:
        grp_preds = preds[offset:offset + gsz]
        grp_labels = y_test[offset:offset + gsz]  # N - rank, so rank 1 == max
        if np.argmax(grp_preds) == np.argmax(grp_labels):
            top1_hits += 1
        offset += gsz
    top1_acc = top1_hits / len(group_test)
    print(f"Top-1 accuracy: {top1_acc:.4f} ({top1_hits}/{len(group_test)})")

    lgb_ranker.booster_.save_model("models/ranker.txt")
    print("Saved model to models/ranker.txt")


if __name__ == "__main__":
    train()