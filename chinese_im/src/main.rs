mod pinyin_dict;
mod input_state;

use anyhow::Result;
use crate::pinyin_dict::PinyinDict;
use crate::input_state::InputState;
use std::io::{self, BufRead};

fn main() -> Result<()> {
    println!("简易命令行拼音输入法 Demo");
    println!("输入拼音字母，按空格或数字(1-9)选择，按回车提交当前 composing，按 Ctrl+C 退出");

    let dict = PinyinDict::load(r"dict/8105.dict.yaml")
        .expect("加载词典失败，请确认 dict/8105.dict.yaml 存在");

    let mut state = InputState::new(dict);

    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        state.display();

        line.clear();
        if stdin.read_line(&mut line).is_err() {
            break;
        }

        let input = line.trim();

        if input.is_empty() {
            // 纯回车
            state.push_char('\n');
            continue;
        }

        // 逐字符处理（支持一次输入多个字符）
        for ch in input.chars() {
            state.push_char(ch);
        }
    }

    println!("\n退出");
    Ok(())
}