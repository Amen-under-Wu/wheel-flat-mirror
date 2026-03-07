let t = 0;
let x = 0;
let y = 0;
let sx = 96;
let sy = 24;
let shape = 0;
let color = 1;
function init() {
    sync(0, 0, true);
    poke(0x4000, 0x22);
    poke(0x8000, 1);
    trace("运行demo", 13);
}

function update() {
    cls(13);
    map(1, 1, 10, 10, 0, 0, 255, 1);
    print_ch("你好wheel flat轮扁!", 84, 84, 0, false, 1, false);
    print_ch("你好wheel flat轮扁!", 84, 94, 0, false, 1, true);
    print_ch("按esc回到终端", 84, 104, 0, false, 1, false);

    if (btn(0)) {
        sy = sy - 1;
    }
    if (btn(1)) {
        sy = sy + 1;
    }
    if (btn(2)) {
        sx = sx - 1;
    }
    if (btn(3)) {
        sx = sx + 1;
    }

    spr(1 + Math.floor(t % 60 / 30) * 2, sx, sy, 14, 3, 0, 0, 2, 2);

    let [xx, yy, left,] = mouse();
    if (left) {
        if (x === -1) {
            x = xx;
            y = yy;
        }
        if (shape === 0) {
            rect(x, y, Math.abs(xx - x) + 1, Math.abs(yy - y) + 1, color);
        } else if (shape === 1) {
            rectb(x, y, Math.abs(xx - x) + 1, Math.abs(yy - y) + 1, color);
        } else if (shape === 2) {
            circ(x, y, Math.sqrt((xx - x) ** 2 + (yy - y) ** 2), color);
        } else if (shape === 3) {
            circb(x, y, Math.sqrt((xx - x) ** 2 + (yy - y) ** 2), color);
        } else if (shape === 4) {
            elli(x, y, Math.abs(xx - x) + 1, Math.abs(yy - y) + 1, color);
        } else if (shape === 5) {
            ellib(x, y, Math.abs(xx - x) + 1, Math.abs(yy - y) + 1, color);
        } else if (shape === 6) {
            line(x, y, xx, yy, color);
        } else if (shape === 7) {
            tri(0, 16, 16, 0, xx, yy, color);
        } else if (shape === 8) {
            trib(0, 16, 16, 0, xx, yy, color);
        } else if (shape === 9) {
            textri(0, 16, 16, 0, xx, yy, 0, 0, 32, 0, 0, 32, false, 0);
        } else if (shape === 10) {
            textri(0, 16, 16, 0, xx, yy, 0, 0, 32, 0, 0, 32, true, 0);
        }
    } else {
        x = -1;
        y = -1;
    }

    if (btnp(4, 60, 10) || keyp(2, 60, 10)) {
        color = (color + 1) % 16;
    }
    if (btnp(5)) {
        shape = (shape + 1) % 11;
    }
    if (keyp(66)) {
        trace("运行时间: " + time() + "毫秒");
        exit();
    }
    t = t + 1;
}
