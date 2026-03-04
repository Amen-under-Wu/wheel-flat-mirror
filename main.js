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
    window.wheel = wheel;
    start_loop(() => {
        wheel.update();
    });
}

window.buttonCallback = main;

window.fileData = null;
        
async function loadFile() {
    const fileInput = document.querySelector('input');
    const file = fileInput.files[0];
    
    if (!file) {
        alert('请选择文件');
        return;
    }
    
    // 读取文件为ArrayBuffer
    const reader = new FileReader();
    reader.onload = function(e) {
        // 将ArrayBuffer转换为Uint8Array并存储
        const arrayBuffer = e.target.result;
        window.fileData = new Uint8Array(arrayBuffer);
        //console.log('文件已加载，大小:', window.fileData.length, '字节');
    };
    reader.readAsArrayBuffer(file);
}

window.loadFile = loadFile;

// 提供给Rust调用的方法，用于获取文件数据
window.getFileData = function() {
    return window.fileData;
};
