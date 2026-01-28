import init, {Wheel} from "./pkg/wheel_flat.js";

init().then(() => {
    const wheel = Wheel.new();
    window.wheel_obj = wheel;
})

let fps_timer = performance.now();
let fps = 0;
const interval = 1000/60;
function timeout_loop(test_func) {
    const start_time = performance.now();
    test_loop(test_func);
    const elapsed = performance.now() - start_time;
    const next_call = Math.max(0, interval - elapsed);
    setTimeout(() => {timeout_loop(test_func);}, next_call);
}
function test_loop(test_func) {
    let new_timer = performance.now();
    test_func();
    fps++;
    if (new_timer - fps_timer >= 1000) {
        console.log(fps);
        fps_timer = new_timer;
        fps = 0;
    }
}
export function start_loop(test_func) {
    timeout_loop(test_func);
}
window.start_loop = start_loop;

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

export function main() {
    document.querySelector("button").remove();
    playSquareWave();
    start_loop(() => {window.wheel_obj.update();})
}

window.buttonCallback = main;
