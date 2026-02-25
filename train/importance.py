import xgboost as xgb
import json
import numpy as np

def load_xgboost_ubj_and_save_importances(ubj_file_path, output_file_path, importance_type='weight'):
    """
    Load an XGBoost model from a .ubj file and save feature importances to a text file.
    
    Parameters:
    -----------
    ubj_file_path : str
        Path to the .ubj model file
    output_file_path : str
        Path where feature importances will be saved
    importance_type : str
        Type of feature importance ('weight', 'gain', 'cover', 'total_gain', 'total_cover')
    """
    
    try:
        # Load the model from .ubj file
        model = xgb.Booster()
        model.load_model(ubj_file_path)
        print(f"Model loaded successfully from {ubj_file_path}")
        
        # Get feature names from the model
        try:
            feature_names = model.feature_names
            if feature_names is None:
                # If feature names aren't stored in the model, use generic names
                num_features = len(model.feature_importance(importance_type='weight'))
                feature_names = [f"f{i}" for i in range(num_features)]
                print("Feature names not found in model. Using generic names f0, f1, ...")
        except:
            # Fallback if feature_names attribute doesn't exist
            feature_names = [f"f{i}" for i in range(len(model.feature_importance(importance_type='weight')))]
            print("Feature names not available. Using generic names f0, f1, ...")
        
        # Get feature importances
        importance_dict = model.get_score(importance_type=importance_type)
        
        # If get_score returns empty, use feature_importance method
        if not importance_dict:
            importance_values = model.feature_importance(importance_type=importance_type)
            importance_dict = {feature_names[i]: float(importance_values[i]) 
                             for i in range(len(importance_values))}
        
        # Convert keys to proper feature names (if they're indices)
        formatted_importance = {}
        for key, value in importance_dict.items():
            if key.startswith('f') and key[1:].isdigit():
                idx = int(key[1:])
                if idx < len(feature_names):
                    formatted_importance[feature_names[idx]] = value
                else:
                    formatted_importance[key] = value
            else:
                formatted_importance[key] = value
        
        # Sort by importance value (descending)
        sorted_importance = dict(sorted(formatted_importance.items(), 
                                       key=lambda x: x[1], reverse=True))
        
        # Write to output file
        with open(output_file_path, 'w') as f:
            # Write header
            f.write(f"Feature Importances (type: {importance_type})\n")
            f.write("=" * 50 + "\n\n")
            
            # Write each feature and its importance
            for feature, importance in sorted_importance.items():
                f.write(f"{feature}: {importance:.6f}\n")
            
            # Add summary statistics
            f.write("\n" + "=" * 50 + "\n")
            f.write(f"Total features: {len(sorted_importance)}\n")
            f.write(f"Sum of importances: {sum(sorted_importance.values()):.6f}\n")
        
        print(f"Feature importances saved to {output_file_path}")
        
        # Also display top features in console
        print(f"\nTop 10 feature importances ({importance_type}):")
        for i, (feature, importance) in enumerate(list(sorted_importance.items())[:10]):
            print(f"{i+1}. {feature}: {importance:.6f}")
        
        return sorted_importance
        
    except Exception as e:
        print(f"Error loading model or saving importances: {e}")
        return None

load_xgboost_ubj_and_save_importances("./model.ubj", "importances.txt")