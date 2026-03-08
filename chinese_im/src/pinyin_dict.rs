use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Entry {
    text: String,
    code: String,
    #[serde(default)]
    weight: f32,
}

#[derive(Debug, Default)]
pub struct PinyinDict {
    // key: 小写无空格拼音("ni"、"hao"、"nihao")
    // value: 按权重降序排列的汉字/词列表
    pub map: HashMap<String, Vec<String>>,
}

impl PinyinDict {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(&path)
            .with_context(|| format!("无法打开词典文件: {:?}", path.as_ref()))?;
        let reader = BufReader::new(file);

        let mut temp: HashMap<String, Vec<(String, f32)>> = HashMap::new();
        let mut in_body = false;

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();

            if trimmed == "..." {
                in_body = true;
                continue;
            }
            if !in_body || trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.split('\t').collect();
            if parts.len() < 2 {
                continue;
            }

            let text = parts[0].to_string();
            let pinyin = parts[1].to_lowercase();
            let weight = parts
                .get(2)
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(1.0);

            for syllable in pinyin.split_whitespace() {
                let key = syllable.trim().to_string();
                if key.is_empty() {
                    continue;
                }
                temp.entry(key)
                    .or_default()
                    .push((text.clone(), weight));
            }
        }

        // 整理：排序 + 去重
        let mut map = HashMap::new();

        for (key, mut candidates) in temp {
            candidates.sort_by(|a, b| b.1.total_cmp(&a.1));
            let mut seen = HashSet::new();
            let mut unique = Vec::new();
            for (text, _) in candidates{
                if seen.insert(text.clone()){
                    unique.push(text);
                }
            }
            map.insert(key, unique);
        }

        Ok(Self { map })
    }

    pub fn get_candidates(&self, input: &str) -> Vec<String> {
        let normalized = input.trim().to_lowercase();

        //不允许空格
        if normalized.contains(char::is_whitespace){
            return vec![];
        }

        self.map
            .get(&normalized)
            .map(|cands| cands.iter().take(10).cloned().collect())
            .unwrap_or_default()

    }
}