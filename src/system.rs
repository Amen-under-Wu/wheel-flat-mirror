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
    pub fn trace(&mut self, message: &str, _color: u8) {
        self.lines.push(message.to_string());
    }
    pub fn reset(&mut self) {}
    pub fn time(&self) -> u64 {
        Date::now() as u64 - self.program_timer
    }
    pub fn tstamp(&self) -> u64 {
        (Date::now() / 1000.0) as u64
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
        context.print_ch(&(">".to_string() + &self.context.input_buffer), 0, (self.context.lines.len() - self.context.top_line) as i32 * 9, 13, true, 1, false);
        if context.keyp(Some(62)) {
            self.context.capslock = !self.context.capslock;
        }
        if let Some(c) = self.get_char(context) {
            self.context.input_buffer.push(c);
        }
        if context.keyp_with_hold_period(50, 60, 5) {
            self.context.lines.push(">".to_string() + &self.context.input_buffer);
            if self.context.input_buffer == "run" {
                self.context.program_timer = Date::now() as u64;
                self.demo.borrow_mut().init(context, &mut self.context);
                self.program = Some(self.demo.clone());
            } else {
                self.context.lines.push("未知命令".to_string());
            }
            self.context.input_buffer.clear();
        }
        if context.keyp_with_hold_period(51, 60, 5) {
            self.context.input_buffer.pop();
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
