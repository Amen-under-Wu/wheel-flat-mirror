import init, { Wheel } from "./pkg/wheel_flat.js";

init().then(() => {
});

let fps_timer = performance.now();
let fps = 0;

function startAnimationLoop(step) {
    function frame() {
        const now = performance.now();
        step();
        fps++;
        if (now - fps_timer >= 1000) {
            console.log(fps);
            fps_timer = now;
            fps = 0;
        }
        requestAnimationFrame(frame);
    }
    requestAnimationFrame(frame);
}

export function start_loop(test_func) {
    startAnimationLoop(test_func);
}

window.start_loop = start_loop;

export function main() {
    const button = document.querySelector("button");
    if (button) {
        button.remove();
    }
    const wheel = Wheel.new();
    start_loop(() => {
        wheel.update();
    });
}

window.buttonCallback = main;
