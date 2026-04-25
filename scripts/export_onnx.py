import os
from pathlib import Path
from optimum.onnxruntime import ORTModelForFeatureExtraction
from transformers import AutoTokenizer

def export_model(model_id, save_dir):
    print(f"📦 Downloading and converting {model_id} to ONNX...")
    
    # 1. Download and convert to ONNX
    # We use ORTModelForFeatureExtraction because we want the embeddings (vectors)
    model = ORTModelForFeatureExtraction.from_pretrained(model_id, export=True)
    tokenizer = AutoTokenizer.from_pretrained(model_id)

    # 2. Save everything to the target directory
    output_path = Path(save_dir)
    output_path.mkdir(parents=True, exist_ok=True)
    
    model.save_pretrained(output_path)
    tokenizer.save_pretrained(output_path)
    
    print(f"✅ Export completed! Files are in: {save_dir}")
    print(f"📄 Essential files for Rust:")
    print(f"   - {save_dir}/model.onnx (The AI Model)")
    print(f"   - {save_dir}/tokenizer.json (For text processing)")

if __name__ == "__main__":
    MODEL_ID = "BAAI/bge-small-en-v1.5"
    SAVE_DIR = "model_assets"
    export_model(MODEL_ID, SAVE_DIR)
