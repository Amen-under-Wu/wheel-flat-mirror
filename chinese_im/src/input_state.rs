use crate::pinyin_dict::PinyinDict;
use std::io::Write;

#[derive(Debug)]
pub struct InputState {
    composing: String,
    candidates: Vec<String>,
    dict: PinyinDict,
    current_page:usize,
    items_per_page:usize,
}

impl InputState {
    pub fn new(dict: PinyinDict) -> Self {
        Self {
            composing: String::new(),
            candidates: vec![],
            current_page:0,
            items_per_page:6,
            dict,
        }
    }

    pub fn push_char(&mut self, ch: char) {
        match ch {
            'a'..='z' | 'A'..='Z' => {
                self.composing.push(ch.to_ascii_lowercase());
                self.update_candidates();
            }
            '+' | '=' => {  // + 键和 = 键下一页
                self.next_page();
            }
            '-' | '_' => {  // - 键和 _ 键上一页
                self.prev_page();
            }
            ' ' => {
                if !self.candidates.is_empty() {
                    self.commit(0);  // 空格选第一个
                }
            }
            '1'..='9' => {
                let idx = (ch as u32 - b'1' as u32) as usize;
                self.commit(idx);
            }
            '\r' | '\n' => { /* ... */ }
            '\x08' | '\x7f' => { /* backspace */ }
            _ => {}
        }
    }
    fn update_candidates(&mut self) {
        self.candidates = self.dict.get_candidates(&self.composing);
        self.current_page = 0;
    }

    pub fn next_page(&mut self) {
        let total_pages = (self.candidates.len() + self.items_per_page - 1) / self.items_per_page;
        if self.current_page + 1 < total_pages {
            self.current_page += 1;
        }
    }

    pub fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
        }
    }

    pub fn get_current_page_candidates(&self) -> Vec<String> {
        let start = self.current_page * self.items_per_page;
        let end = (start + self.items_per_page).min(self.candidates.len());

        if start >= self.candidates.len() {
            return vec![];
        }

        self.candidates[start..end].to_vec()
    }

    pub fn commit(&mut self, idx: usize) {
        if idx < self.candidates.len() {
            let selected = self.candidates[idx].clone();
            println!("\n→ 选中: {}", selected);
            self.composing.clear();
            self.candidates.clear();
            self.current_page = 0;
        }
    }

    pub fn display(&self) {
        print!("\r输入: {}    ", self.composing);

        if !self.candidates.is_empty() {
            let page_cands = self.get_current_page_candidates();
            let total_pages = (self.candidates.len() + self.items_per_page - 1) / self.items_per_page;

            print!("  候选 (第 {}/{} 页): ", self.current_page + 1, total_pages);

            for (i, cand) in page_cands.iter().enumerate() {
                let global_idx = self.current_page * self.items_per_page + i;
                print!("{}.{}  ", global_idx + 1, cand);
            }

            // 显示翻页提示（可选）
            if total_pages > 1 {
                print!("   [+]下一页  [-]上一页");
            }
        }

        print!("          ");
        std::io::stdout().flush().unwrap();
    }

    pub fn is_active(&self) -> bool {
        !self.composing.is_empty() || !self.candidates.is_empty()
    }
}