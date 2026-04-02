use js_sys::Date;

pub struct SystemContext {
    pub lines: Vec<(String, u8)>,
    pub input_buffer: String,
    pub cursor_pos: usize,
    pub top_line: usize,
    pub program_timer: u64,
    pub exit_flag: bool,
    pub reset_flag: bool,
    pub capslock: bool,
}

impl SystemContext {
    const MAX_LINES: usize = crate::cartridge::ram::Vram::SCREEN_HEIGHT / 9;
    pub fn new() -> Self {
        Self {
            lines: vec![
                ("拓竹杯初审未过审作品 wheel flat 轮扁".to_string(), 4),
                ("使用方法请自行阅读源码以探索 :P".to_string(), 5),
            ],
            input_buffer: String::new(),
            cursor_pos: 0,
            top_line: 0,
            program_timer: Date::now() as u64,
            reset_flag: false,
            exit_flag: false,
            capslock: false,
        }
    }
    pub fn exit(&mut self) {
        self.exit_flag = true;
    }

    pub fn split_line(line: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut current_line = String::new();
        let mut w = 0;
        for c in line.chars() {
            w = if c.is_ascii() { w + 6 } else { w + 9 };
            if w >= 240 || c == '\n' {
                result.push(current_line);
                current_line = String::new();
                w = 0;
            }
            current_line.push(c);
        }
        if !current_line.is_empty() {
            result.push(current_line);
        }
        result
    }
    pub fn line_count(line: &str) -> usize {
        // used for input only
        let mut count = 0;
        let mut w = 6; // start with 6 to account for the ">" prompt
        for c in line.chars() {
            w = if c.is_ascii() { w + 6 } else { w + 9 };
            if w >= 240 || c == '\n' {
                count += 1;
                w = 0;
            }
        }
        if w > 0 {
            count += 1;
        }
        count
    }
    pub fn trace(&mut self, message: &str, color: u8) {
        let new_lines = Self::split_line(message);
        self.lines
            .extend(new_lines.into_iter().map(|line| (line.clone(), color)));
    }

    pub fn reset(&mut self) {
        self.reset_flag = true;
    }
    pub fn time(&self) -> u64 {
        Date::now() as u64 - self.program_timer
    }
    pub fn tstamp(&self) -> u64 {
        (Date::now() / 1000.0) as u64
    }

    pub fn scroll(&mut self, lines: i32) {
        let max_top = if self.lines.len() + Self::line_count(&self.input_buffer) > Self::MAX_LINES {
            self.lines.len() + Self::line_count(&self.input_buffer) - Self::MAX_LINES
        } else {
            0
        };
        if lines > 0 {
            self.top_line = (self.top_line + lines as usize).min(max_top);
        } else {
            self.top_line = self.top_line.saturating_sub((-lines) as usize);
        }
    }
    pub fn scroll_to_bottom(&mut self) {
        let max_top = if self.lines.len() + Self::line_count(&self.input_buffer) > Self::MAX_LINES {
            self.lines.len() + Self::line_count(&self.input_buffer) - Self::MAX_LINES
        } else {
            0
        };
        self.top_line = max_top;
    }
}
