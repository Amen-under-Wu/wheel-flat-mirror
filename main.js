import init, {Wheel} from "./pkg/wheel_flat.js";

init().then(() => {
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

export function main() {
    document.querySelector("button").remove();
    //playSquareWave();
    const audioContext = new (window.AudioContext || window.webkitAudioContext)();
    const wheel = Wheel.new(audioContext);
    window.wheel_obj = wheel;
    start_loop(() => {window.wheel_obj.update();})
}

window.buttonCallback = main;
