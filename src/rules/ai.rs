//! # AI Module

use ort::{session::Session as OrtSession, value::Tensor};
use std::collections::HashMap;
use std::path::Path;
use tokenizers::Tokenizer;

use crate::config::{Config, Rule};

pub struct PanosAI {
    tokenizer: Tokenizer,
    session: OrtSession,
    pub cache: HashMap<String, Vec<f32>>,
}

impl PanosAI {
    pub fn new(model_dir: &str, rules: &[Rule]) -> anyhow::Result<Self> {
        let tokenizer_path = format!("{}/tokenizer.json", model_dir);
        let model_path = format!("{}/model.onnx", model_dir);

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

        let session = OrtSession::builder()?.commit_from_file(model_path)?;

        let mut ai = Self {
            tokenizer,
            session,
            cache: HashMap::new(),
        };

        let mut cache = HashMap::new();
        for rule in rules {
            if let Some(label) = &rule.semantic_label {
                if let Ok(mut embedding) = ai.get_embedding(label) {
                    Self::normalize(&mut embedding);
                    cache.insert(label.clone(), embedding);
                }
            }
        }

        ai.cache = cache;
        Ok(ai)
    }

    fn normalize(v: &mut [f32]) {
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > f32::EPSILON {
            for x in v.iter_mut() {
                *x /= norm;
            }
        }
    }

    fn tokenize(&self, text: &str) -> anyhow::Result<(Vec<i64>, Vec<i64>, usize)> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;

        let ids: Vec<i64> = encoding.get_ids().iter().map(|&x| x as i64).collect();
        let mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&x| x as i64)
            .collect();
        let seq_len = ids.len();

        Ok((ids, mask, seq_len))
    }

    pub fn get_embedding(&mut self, text: &str) -> anyhow::Result<Vec<f32>> {
        let (ids, mask, seq_len) = self.tokenize(text)?;

        let ids_tensor = Tensor::<i64>::from_array(([1usize, seq_len], ids))?;
        let mask_tensor = Tensor::<i64>::from_array(([1usize, seq_len], mask))?;
        let type_ids = vec![0i64; seq_len];
        let type_ids_tensor = Tensor::<i64>::from_array(([1usize, seq_len], type_ids))?;

        let outputs = self.session.run(ort::inputs![
            "input_ids" => ids_tensor,
            "attention_mask" => mask_tensor,
            "token_type_ids" => type_ids_tensor,
        ])?;

        let (shape, data) = outputs["last_hidden_state"].try_extract_tensor::<f32>()?;

        let hidden_size = shape[2] as usize;
        let token_count = shape[1] as usize;

        let mut embedding = vec![0.0f32; hidden_size];
        for token_idx in 0..token_count {
            for dim_idx in 0..hidden_size {
                embedding[dim_idx] += data[token_idx * hidden_size + dim_idx];
            }
        }

        let count = token_count as f32;
        if count > 0.0 {
            for val in &mut embedding {
                *val /= count;
            }
        }

        Ok(embedding)
    }

    pub fn cosine_similarity(&self, v1: &[f32], v2: &[f32]) -> f32 {
        let mut v1_norm = v1.to_vec();
        Self::normalize(&mut v1_norm);

        v1_norm.iter().zip(v2.iter()).map(|(a, b)| a * b).sum()
    }

    pub fn determine_rule<'a>(
        &mut self,
        file_name: &str,
        config: &Config,
        rules: &'a [Rule],
    ) -> Option<&'a Rule> {
        let cleaned_name = Path::new(file_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(file_name)
            .replace('_', " ")
            .replace('-', " ");

        let bge_query = format!("query: {}", cleaned_name);
        let file_emb = self.get_embedding(&bge_query).ok()?;

        for rule in rules {
            if let Some(label) = &rule.semantic_label {
                if let Some(label_emb) = self.cache.get(label) {
                    let score = self.cosine_similarity(&file_emb, label_emb);

                    if score >= config.ai_threshold {
                        return Some(rule);
                    }
                }
            }
        }

        None
    }
}
