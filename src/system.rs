pub struct Console {
    lines: Vec<String>,
    input_buffer: String,
    cursor_pos: usize,
    top_line: usize,
    demo: Box<dyn crate::cartridge::CartProgram>,
    in_demo: bool,
}

impl Console {
    pub fn new(demo: Box<dyn crate::cartridge::CartProgram>) -> Self {
        Console {
            lines: vec!["拓竹杯参赛作品 wheel flat 轮扁".to_string(), "输入 run 进入演示".to_string()],
            input_buffer: String::new(),
            cursor_pos: 0,
            top_line: 0,
            demo,
            in_demo: false,
        }
    }
}

impl crate::cartridge::CartProgram for Console {
    fn update(&mut self, context: &mut crate::cartridge::CartContext) {
        if self.in_demo {
            self.demo.update(context);
            if context.keyp(Some(66)) {
                self.in_demo = false;
            }
            return;
        }
        context.cls(0);
        for i in self.top_line..self.lines.len() {
            context.print_ch(&self.lines[i], 0, (i - self.top_line) as i32 * 9, 13, true, 1, false);
        }
        context.print_ch(&(">".to_string() + &self.input_buffer), 0, (self.lines.len() - self.top_line) as i32 * 9, 13, true, 1, false);
        for i in 1..26 {
            if context.keyp_with_hold_period(i, 60, 5) {
                self.input_buffer.push((i - 1 + 'a' as u8) as char);
            }
        }
        if context.keyp_with_hold_period(50, 60, 5) {
            self.lines.push(">".to_string() + &self.input_buffer);
            if self.input_buffer == "run" {
                self.in_demo = true;
                self.demo.init(context);
            } else {
                self.lines.push("未知命令".to_string());
            }
            self.input_buffer.clear();
        }
        if context.keyp_with_hold_period(51, 60, 5) {
            self.input_buffer.pop();
        }
    }
}
