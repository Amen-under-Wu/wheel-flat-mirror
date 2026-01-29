use web_sys::WebGl2RenderingContext as GL;
use wasm_bindgen::JsCast;
pub struct Screen {
    gl: GL,
    program: web_sys::WebGlProgram,
    buffer: Vec<f32>,
    canvas_width: f32,
}
impl Screen {
    pub const WIDTH: u32 = (240 + 16) * 2;
    pub const HEIGHT: u32 = (136 + 8) * 2;
    const VERTEX_SHADER_SOURCE: &str = r#"
attribute vec2 a_position;
attribute vec3 a_color;

// 传递给片元着色器的变量
varying vec3 v_color;

uniform float u_pixelSize;

void main() {
gl_Position = vec4(a_position, 0.0, 1.0);

gl_PointSize = u_pixelSize;

v_color = a_color;
}
    "#;
    const FRAGMENT_SHADER_SOURCE: &str = r#"
precision mediump float;

varying vec3 v_color;

void main() {
gl_FragColor = vec4(v_color, 1.0);
}
    "#;
    pub fn new(gl: GL) -> Self {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        let vertex_shader = Self::compile_shader(&gl, GL::VERTEX_SHADER, Self::VERTEX_SHADER_SOURCE);
        let fragment_shader = Self::compile_shader(&gl, GL::FRAGMENT_SHADER, Self::FRAGMENT_SHADER_SOURCE);
        let program = Self::create_program(&gl, &vertex_shader, &fragment_shader).unwrap();
        gl.use_program(Some(&program));
        let buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));

        let pos_attr_loc = gl.get_attrib_location(&program, "a_position") as u32;
        let color_attr_loc = gl.get_attrib_location(&program, "a_color") as u32;
        gl.enable_vertex_attrib_array(pos_attr_loc);
        gl.enable_vertex_attrib_array(color_attr_loc);
        let stride = 5 * 4;
        gl.vertex_attrib_pointer_with_i32(pos_attr_loc, 2, GL::FLOAT, false, stride, 0);
        gl.vertex_attrib_pointer_with_i32(color_attr_loc, 3, GL::FLOAT, false, stride, 2 * 4);
        
        let mut buffer = Vec::<f32>::new();
        buffer.reserve((5 * Self::WIDTH * Self::HEIGHT) as usize);
        for i in 0..Self::HEIGHT as usize {
            for j in 0..Self::WIDTH as usize {
                //buffer[(i * Self::WIDTH as usize + j) * 5] = (j * 2 + 1) as f32 / Self::WIDTH as f32 - 1.0;
                //buffer[(i * Self::WIDTH as usize + j) * 5 + 1] = (i * 2 + 1) as f32 / Self::HEIGHT as f32 - 1.0;
                buffer.push((j * 2 + 1) as f32 / Self::WIDTH as f32 - 1.0);
                buffer.push((i * 2 + 1) as f32 / Self::HEIGHT as f32 - 1.0);
                buffer.push(0.0);
                buffer.push(0.0);
                buffer.push(0.0);
            }
        }

        Self { 
            gl,
            program,
            buffer,
            canvas_width: 0.0,
        }
    }
    fn compile_shader(gl: &GL, _type: u32, source: &str) -> web_sys::WebGlShader {
        let shader = gl.create_shader(_type).expect("failed to create shader");
        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);
        shader
    }
    fn create_program(gl: &GL, vertex_shader: &web_sys::WebGlShader, fragment_shader: &web_sys::WebGlShader) -> Option<web_sys::WebGlProgram> {
        let program = gl.create_program().expect("failed to create program");
        gl.attach_shader(&program, vertex_shader);
        gl.attach_shader(&program, fragment_shader);
        gl.link_program(&program);
        if !(gl.get_program_parameter(&program, GL::LINK_STATUS).as_bool().unwrap_or(false)) {
            gl.delete_program(Some(&program));
            None
        } else {
            Some(program)
        }
    }
    pub fn adjust_size(&mut self, canvas_w: f32) {
        let u_pix_size = self.gl.get_uniform_location(&self.program, "u_pixelSize").unwrap();
        let pixel_size = canvas_w / Self::WIDTH as f32;
        self.gl.uniform1f(Some(&u_pix_size), pixel_size);
        let canvas_h = canvas_w as u32 * Self::HEIGHT / Self::WIDTH;
        self.gl.viewport(0, 0, canvas_w as i32, canvas_h as i32);
        self.canvas_width = canvas_w;
    }
    pub fn update(&mut self, buffer: &Vec<u8>) {
        for i in 0..(Self::WIDTH * Self::HEIGHT) as usize {
            for j in 0..3 {
                self.buffer[i * 5 + 2 + j] = buffer[i * 3 + j] as f32 / 255.0;
            }
        }
    } 
    pub fn display(&self) {
        let vertices_array = {
            let memory_buffer = wasm_bindgen::memory()
                .dyn_into::<js_sys::WebAssembly::Memory>().unwrap()
                .buffer();
            let location: u32 = self.buffer.as_ptr() as u32 / 4;
            js_sys::Float32Array::new(&memory_buffer).subarray(location, location + self.buffer.len() as u32)
        };
        self.gl.buffer_data_with_array_buffer_view(
            GL::ARRAY_BUFFER,
            &vertices_array,
            GL::STATIC_DRAW,
        );
        self.gl.draw_arrays(GL::POINTS, 0, (Self::WIDTH * Self::HEIGHT) as i32);
    }
}
pub trait Display {
    fn display_screen(&mut self, screen_buffer: &Vec<u8>);
}
impl Display for Screen {
    fn display_screen(&mut self, screen_buffer: &Vec<u8>) {
        self.update(screen_buffer);
        self.display();
    }
}
