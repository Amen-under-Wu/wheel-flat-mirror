// 顶点着色器源码
const vertexShaderSource = `
    // 属性变量：接收从缓冲区传入的顶点数据
    attribute vec2 a_position;
    attribute vec3 a_color;
    
    // 传递给片元着色器的变量
    varying vec3 v_color;

    uniform float u_pixelSize;
    
    void main() {
        // 将顶点位置转换为裁剪空间坐标
        gl_Position = vec4(a_position, 0.0, 1.0);
        
        // 设置点的大小
        gl_PointSize = u_pixelSize;
        
        // 将颜色传递给片元着色器
        v_color = a_color;
    }
`;

// 片元着色器源码
const fragmentShaderSource = `
    // 中等精度浮点数
    precision mediump float;
    
    // 从顶点着色器传入的颜色
    varying vec3 v_color;
    
    void main() {
        // 设置片元颜色
        gl_FragColor = vec4(v_color, 1.0);
    }
`;

// 主函数

let gl;

function init_main() {
    // 获取Canvas元素
    const canvas = document.getElementById('glCanvas');
    if (!canvas) {
        console.error('无法找到Canvas元素');
        return;
    }

    // 获取WebGL上下文
    gl = canvas.getContext('webgl');
    if (!gl) {
        console.error('浏览器不支持WebGL');
        return;
    }

    // 编译着色器
    const vertexShader = compileShader(gl, gl.VERTEX_SHADER, vertexShaderSource);
    const fragmentShader = compileShader(gl, gl.FRAGMENT_SHADER, fragmentShaderSource);
    
    // 创建着色器程序
    const program = createProgram(gl, vertexShader, fragmentShader);
    
    // 使用程序
    gl.useProgram(program);
    gl.program = program;
    update_size();

    init_vertices();

    // 创建缓冲区
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(vertices), gl.STREAM_DRAW);

    // 获取属性位置
    const positionAttributeLocation = gl.getAttribLocation(program, 'a_position');
    const colorAttributeLocation = gl.getAttribLocation(program, 'a_color');

    // 设置视口
    gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
    
    // 清除画布
    gl.clearColor(0.9, 0.9, 0.9, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);

    // 启用属性
    gl.enableVertexAttribArray(positionAttributeLocation);
    gl.enableVertexAttribArray(colorAttributeLocation);

    // 计算步长（每个顶点占用的字节数）
    const stride = 5 * Float32Array.BYTES_PER_ELEMENT; // 5个浮点数
    
    // 设置位置属性指针
    gl.vertexAttribPointer(
        positionAttributeLocation, // 属性位置
        2,                         // 每个顶点有几个分量（x, y）
        gl.FLOAT,                  // 数据类型
        false,                     // 是否归一化
        stride,                    // 步长
        0                          // 偏移量
    );

    // 设置颜色属性指针
    gl.vertexAttribPointer(
        colorAttributeLocation,    // 属性位置
        3,                         // 每个颜色有几个分量（r, g, b, a）
        gl.FLOAT,                  // 数据类型
        false,                     // 是否归一化
        stride,                    // 步长
        2 * Float32Array.BYTES_PER_ELEMENT // 偏移量（跳过前两个浮点数：x, y）
    );

    // 使用drawArrays绘制顶点
    gl.drawArrays(gl.POINTS, 0, vertices.length / 5);
}

let fps_timer = performance.now();
let fps = 0;
let count = 0;
const interval = 1000/60;
function timeout_loop() {
    const start_time = performance.now();
    test_loop();
    ++count;
    const elapsed = performance.now() - start_time;
    const next_call = Math.max(0, interval - elapsed);
    setTimeout(timeout_loop, next_call);
}
function test_loop() {
    let new_timer = performance.now();
    draw_new();
    fps++;
    if (new_timer - fps_timer >= 1000) {
        console.log(fps);
        fps_timer = new_timer;
        fps = 0;
    }
}
function start_loop() {
    timeout_loop();
}

let vertices = [];

const screen_w = 240;
const screen_h = 136;

function init_vertices() {
    for (let i = 0; i < screen_h; ++i) {
        for (let j = 0; j < screen_w; ++j) {
            vertices[(i * screen_w + j) * 5] = (j + 0.5) * 2 / screen_w - 1.0;
            vertices[(i * screen_w + j) * 5 + 1] = (i + 0.5) * 2 / screen_h - 1.0;
            vertices[(i * screen_w + j) * 5 + 2] = Math.random();
            vertices[(i * screen_w + j) * 5 + 3] = Math.random();
            vertices[(i * screen_w + j) * 5 + 4] = Math.random();
        }
    }
}

function draw_new() {
    for (let i = 0; i < screen_w * screen_h; ++i) {
        vertices[i * 5 + 2] = Math.random();
        vertices[i * 5 + 3] = Math.random();
        vertices[i * 5 + 4] = Math.random();
    }
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(vertices), gl.STREAM_DRAW);
    gl.drawArrays(gl.POINTS, 0, screen_w * screen_h);
}

// 编译着色器
function compileShader(gl, type, source) {
    const shader = gl.createShader(type);
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    
    // 检查编译状态
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        console.error('着色器编译错误:', gl.getShaderInfoLog(shader));
        gl.deleteShader(shader);
        return null;
    }
    
    return shader;
}

// 创建着色器程序
function createProgram(gl, vertexShader, fragmentShader) {
    const program = gl.createProgram();
    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);
    
    // 检查链接状态
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        console.error('程序链接错误:', gl.getProgramInfoLog(program));
        gl.deleteProgram(program);
        return null;
    }
    
    return program;
}

function update_size() {
    let u_pixelSize = gl.getUniformLocation(gl.program, "u_pixelSize");
    let pixel_size = gl.canvas.width / screen_w;
    gl.uniform1f(u_pixelSize, pixel_size);
}

async function playSquareWave() {
    // 创建 AudioContext（需用户交互或在安全上下文中）
    const audioContext = new (window.AudioContext || window.webkitAudioContext)();

    // 创建振荡器
    const oscillator = audioContext.createOscillator();
    oscillator.type = 'square'; // 设置为方波
    oscillator.frequency.setValueAtTime(440, audioContext.currentTime); // A4 = 440 Hz

    // 连接到输出
    oscillator.connect(audioContext.destination);

    // 启动并设定停止时间
    oscillator.start();
    oscillator.stop(audioContext.currentTime + 0.3); // 播放 0.3 秒

    // 可选：添加完成回调
    oscillator.onended = () => {
        console.log('方波播放结束');
    };
}

function main() {
    playSquareWave();
    start_loop();
}

// 页面加载完成后执行
window.onload = init_main;
