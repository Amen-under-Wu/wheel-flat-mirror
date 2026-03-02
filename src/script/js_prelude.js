function btnp(id, hold, period = 1) {
    if (typeof hold === 'undefined') {
        return btnp_1(id);
    }
    else {
        return btnp_3(id, hold, period);
    }
}

function clip(x = 0, y = 0, w = 240, h = 136) {
    clip_4(x, y, w, h);
}

function cls(color = 0) {
    cls_1(color);
}

function font(text, x, y, trans_color = -1, w = 8, h = 8, fixed = false, scale = 1) {
    // unfinished
}

function key(code) {
    if (typeof code === 'undefined') {
        return key_0();
    }
    else {
        return key_1(code);
    }
}

function keyp(code, hold, period = 1) {
    if (typeof hold === 'undefined') {
        return keyp_1(code);
    }
    else {
        return keyp_3(code, hold, period);
    }
}

function map(x = 0, y = 0, w = 30, h = 17, sx = 0, sy = 0, trans_color = -1, scale = 1, remap) {
    if (typeof remap === 'undefined') {
        map_8(x, y, w, h, sx, sy, trans_color, scale);
    }
    else {
        // rewrite map in js
    }
}

function peek(addr, bits = 8) {
    if (bits === 8) {
        return peek_1(addr);
    }
    else {
        return peek_2(addr, bits);
    }
}

function pix(x, y, color) {
    if (typeof color === 'undefined') {
        return pix_2(x, y);
    }
    else {
        pix_3(x, y, color);
    }
}

function pmem(idx, val) {
    if (typeof val === 'undefined') {
        return pmem_1(idx);
    }
    else {
        return pmem_2(idx, val);
    }
}

function poke(addr, val, bits = 8) {
    if (bits === 8) {
        poke_2(addr, val);
    }
    else {
        poke_3(addr, val, bits);
    }
}

function print(text, x = 0, y = 0, color = 12, fixed = false, scale = 1, alt_font = false) {
    return print_7(text, x, y, color, fixed, scale, alt_font);
}

function print_ch(text, x = 0, y = 0, color = 12, fixed = false, scale = 1, alt_font = false) {
    return print_ch_7(text, x, y, color, fixed, scale, alt_font);
}

function spr(id, x, y, trans_color = -1, scale = 1, flip = 0, rotate = 0, w = 1, h = 1) {
    spr_vec_9([id, x, y, trans_color, scale, flip, rotate, w, h]);
}

function textri(x1, y1, x2, y2, x3, y3, u1, v1, u2, v2, u3, v3, use_map = false, trans_color = -1) {
    textri_3([x1, y1, x2, y2, x3, y3, u1, v1, u2, v2, u3, v3], use_map, trans_color);
}

function trace(msg, color = 15) {
    trace_2(msg, color);
}
