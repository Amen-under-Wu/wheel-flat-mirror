use std::rc::Rc;
use std::cell::RefCell;
use js_sys::Date;

pub struct SystemContext {
    lines: Vec<String>,
    input_buffer: String,
    cursor_pos: usize,
    top_line: usize,
    program_timer: u64,
    exit_flag: bool,
    capslock: bool,
}

impl SystemContext {
    const MAX_LINES: usize = crate::cartridge::ram::Vram::SCREEN_HEIGHT / 9;
    pub fn new() -> Self {
        Self {
            lines: vec!["拓竹杯参赛作品 wheel flat 轮扁".to_string(), "输入 run 进入演示".to_string()],
            input_buffer: String::new(),
            cursor_pos: 0,
            top_line: 0,
            program_timer: Date::now() as u64,
            exit_flag: false,
            capslock: false,
        }
    }
    pub fn exit(&mut self) {
        self.exit_flag = true;
    }

    fn split_line(line: &str) -> Vec<String> {
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
    fn line_count(line: &str) -> usize { // used for input only
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
    pub fn trace(&mut self, message: &str, _color: u8) {
        let new_lines = Self::split_line(message);
        self.lines.extend(new_lines);
    }

    pub fn reset(&mut self) {}
    pub fn time(&self) -> u64 {
        Date::now() as u64 - self.program_timer
    }
    pub fn tstamp(&self) -> u64 {
        (Date::now() / 1000.0) as u64
    }

    fn scroll(&mut self, lines: i32) {
        let max_top = if self.lines.len() + Self::line_count(&self.input_buffer) > Self::MAX_LINES { self.lines.len() + Self::line_count(&self.input_buffer) - Self::MAX_LINES } else { 0 };
        if lines > 0 {
            self.top_line = (self.top_line + lines as usize).min(max_top);
        } else {
            self.top_line = self.top_line.saturating_sub((-lines) as usize);
        }
    }
    fn scroll_to_bottom(&mut self) {
        let max_top = if self.lines.len() + Self::line_count(&self.input_buffer)> Self::MAX_LINES { self.lines.len() + Self::line_count(&self.input_buffer) - Self::MAX_LINES } else { 0 };
        self.top_line = max_top;
    }

}

pub trait SystemProgram {
    fn init(&mut self, context: &mut crate::cartridge::CartContext, sys_context: &mut SystemContext) {}
    fn update(&mut self, context: &mut crate::cartridge::CartContext, sys_context: &mut SystemContext) {}
    fn scanline(&mut self, context: &mut crate::cartridge::CartContext, sys_context: &mut SystemContext, i: i32) {}
    fn overlay(&mut self, context: &mut crate::cartridge::CartContext, sys_context: &mut SystemContext) {}
}

pub struct WheelSystem {
    context: SystemContext,
    demo: Rc<RefCell<dyn SystemProgram>>,
    program: Option<Rc<RefCell<dyn SystemProgram>>>,
}

impl WheelSystem {
    pub fn new(demo: Rc<RefCell<dyn SystemProgram>>) -> Self {
        Self {
            context: SystemContext::new(),
            demo,
            program: None,
        }
    }
    fn get_char(&self, context: &crate::cartridge::CartContext) -> Option<char> {
        const KEYBOARD_OFFSET: usize = crate::cartridge::ram::Ram::KEYBOARD_OFFSET;
        const NUM_SHIFTS: [char; 10] = [')', '!', '@', '#', '$', '%', '^', '&', '*', '('];
        let keys: Vec<u8> = (0..4).map(|i| context.peek(KEYBOARD_OFFSET + i)).collect();
        let shift = keys.contains(&64) || self.context.capslock;
        for i in keys {
            if context.keyp_with_hold_period(i, 60, 5) {
                let c = match i {
                    1..=26 => (i - 1 + if shift { b'A' } else { b'a' }) as char,
                    27..=36 => if shift { NUM_SHIFTS[(i - 27) as usize] } else { (i - 27 + b'0') as char },
                    37 => if shift { '_' } else { '-' },
                    38 => if shift { '+' } else { '=' },
                    39 => if shift { '{' } else { '[' },
                    40 => if shift { '}' } else { ']' },
                    41 => if shift { '|' } else { '\\' },
                    42 => if shift { ':' } else { ';' },
                    43 => if shift { '"' } else { '\'' },
                    44 => if shift { '~' } else { '`' },
                    45 => if shift { '<' } else { ',' },
                    46 => if shift { '>' } else { '.' },
                    47 => if shift { '?' } else { '/' },
                    48 => ' ',
                    49 => '\t',
                    79..=88 => (i - 79 + b'0') as char,
                    89 => '+',
                    90 => '-',
                    91 => '*',
                    92 => '/',
                    94 => '.',
                    _ => continue,
                };
                return Some(c);
            }
        }
        None
    }
}

impl crate::cartridge::CartProgram for WheelSystem {
    fn update(&mut self, context: &mut crate::cartridge::CartContext) {
        if let Some(program) = &self.program {
            program.borrow_mut().update(context, &mut self.context);
            if self.context.exit_flag {
                self.program = None;
                self.context.exit_flag = false;
            }
            return;
        }
        context.cls(0);
        for i in self.context.top_line..self.context.lines.len() {
            context.print_ch(&self.context.lines[i], 0, (i - self.context.top_line) as i32 * 9, 13, true, 1, false);
        }
        let input_lines = SystemContext::split_line(&(">".to_string() + &self.context.input_buffer));
        for i in 0..input_lines.len() {
            context.print_ch(&input_lines[i], 0, (self.context.lines.len() - self.context.top_line) as i32 * 9 + i as i32 * 9, 13, true, 1, false);
        }
        //context.print_ch(, 0, (self.context.lines.len() - self.context.top_line) as i32 * 9, 13, true, 1, false);
        if context.keyp(Some(62)) {
            self.context.capslock = !self.context.capslock;
        }
        if let Some(c) = self.get_char(context) {
            self.context.scroll_to_bottom();
            self.context.input_buffer.push(c);
        }
        if context.keyp_with_hold_period(50, 60, 5) {
            self.context.lines.extend(input_lines);
            match self.context.input_buffer.as_str() {
                "" => {},
                "clear" => self.context.lines.clear(),
                "cls" => self.context.lines.clear(),
                "run" => {
                    self.context.program_timer = Date::now() as u64;
                    self.demo.borrow_mut().init(context, &mut self.context);
                    self.program = Some(self.demo.clone());
                },
                _ => {
                    self.context.lines.push("未知命令".to_string());
                },
            }
            self.context.input_buffer.clear();
            self.context.scroll_to_bottom();
        }
        if context.keyp_with_hold_period(51, 60, 5) {
            self.context.input_buffer.pop();
        }
        let (_, _, _, _, _, _, sy) = context.mouse();
        if sy > 0 {
            self.context.scroll(1);
        } else if sy < 0 {
            self.context.scroll(-1);
        }

    }
    fn scanline(&mut self, context: &mut crate::cartridge::CartContext, line_i: usize) {
        if let Some(program) = &self.program {
            program.borrow_mut().scanline(context, &mut self.context, line_i as i32);
        }
    }
    fn overlay(&mut self, context: &mut crate::cartridge::CartContext) {
        if let Some(program) = &self.program {
            program.borrow_mut().overlay(context, &mut self.context);
        }
    }
}
