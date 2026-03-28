// title: Fallspire
// original author: petet and sintel
// Rust porting: AuW

use crate::cartridge::CartContext;
use crate::system::SystemContext;
use crate::wrapper::InternalProgram;
use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;

type Float = f32;

fn round(v: Float, d: Option<Float>) -> Float {
    let m = (10.0 as Float).powf(d.unwrap_or(0.0));
    (v * m + 0.5).floor() / m
}

fn smoothmin(a: Float, b: Float, k: Option<Float>) -> Float {
    let k = k.unwrap_or(0.1);
    let h = (k - (a - b).abs()).max(0.0) / k;
    a.min(b) - h * h * k * 0.25
}

fn smootherstep(x: Float) -> Float {
    if x <= 0.0 {
        0.0
    } else if x >= 1.0 {
        1.0
    } else {
        x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
    }
}

fn easeout(x: Float, q: Option<Float>) -> Float {
    let q = q.unwrap_or(100.0);
    if x <= -1.570796 * q {
        -x - 0.570796 * q
    } else {
        if x >= 0.0 {
            0.0
        } else {
            -q * ((x / q).cos() - 1.0)
        }
    }
}

struct Camera {
    x: Float,
    y: Float,
    z: Float,
    xtilt: Float,
    ytilt: Float,
    ztilt: Float,
}

impl Camera {
    fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            xtilt: 0.0,
            ytilt: 0.0,
            ztilt: 0.0,
        }
    }
}

struct Leaf {
    r: Vec<i32>,
    th: Vec<Float>,
}

impl Leaf {
    fn new() -> Self {
        Self {
            r: vec![12, 3, 14, 6, 12, 5, 14, 5, 12, 6, 14, 3, 12],
            th: vec![
                0.0, 0.1, 1.22, 1.57, 2.09, 2.62, 3.14, -2.62, -2.09, -1.57, -1.22, -0.1, 0.0,
            ],
        }
    }
}

struct Track {
    transp: i32,
    ind: i32,
    instr: i32,
    div: i32,
}
struct AudioContext {
    clock: i32,
    global_time: i32,
    spd: i32,
    bspd: i32,
    vce: usize,
    count: i32,
    seqs: Vec<Seq>,
    seqind: usize,
    tracks: Vec<Track>,
    fin: bool,
    pattern: Vec<Vec<Option<i32>>>,
}
impl AudioContext {
    fn new() -> Self {
        let pattern = vec![
            vec![
                Some(4),
                Some(7),
                Some(11),
                Some(4),
                Some(7),
                Some(11),
                Some(7),
                Some(14),
            ],
            vec![Some(14), Some(7), Some(11), Some(6), None],
            vec![Some(0), None, Some(14), None],
            vec![Some(12), Some(7), Some(11), Some(6), None],
            vec![Some(10), Some(5), Some(3), None, Some(7)],
            vec![
                Some(5),
                Some(7),
                Some(10),
                Some(3),
                Some(7),
                Some(10),
                Some(7),
                Some(14),
            ],
            vec![
                Some(-12),
                Some(7),
                Some(14),
                Some(16),
                None,
                None,
                None,
                None,
            ],
            vec![
                Some(-12),
                Some(7),
                Some(14),
                Some(15),
                None,
                None,
                None,
                None,
            ],
            vec![
                Some(-9),
                Some(7),
                Some(14),
                Some(15),
                None,
                None,
                None,
                None,
            ],
            vec![Some(4), Some(0), Some(-1), Some(-5), Some(-1), Some(0)],
            vec![Some(0), Some(4), Some(7), Some(11), Some(7), Some(4)],
            vec![Some(3), Some(1), Some(-2), Some(-5), Some(-2), Some(1)],
            vec![Some(1), Some(5), Some(7), Some(11), Some(7), Some(5)],
            vec![Some(4), Some(2), Some(-1), Some(-5), Some(-1), Some(2)],
            vec![Some(7), Some(4), Some(2), Some(-2), Some(2), Some(4)],
            vec![Some(0), Some(4), Some(7), Some(14), Some(7), Some(4)],
            vec![Some(0), None, None],
            vec![None, Some(2), Some(3), Some(10)],
            vec![None, Some(2), Some(4), Some(11)],
            vec![None, Some(1), Some(4), Some(11)],
            vec![None, Some(1), Some(4), Some(10)],
            vec![
                None,
                Some(4),
                Some(11),
                Some(7),
                Some(9),
                Some(11),
                Some(18),
                None,
            ],
            vec![Some(0), Some(-12), None, None, None],
            vec![
                Some(-24),
                Some(0),
                Some(9),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            vec![
                Some(-24),
                Some(3),
                Some(14),
                Some(22),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        ];
        let seqs = vec![
            Seq::new(1, 90, vec![[Some(-3), Some(24), Some(5), Some(16)]], None),
            Seq::new(1, 32, vec![[Some(-3), Some(25), Some(7), Some(2)]], Some(8)),
            Seq::new(
                3,
                32,
                vec![
                    [Some(-12), Some(1), Some(0), Some(1)],
                    [Some(0), Some(2), Some(3), Some(4)],
                    [Some(-24), Some(3), Some(1), Some(4)],
                ],
                Some(12),
            ),
            Seq::new(
                3,
                32,
                vec![
                    [None, None, None, None],
                    [Some(0), Some(4), None, None],
                    [Some(-27), None, None, None],
                ],
                None,
            ),
            Seq::new(
                3,
                32,
                vec![
                    [Some(-13), None, None, None],
                    [Some(-1), None, None, None],
                    [Some(-28), None, None, None],
                ],
                None,
            ),
            Seq::new(
                3,
                32,
                vec![
                    [Some(-13), Some(6), None, None],
                    [Some(-1), Some(5), None, None],
                    [Some(-25), None, None, None],
                ],
                None,
            ),
            Seq::new(
                3,
                32,
                vec![
                    [Some(-14), None, None, None],
                    [Some(-2), None, None, None],
                    [Some(-26), None, None, None],
                ],
                None,
            ),
            Seq::new(1, 16, vec![[Some(-11), Some(7), Some(2), Some(2)]], None),
            Seq::new(1, 16, vec![[Some(-9), None, None, None]], None),
            Seq::new(1, 16, vec![[Some(-7), Some(8), None, None]], Some(13)),
            Seq::new(1, 16, vec![[Some(-7), Some(9), None, None]], Some(14)),
            Seq::new(
                2,
                24,
                vec![
                    [Some(13), Some(10), Some(4), Some(1)],
                    [Some(-11), Some(11), Some(0), Some(1)],
                ],
                Some(17),
            ),
            Seq::new(
                2,
                24,
                vec![
                    [Some(13), Some(12), None, None],
                    [Some(-11), Some(13), None, None],
                ],
                Some(15),
            ),
            Seq::new(
                2,
                24,
                vec![
                    [Some(11), Some(14), None, None],
                    [Some(-13), Some(11), None, None],
                ],
                None,
            ),
            Seq::new(
                2,
                24,
                vec![
                    [Some(9), Some(15), None, None],
                    [Some(-15), Some(16), None, None],
                ],
                None,
            ),
            Seq::new(
                2,
                16,
                vec![
                    [Some(9), Some(18), Some(6), Some(2)],
                    [Some(-15), Some(17), Some(6), Some(4)],
                ],
                Some(14),
            ),
            Seq::new(
                2,
                16,
                vec![
                    [Some(8), Some(19), None, None],
                    [Some(-16), None, None, None],
                ],
                None,
            ),
            Seq::new(
                2,
                16,
                vec![[Some(6), None, None, None], [Some(-18), None, None, None]],
                None,
            ),
            Seq::new(
                2,
                16,
                vec![
                    [Some(6), Some(20), None, None],
                    [Some(-12), None, None, None],
                ],
                Some(15),
            ),
            Seq::new(
                2,
                16,
                vec![
                    [Some(6), Some(21), None, None],
                    [Some(-24), None, None, None],
                ],
                Some(16),
            ),
            Seq::new(
                2,
                32,
                vec![
                    [Some(-7), Some(22), Some(6), Some(4)],
                    [Some(-19), Some(23), Some(6), Some(16)],
                ],
                Some(16),
            ),
        ];
        let mut res = Self {
            clock: 0,
            global_time: 0,
            spd: 12,
            bspd: 12,
            vce: 0,
            count: 0,
            seqs,
            seqind: 0,
            tracks: (0..4)
                .map(|_| Track {
                    transp: 0,
                    ind: 1,
                    div: 1,
                    instr: 0,
                })
                .collect(),
            fin: false,
            pattern,
        };
        res.play_seq();
        res
    }
    fn play_seq(&mut self) {
        if let Some(seq) = self.seqs.get(self.seqind) {
            for i in 0..seq.numvc {
                if let Some(transp) = seq.t[i][0] {
                    self.tracks[i].transp = transp
                };
                if let Some(ind) = seq.t[i][1] {
                    self.tracks[i].ind = ind
                };
                if let Some(instr) = seq.t[i][2] {
                    self.tracks[i].instr = instr
                };
                if let Some(div) = seq.t[i][3] {
                    self.tracks[i].div = div
                };
            }
            self.bspd = seq.spd.unwrap_or(self.bspd);
        } else {
            self.fin = true;
        }
    }
    fn get_note(&self, n: usize, d: Option<i32>) -> Option<i32> {
        self.pattern[n][(self.count / d.unwrap_or(1)) as usize % self.pattern[n].len()]
    }
}
struct AudioCore {
    cart: Rc<RefCell<CartContext>>,
    context: AudioContext,
    leafind: usize,
    voices: Vec<Voice>,
}
impl AudioCore {
    fn new(cart: Rc<RefCell<CartContext>>) -> Self {
        let mut voices = Vec::new();
        for i in 0..4 {
            voices.push(Voice::new(i));
            voices[i].init(&mut cart.borrow_mut());
        }
        Self {
            cart,
            context: AudioContext::new(),
            leafind: 0,
            voices,
        }
    }
    fn leaf_bloop(&mut self) {
        const TB: [i32; 6] = [-3, 4, 9, 11, 13, 16];
        self.voices[3].play(TB[self.leafind] - 12, 6);
        self.leafind += 1;
    }
    fn trig(&mut self) {
        if let Some(seq) = self.context.seqs.get(self.context.seqind) {
            for i in 0..seq.numvc {
                if let Some(note) = self.context.get_note(
                    self.context.tracks[i].ind as usize - 1,
                    Some(self.context.tracks[i].div),
                ) {
                    self.voices[self.context.vce % 4].play(
                        note + self.context.tracks[i].transp,
                        self.context.tracks[i].instr as u8,
                    );
                    self.context.vce += 1;
                }
            }
        }
        self.context.count += 1;
    }
}

struct FallSpireCore {
    gcore: GraphicsCore,
    acore: AudioCore,
    cart: Rc<RefCell<CartContext>>,
    system: Rc<RefCell<SystemContext>>,
    scene_number: usize,
    fade_amt: Float,
    rng: rand::rngs::ThreadRng,
}
impl FallSpireCore {
    fn new(cart: Rc<RefCell<CartContext>>, system: Rc<RefCell<SystemContext>>) -> Self {
        Self {
            gcore: GraphicsCore::new(cart.clone()),
            acore: AudioCore::new(cart.clone()),
            cart,
            system,
            scene_number: 0,
            fade_amt: 0.0,
            rng: rand::thread_rng(),
        }
    }
    fn next_scene(&mut self) {
        self.scene_number += 1;
        self.fade_amt = 120.0;
    }
    fn update_music(&mut self) {
        self.acore.context.clock += 1;
        self.acore.context.global_time += 1;
        if let Some(seq) = self.acore.context.seqs.get(self.acore.context.seqind) {
            let h = seq.l as Float * 0.5;
            let s = (((self.acore.context.count % seq.l) as Float - h) / h).powi(4);
            self.acore.context.spd = if self.acore.context.seqind as i32 - 1 < 14 {
                (self.acore.context.bspd as Float + s * 10.0).floor() as i32
            } else {
                self.acore.context.bspd
            };
            if self.acore.context.clock >= self.acore.context.spd {
                self.acore.context.clock = 0;
                if self.acore.context.count == seq.l {
                    self.acore.context.count = 0;
                    self.acore.context.seqind += 1;
                    self.acore.context.play_seq();
                    let seqind = self.acore.context.seqind as i32 - 1;
                    if [0, 3, 5, 7, 9, 10, 14, 16].contains(&seqind) {
                        self.next_scene();
                    }
                }
                self.acore.trig();
            }
        }
        for i in 0..4 {
            self.update_voice(i);
        }
    }
    fn update_voice(&mut self, idx: usize) {
        let context = &mut self.acore.context;
        let rng = &mut self.rng;
        let voice = &mut self.acore.voices[idx];
        const PI: Float = std::f64::consts::PI as Float;
        voice.volume *= 1.0 - voice.decay;
        let mut w = [0.0; 32];
        voice.filter.update(0.5, 1.0);
        match voice.algo {
            0 => {
                voice
                    .filter
                    .update(0.15 * (context.global_time as Float * 0.01).sin(), 1.0);
                for i in 0..32 {
                    w[i] = if voice.t == 0 {
                        voice.wave[i] + ((i * 2) % 32) as Float / 32.0
                    } else {
                        if voice.t == 7 {
                            voice.wave[i] + ((i * 3) % 32) as Float * 0.8 / 32.0
                        } else {
                            voice.wave[i]
                        }
                    };
                }
            }
            1 => {
                voice
                    .filter
                    .update(0.9 * (-voice.t as Float * 0.06).exp(), 1.0);
                for i in 0..32 {
                    w[i] = (i as i32
                        + (i as i32
                            + (voice.index as i32
                                + (context.global_time as Float * 0.20).floor() as i32))
                            % 32
                        + (i as i32
                            + (voice.index as i32
                                + (-context.global_time as Float * 0.24).floor() as i32))
                            % 32) as Float
                        / (3.0 * 16.0);
                }
            }
            2 => {
                voice.filter.update(200.0 / (voice.freq * 32.0) + 0.1, 1.0);
                for i in 0..32 {
                    w[i] = if voice.t == 0 {
                        rng.r#gen::<Float>() * 1.5
                    } else {
                        voice.wave[i] * 0.92
                            + 0.07
                                * (0.5 * voice.wave[i]
                                    + 0.25 * voice.wave[(i + 1) % 32]
                                    + 0.25 * voice.wave[(i + 31) % 32])
                            + 0.08 * (0.5 - rng.r#gen::<Float>())
                    };
                }
            }
            3 => {
                voice.vibd = 0.02 * (1.0 - (-voice.t as Float * 0.12).exp());
                for i in 0..32 {
                    w[i] = 0.7
                        * (2.0 * PI * (i as Float / 32.0)
                            + 1.2
                                * (5.0 * PI * (i as Float / 32.0)).sin()
                                * (-voice.t as Float * 0.08).exp())
                        .sin();
                }
            }
            4 => {
                voice.filter.update(
                    0.1 + 0.05 * (1.0 + context.global_time as Float * 0.01).sin(),
                    1.2,
                );
                for i in 0..32 {
                    w[i] = 0.5
                        * (2.0 * PI * (i as Float / 32.0)
                            + 2.0
                                * (4.0 * PI * (i as Float / 32.0)).sin()
                                * (-voice.t as Float * 0.04).exp())
                        .sin();
                }
            }
            5 => {
                voice
                    .filter
                    .update((0.02 + voice.t as Float * 0.001).min(0.5), 1.7);
                let r = rng.r#gen::<Float>();
                let r2 = rng.r#gen::<Float>();
                let r3 = rng.r#gen::<Float>();
                for i in 0..32 {
                    w[i] = if r3 < 0.1 {
                        voice.wave[i]
                            + r2 * 0.1
                                * (2.0 * PI * r2
                                    + (1.0 + (r * 5.0).floor()) * 2.0 * PI * i as Float / 32.0)
                                    .sin()
                    } else {
                        voice.wave[i] * 0.98
                    }
                }
            }
            6 => {
                voice.filter.update(
                    0.3 + 0.25 * (voice.index as Float + context.global_time as Float * 0.01).sin(),
                    1.2,
                );
                let v = voice.t % (context.spd * 3);
                voice.vibd = 0.02 * (1.0 - (-voice.t as Float * 0.12).exp());
                for i in 0..32 {
                    w[i] = 0.4
                        * (2.0 * PI * (i as Float / 32.0)
                            + 1.2
                                * (5.0 * PI * (i as Float / 32.0)).sin()
                                * (-voice.t as Float * 0.03).exp())
                        .sin()
                        * (0.5 + 0.5 * (-v as Float * 0.08).exp());
                }
            }
            7 => {
                voice.filter.update(
                    0.9 * (-voice.t as Float * 0.07).exp() * (1.0 - voice.index as Float * 0.2),
                    1.7,
                );
                voice.vibd = 0.02 * (1.0 - (-voice.t as Float * 0.005).exp());
                for i in 0..32 {
                    w[i] = i as Float / 48.0;
                }
            }
            _ => (),
        }
        let mut mean = 0.0;
        for i in 0..32 {
            voice.wave[i] = w[i];
            w[i] *= voice.volume;
            mean += w[i];
        }
        let mut max: Float = 0.0;
        for i in 0..32 {
            w[i] -= mean / 32.0;
            w[i] = voice.filter.process(w[i]);
            max = max.max(w[i].abs());
        }
        for i in 0..32 {
            let o = 8 + (8.0 * w[i] / max).floor() as i32;
            let o = o.clamp(0, 15);
            self.cart
                .borrow_mut()
                .poke4(0x1ff3c + 36 * voice.index + i, o as u8);
        }
        let vol = (16.0 * max).floor().min(15.0) as u8;
        self.cart
            .borrow_mut()
            .poke4(0x1ff3b + 36 * voice.index, vol);
        let f = (voice.freq
            * (1.0
                + voice.vibd
                    * (context.global_time as Float * voice.vibs + voice.index as Float).sin()))
        .floor() as u16;
        self.cart
            .borrow_mut()
            .poke(0xff9c + 18 * voice.index, (f & 0xff) as u8);
        self.cart
            .borrow_mut()
            .poke4(0x1ff3a + 36 * voice.index, (f >> 8) as u8);
        voice.t += 1;
    }
}
struct GraphicsCore {
    cart: Rc<RefCell<CartContext>>,
    palette: Vec<u8>,
    camera: Camera,
    leaf: Leaf,
    tree_model: Vec<TreeUnit>,
}

impl GraphicsCore {
    fn new(cart: Rc<RefCell<CartContext>>) -> Self {
        let palette = (0..48).map(|i| cart.borrow().peek(0x3fc0 + i)).collect();
        Self {
            cart,
            palette,
            camera: Camera::new(),
            leaf: Leaf::new(),
            tree_model: model_tree(),
        }
    }
    fn fade(&self, amt: i32) {
        for i in 3..48 {
            self.cart.borrow_mut().poke(
                0x3fc0 + i,
                (self.palette[i] as i32 + amt).min(255).max(0) as u8,
            );
        }
    }
    fn scanline_offset(&self, amt: u8) {
        self.cart.borrow_mut().poke(0x3ff9, amt);
    }
    fn palette_index(&self, inds: &[u8]) {
        for i in 0..inds.len() {
            self.cart.borrow_mut().poke4(0x3ff0 * 2 + i + 1, inds[i]);
        }
        for i in inds.len() + 1..16 {
            self.cart.borrow_mut().poke4(0x3ff0 * 2 + i, i as u8);
        }
    }
    fn leaf_offset(
        &self,
        x: Float,
        y: Float,
        ang: Float,
        xs: Float,
        ys: Float,
        off: i32,
        col: u8,
        warp: Option<Float>,
    ) {
        let warp = warp.unwrap_or(0.0);
        if y - ys.abs() * 14.0 > 136.0 {
            return;
        }
        let r = (self.leaf.r[0] + off).max(0) as Float;
        let a = self.leaf.th[0] + ang;
        let mut xp = x + r * a.cos() * xs;
        let mut yp = y + r * a.sin() * ys - (r / 14.0) * (r / 14.0) * warp;
        for i in 1..13 {
            let r = (self.leaf.r[i] + off).max(0) as Float;
            let a = self.leaf.th[i] + ang;
            let xx = x + r * a.cos() * xs;
            let yy = y + r * a.sin() * ys - (r / 14.0) * (r / 14.0) * warp;
            self.cart.borrow_mut().tri(
                x as f32, y as f32, xp as f32, yp as f32, xx as f32, yy as f32, col,
            );
            // rp = r
            xp = xx;
            yp = yy;
        }
    }
    fn maple_leaf(
        &self,
        x: Float,
        y: Float,
        ang: Float,
        xs: Option<Float>,
        ys: Option<Float>,
        warp: Option<Float>,
    ) {
        let xs = xs.unwrap_or(1.0);
        let ys = ys.unwrap_or(1.0);
        self.leaf_offset(x, y, ang, xs, ys, 0, 9, warp);
        self.leaf_offset(x, y, ang, xs, ys, -4, 11, warp);
    }
    fn leaf_ripple(
        &self,
        x: Float,
        y: Float,
        ang: Float,
        xs: Float,
        ys: Float,
        rad: Float,
        w: Option<Float>,
        mag: Float,
        u: Option<Float>,
        v: Option<Float>,
    ) {
        let mut cart = self.cart.borrow_mut();
        let u = u.unwrap_or(0.0);
        let v = v.unwrap_or(68.0 * 8.0);
        let w = w.unwrap_or(5.0);
        let rp = (self.leaf.r[0] as Float + rad).max(0.0);
        let a = self.leaf.th[0] + ang;
        let c = a.cos() * xs;
        let s = a.sin() * ys;
        let mut x2 = x + rp * c;
        let mut y2 = y + rp * s;
        let mut x1 = x2 + w * c;
        let mut y1 = y2 + w * s;
        let mut x3 = x2 - w * c;
        let mut y3 = y2 - w * s;
        for i in 1..13 {
            let rp = (self.leaf.r[i] as Float + rad).max(0.0);
            let a = self.leaf.th[i] + ang;
            let c = a.cos() * xs;
            let s = a.sin() * ys;
            let x5 = x + rp * c;
            let y5 = y + rp * s;
            let x4 = x5 + w * c;
            let y4 = y5 + w * s;
            let x6 = x5 - w * c;
            let y6 = y5 - w * s;
            cart.textri(
                x1 as f32,
                y1 as f32,
                x2 as f32,
                y2 as f32,
                x4 as f32,
                y4 as f32,
                (x1 + u) as f32,
                (y1 + v) as f32,
                (x2 + u + (mag * 0.5).trunc()) as f32,
                (y2 - mag + v) as f32,
                (x4 + u) as f32,
                (y4 + v) as f32,
                true,
                255,
            );
            cart.textri(
                x2 as f32,
                y2 as f32,
                x4 as f32,
                y4 as f32,
                x5 as f32,
                y5 as f32,
                (x2 + u + (mag * 0.5).trunc()) as f32,
                (y2 - mag + v) as f32,
                (x4 + u) as f32,
                (y4 + v) as f32,
                (x5 + u + (mag * 0.5).trunc()) as f32,
                (y5 - mag + v) as f32,
                true,
                255,
            );
            cart.textri(
                x3 as f32,
                y3 as f32,
                x2 as f32,
                y2 as f32,
                x6 as f32,
                y6 as f32,
                (x3 + u) as f32,
                (y3 + v) as f32,
                (x2 + u + (mag * 0.5).trunc()) as f32,
                (y2 - mag + v) as f32,
                (x6 + u) as f32,
                (y6 + v) as f32,
                true,
                255,
            );
            cart.textri(
                x2 as f32,
                y2 as f32,
                x6 as f32,
                y6 as f32,
                x5 as f32,
                y5 as f32,
                (x2 + u + (mag * 0.5).trunc()) as f32,
                (y2 - mag + v) as f32,
                (x6 + u) as f32,
                (y6 + v) as f32,
                (x5 + u + (mag * 0.5).trunc()) as f32,
                (y5 - mag + v) as f32,
                true,
                255,
            );
            x1 = x4;
            y1 = y4;
            x2 = x5;
            y2 = y5;
            x3 = x6;
            y3 = y6;
        }
    }
    fn tree1(&self, x: i32, y: i32) {
        self.cart.borrow_mut().map(1, 1, 20, 16, x, y, 6, 1);
    }
    fn tree2(&self, x: i32, y: i32) {
        self.cart.borrow_mut().map(22, 1, 13, 16, x, y, 6, 1);
    }
    fn tower_repeat(&self, x: i32, y: i32) {
        let mut cart = self.cart.borrow_mut();
        let mut i = y - 64;
        while i >= -64 {
            cart.map(58, 0, 1, 8, x, i, 255, 1);
            i -= 64;
        }
        /*for i in (64..=64 - y).step_by(64) {
            cart.map(58, 0, 1, 8, x, -i, 255, 1);
        }*/
    }
    fn cloud_repeat(&self, x: i32, y: i32) {
        let mut cart = self.cart.borrow_mut();
        cart.map(60, 34, 30, 17, x % 240, y % 136, 6, 1);
        cart.map(60, 34, 30, 17, x % 240 - 240, y % 136, 6, 1);
        cart.map(60, 34, 30, 17, x % 240, y % 136 - 136, 6, 1);
        cart.map(60, 34, 30, 17, x % 240 - 240, y % 136 - 136, 6, 1);
        cart.map(60, 34, 30, 17, x % 240 + 240, y % 136, 6, 1);
        cart.map(60, 34, 30, 17, x % 240, y % 136 + 136, 6, 1);
        cart.map(60, 34, 30, 17, x % 240 + 240, y % 136 + 136, 6, 1);
    }
    fn tower_top(&self, x: i32, y: i32) {
        self.cart.borrow_mut().map(92, 1, 4, 30, x, y, 6, 1);
    }
    fn bgtree1(&self, x: i32, y: i32) {
        self.cart.borrow_mut().map(0, 24, 14, 10, x, y, 6, 1);
    }
    fn bgtree2(&self, x: i32, y: i32) {
        self.cart.borrow_mut().map(14, 23, 12, 11, x, y, 6, 1);
    }
    fn title(&self, x: Option<i32>, y: Option<i32>, index: Option<i32>) {
        let x = x.unwrap_or(0);
        let y = y.unwrap_or(0);
        let index = index.unwrap_or(10);
        const PALETTES: [[u8; 2]; 10] = [
            [6, 6],
            [5, 6],
            [4, 5],
            [11, 7],
            [15, 15],
            [7, 15],
            [12, 15],
            [11, 7],
            [10, 7],
            [9, 12],
        ];
        let index = index.clamp(1, 10) as usize - 1;
        self.palette_index(&PALETTES[index]);
        self.cart
            .borrow_mut()
            .map(31, 19, 28, 11, x + 8, y + 16, 6, 1);
        self.palette_index(&[]);
    }
    fn terrain1(&self, x: i32, y: i32, h: Option<i32>) {
        let mut cart = self.cart.borrow_mut();
        let h = h.unwrap_or(8);
        let xmod = (x % 240 + 240) % 240;
        cart.map(60, 0, 30 - xmod / 8, h, xmod, y, 6, 1);
        cart.map(
            60 + 29 - xmod / 8,
            0,
            xmod / 8 + 1,
            h,
            (x % 8 + 8) % 8 - 8,
            y,
            6,
            1,
        );
    }
    fn terrain2(&self, x: i32, y: i32, h: Option<i32>) {
        let mut cart = self.cart.borrow_mut();
        let h = h.unwrap_or(9);
        let xmod = (x % 240 + 240) % 240;
        cart.map(60, 8, 30 - xmod / 8, h, xmod, y, 6, 1);
        cart.map(
            60 + 29 - xmod / 8,
            8,
            xmod / 8 + 1,
            h,
            (x % 8 + 8) % 8 - 8,
            y,
            6,
            1,
        );
    }
}

struct LeafStruct {
    x: Float,
    y: Float,
    z: Float,
    ang: Float,
    dz: Float,
    warp: Float,
    da: Float,
}

struct Ripple {
    x: Float,
    y: Float,
    ang: Float,
    t: i32,
}

trait FallSpireScene {
    fn update(&mut self);
    fn scanline(&mut self, _row: i32) {}
    fn overlay(&mut self) {}
}

struct ScenePond {
    core: Rc<RefCell<FallSpireCore>>,
    t: i32,
    ytilt: Float,
    timer: Float,
    leaves: Vec<LeafStruct>,
    ripples: Vec<Ripple>,
    pan1_time: i32,
    pan2_time: i32,
    title_time: i32,
}

impl ScenePond {
    fn new(core: Rc<RefCell<FallSpireCore>>) -> Self {
        let t = 0;
        let ytilt = 0.5;
        let timer = 120.0;
        let leaves = Vec::new();
        let ripples = Vec::new();
        let pan1_time = 500;
        let pan2_time = 1000;
        let title_time = pan2_time + 200;
        Self {
            core,
            t,
            ytilt,
            timer,
            leaves,
            ripples,
            pan1_time,
            pan2_time,
            title_time,
        }
    }
    fn rand(&self) -> Float {
        self.core.borrow_mut().rng.r#gen()
    }
    fn leaf(&mut self, x: Float, y: Float) {
        let camera_y = self.core.borrow().gcore.camera.y;
        self.leaves.push(LeafStruct {
            x: x + camera_y * 0.125,
            y,
            z: 150.0 - camera_y,
            ang: self.rand() * 3.14,
            dz: -0.75,
            warp: 5.0,
            da: (self.rand() - 0.5) * 0.02,
        });
    }
    fn ripple(&mut self, x: Float, y: Float, ang: Float) {
        self.core.borrow_mut().acore.leaf_bloop();
        self.ripples.push(Ripple { x, y, ang, t: 0 });
        self.ripples.push(Ripple { x, y, ang, t: -20 });
        self.ripples.push(Ripple { x, y, ang, t: -40 });
    }
}

impl FallSpireScene for ScenePond {
    fn update(&mut self) {
        self.core.borrow().cart.borrow_mut().cls(6);
        self.timer -= 1.0;
        if self.timer <= 0.0 && self.t < self.title_time - 60 {
            // self.core.borrow_mut().rng.reseed();
            let x = self.core.borrow_mut().rng.gen_range(-20..=120) as Float;
            let y = self.core.borrow_mut().rng.gen_range(1..=80) as Float;
            self.leaf(x, y);
            self.timer = (self.t as Float).sin() * 50.0 + (240.0 - self.t as Float * 0.2).max(90.0);
        }
        self.core.borrow_mut().gcore.camera.y = -100.0
            * smootherstep((self.t - self.pan1_time) as Float / 400.0)
            - 300.0 * smootherstep((self.t - self.pan2_time) as Float / 400.0);
        let camera_y = self.core.borrow().gcore.camera.y;
        self.ytilt = 0.7 + (camera_y / 200.0).max(-0.5);
        self.t += 1;
        self.core
            .borrow()
            .cart
            .borrow_mut()
            .map(0, 68, 30, 17, 0, -camera_y as i32, 255, 1);
        for r in self.ripples.iter_mut() {
            let rt = r.t as Float;
            r.t += 1;
            if rt > 0.0 {
                self.core.borrow().gcore.leaf_ripple(
                    r.x,
                    r.y * self.ytilt - camera_y,
                    r.ang,
                    1.0,
                    self.ytilt,
                    rt * 0.5,
                    Some(5.0),
                    5.0 - rt / 40.0,
                    Some(0.0),
                    Some(68.0 * 8.0 + camera_y),
                );
            }
        }
        self.ripples.retain(|r| r.t <= 199);
        let mut params = Vec::new();
        for l in self.leaves.iter_mut() {
            l.z += l.dz;
            l.ang += l.da;
            l.x += 0.1;
            l.y += 0.05;
            if l.z <= 1.0 {
                l.z = 1.0;
                if l.dz != 0.0 {
                    l.dz = 0.0;
                    params.push((l.x, l.y, l.ang));
                }
                if l.warp > 1.0 {
                    l.warp *= 0.95;
                }
            } else {
                let t = self.t as Float;
                l.warp += t.sin() * 0.3;
                l.x += (t / 60.0).cos() / 3.0 + 0.1;
                l.y += (t / 70.0).sin() * 0.2;
                if l.y < 5.0 {
                    l.y += (5.0 - l.y) * 0.2;
                }
            }
        }
        for p in params {
            self.ripple(p.0, p.1, p.2);
        }
        for l in self.leaves.iter() {
            self.core.borrow().gcore.leaf_offset(
                l.x,
                l.y * self.ytilt + l.z - camera_y,
                -l.ang,
                1.0,
                -self.ytilt,
                0,
                8,
                Some(-l.warp),
            );
        }
        self.leaves.retain(|l| l.x <= 280.0);
    }
    fn scanline(&mut self, row: i32) {
        let camera_y = self.core.borrow().gcore.camera.y;
        if row as Float > camera_y {
            let t = self.core.borrow().system.borrow().time();
            self.core.borrow().gcore.scanline_offset(
                (((row as Float + camera_y) / 3.0 + t as Float / 200.0).sin() * 2.0) as u8,
            )
        } else {
            self.core.borrow().gcore.scanline_offset(0);
        }
    }
    fn overlay(&mut self) {
        let camera_y = self.core.borrow().gcore.camera.y;
        let yy = -camera_y * 0.6 - self.ytilt * 150.0;
        self.core.borrow().gcore.terrain1(
            0,
            yy.round() as i32,
            Some((-yy - camera_y) as i32 / 8 - 1),
        );
        let yy = -camera_y * 0.8 - self.ytilt * 50.0;
        self.core.borrow().gcore.bgtree1(0, -80 + yy as i32);
        self.core.borrow().gcore.bgtree2(14 * 8, -88 + yy as i32);
        self.core.borrow().cart.borrow_mut().rect(
            0,
            yy as i32,
            240,
            (-yy - camera_y - 2.0).round() as i32,
            2,
        );
        self.core.borrow().gcore.tree1(8, -136 - camera_y as i32);
        self.core
            .borrow()
            .cart
            .borrow_mut()
            .rect(0, -8 - camera_y as i32, 240, 8, 1);
        self.core.borrow().cart.borrow_mut().line(
            0.0,
            (-camera_y - 1.0) as f32,
            240.0,
            (-camera_y - 1.0) as f32,
            2,
        );
        for l in self.leaves.iter() {
            self.core.borrow().gcore.maple_leaf(
                l.x,
                l.y * self.ytilt - l.z - camera_y,
                l.ang,
                Some(1.0),
                Some(self.ytilt),
                Some(l.warp),
            );
        }
        if self.t > self.title_time {
            self.core
                .borrow()
                .gcore
                .title(Some(0), Some(0), Some((self.t - self.title_time) / 5));
        }
    }
}

struct SceneForest {
    core: Rc<RefCell<FallSpireCore>>,
    t: i32,
    pan_duration: i32,
    title_vanish_time: i32,
}
impl SceneForest {
    fn new(core: Rc<RefCell<FallSpireCore>>) -> Self {
        Self {
            core,
            t: 0,
            pan_duration: 1100,
            title_vanish_time: 340,
        }
    }
}
impl FallSpireScene for SceneForest {
    fn update(&mut self) {
        let camera_x = 400.0 - 3.0 * easeout((self.t - self.pan_duration) as Float, Some(200.0));
        let camera_y = -easeout((self.t - 400) as Float, Some(200.0));
        let xi = camera_x.floor() as i32;
        let yi = camera_y.floor() as i32;
        self.core.borrow().cart.borrow_mut().cls(6);
        self.core.borrow_mut().gcore.camera.x = camera_x;
        self.core.borrow_mut().gcore.camera.y = camera_y;
        let core = &self.core.borrow().gcore;
        core.palette_index(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 7, 7, 13, 14, 7]);
        core.cart
            .borrow_mut()
            .map(21, 35, 26, 12, -xi / 8 + self.t / 20, 20 - yi / 8, 6, 1);
        core.palette_index(&[1, 3, 4]);
        core.cart
            .borrow_mut()
            .map(0, 34, 20, 12, 120 - xi / 7 - self.t / 10, 30 - yi / 7, 6, 1);
        core.tower_repeat(
            120 - (camera_x / 6.0).floor() as i32,
            120 - (camera_y / 6.0).floor() as i32,
        );
        core.cart.borrow_mut().map(
            21,
            35,
            26,
            12,
            (-camera_x / 6.0).floor() as i32 + self.t / 8,
            35 - (camera_y / 6.0).floor() as i32,
            6,
            1,
        );
        core.palette_index(&[1, 2, 4, 4, 11]);
        core.terrain1(
            (-camera_x / 4.0).floor() as i32 + 100,
            80 - (camera_y / 4.0).floor() as i32,
            None,
        );
        core.palette_index(&[1, 2, 3, 4, 4]);
        core.terrain1(
            (-camera_x / 2.0).floor() as i32,
            95 - (camera_y / 2.0).floor() as i32,
            None,
        );
        core.palette_index(&[]);
        let xp = -(2.0 * camera_x) as i32 / 3;
        let yp = -(2.0 * camera_y) as i32 / 3;
        if camera_x < 0.0 {
            core.bgtree1(xp % 360 - 120, 56 + yp);
            core.bgtree2((xp + 100) % 360 - 120, 44 + yp);
            core.bgtree1((xp + 230) % 360 - 120, 52 + yp);
        } else {
            core.bgtree1(xp - 120, 56 + yp);
            core.bgtree2(xp + 100 - 120, 44 + yp);
            core.bgtree1(xp + 230 - 120, 52 + yp);
        }
        core.cart.borrow_mut().spr(
            257 + ((self.t / 15) % 3) * 3,
            (600.0 - camera_x) as i32,
            (120.0 - 24.0 - camera_y) as i32,
            6,
            1,
            0,
            0,
            3,
            3,
        );
        core.terrain2(-xi, (120.0 - camera_y) as i32, None);
        core.palette_index(&[1, 1]);
        let xp = -xi;
        let yp = -yi;
        if camera_x < 0.0 {
            core.tree1(xp % 700 - 200, 8 + yp);
            core.tree2((xp + 240) % 700 - 200, 8 + yp);
            core.tree1((xp + 370) % 700 - 200, 16 + yp);
        } else {
            core.tree1(xp - 200, 8 + yp);
            core.tree2((xp + 240) - 200, 8 + yp);
            core.tree1((xp + 370) - 200, 16 + yp);
        }
        core.palette_index(&[]);
        self.t += 1;
    }
    fn overlay(&mut self) {
        if self.t < self.title_vanish_time - 5 {
            self.core.borrow().gcore.title(
                Some(0),
                Some(0),
                Some((self.title_vanish_time - self.t) / 5),
            );
        }
    }
}

struct SceneFlight {
    core: Rc<RefCell<FallSpireCore>>,
    t: i32,
}
impl SceneFlight {
    const SF: Float = 30.0;
    fn new(core: Rc<RefCell<FallSpireCore>>) -> Self {
        Self { core, t: 0 }
    }
}
impl FallSpireScene for SceneFlight {
    fn update(&mut self) {
        self.core.borrow().cart.borrow_mut().cls(6);
        let core = &mut self.core.borrow_mut().gcore;
        let camera = &mut core.camera;
        let t = self.t as Float;
        camera.ytilt =
            60.0 + 10.0 * (t * 0.01).cos() - 200.0 * (1.0 - smootherstep((t - 200.0) / 400.0));
        let horizon_f = camera.ytilt;
        let horizon = horizon_f as i32;
        camera.z = t * 2.0;
        let camera_z = camera.z;
        camera.y =
            100.0 + 50.0 * (t * 0.01).sin() + 500.0 * (1.0 - smootherstep((t - 200.0) / 400.0));
        let camera_y = camera.y;
        core.palette_index(&[]);
        let towerx = 140 + self.t / 80;
        core.tower_repeat(towerx, horizon + 74 - (self.t / 30) % 64);
        core.cart.borrow_mut().tri(
            towerx as f32,
            (horizon_f + 10.0) as f32,
            (towerx - 4) as f32,
            136.0,
            (towerx + 12) as f32,
            136.0,
            5,
        );
        core.cart.borrow_mut().tri(
            (towerx + 8) as f32,
            (horizon_f + 10.0) as f32,
            (towerx - 4) as f32,
            136.0,
            (towerx + 12) as f32,
            136.0,
            5,
        );
        core.cart.borrow_mut().tri(
            towerx as f32,
            (horizon_f + 10.0) as f32,
            (towerx + 8) as f32,
            (horizon_f + 10.0) as f32,
            (towerx + 4) as f32,
            160.0,
            3,
        );
        for i in 0..=(horizon + 10).clamp(0, 136) {
            let z = -Self::SF * (-200.0 + camera_y * 0.2) / (horizon_f - i as Float + 20.0);
            let ty = camera_z * 0.5 + z;
            let u = t as Float * 0.2 + (120 + (((ty as i32 / 32) * 3) % 8) * 240) as Float;
            let v = 123.0 * 8.0 + ty % 32.0;
            core.cart.borrow_mut().textri(
                0.0,
                i as f32,
                240.0,
                i as f32,
                240.0,
                (i + 1) as f32,
                (u - z) as f32,
                v as f32,
                (u + z) as f32,
                v as f32,
                (u + z) as f32,
                v as f32,
                true,
                6,
            );
        }
        core.palette_index(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 7, 7, 13, 14, 7]);
        for i in (horizon + 10).clamp(0, 136)..=136 {
            let z = -Self::SF * (-200.0 + camera_y * 0.2) / (i as Float - horizon_f);
            let ty = camera_z * 0.5 + z;
            let u = t as Float * 0.2
                + (120 + (((((ty / 32.0).floor() as i32) * 3) % 8 + 8) % 8) * 240) as Float;
            let v = 123.0 * 8.0 + (ty % 32.0 + 32.0) % 32.0;
            core.cart.borrow_mut().textri(
                0.0,
                i as f32,
                240.0,
                i as f32,
                240.0,
                (i + 1) as f32,
                (u - z) as f32,
                v as f32,
                (u + z) as f32,
                v as f32,
                (u + z) as f32,
                v as f32,
                true,
                6,
            );
        }
        self.t += 1;
    }
    fn scanline(&mut self, row: i32) {
        let core = &self.core.borrow().gcore;
        let row = row as Float;
        if row >= core.camera.ytilt + 10.0 {
            core.scanline_offset(
                ((50.0 * (row - core.camera.ytilt).ln()
                    - self.core.borrow().system.borrow().time() as Float / 20.0)
                    .sin()
                    * (40.0 + row - core.camera.ytilt)
                    / 60.0) as u8,
            );
        } else {
            core.scanline_offset(0);
        }
    }
    fn overlay(&mut self) {
        let core = &self.core.borrow().gcore;
        core.palette_index(&[]);
        let horizon = core.camera.ytilt;
        for i in (horizon as i32 + 10).max(0)..=136 {
            let z = -Self::SF * core.camera.y / (i as Float - horizon);
            let ty = -core.camera.z - z;
            let u = ty * 0.2
                + (ty * 0.01).sin() * 50.0
                + (120 + ((((ty / 32.0).floor() as i32 * 3) % 8 + 8) % 8) * 240) as Float;
            let v = 119.0 * 8.0 + (ty % 32.0 + 32.0) % 32.0;
            core.cart.borrow_mut().textri(
                0.0,
                i as f32,
                240.0,
                i as f32,
                240.0,
                (i + 1) as f32,
                (u - z) as f32,
                v as f32,
                (u + z) as f32,
                v as f32,
                (u + z) as f32,
                v as f32,
                true,
                14,
            );
        }
    }
}

struct SceneTowerBase {
    core: Rc<RefCell<FallSpireCore>>,
    t: i32,
}
impl SceneTowerBase {
    fn new(core: Rc<RefCell<FallSpireCore>>) -> Self {
        Self { core, t: 0 }
    }
}
impl FallSpireScene for SceneTowerBase {
    fn update(&mut self) {
        const ASCENT_TIME: Float = 400.0;
        let core = &mut self.core.borrow_mut().gcore;
        let camera = &mut core.camera;
        camera.x = -easeout((self.t - 500) as Float, Some(400.0));
        camera.y = -40.0 + 30.0 * smootherstep(self.t as Float / ASCENT_TIME)
            - (2.0 * easeout(ASCENT_TIME - self.t as Float, Some(200.0))).trunc();
        let cx = camera.x;
        let cy = camera.y;
        core.cart.borrow_mut().cls(6);
        core.palette_index(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 7, 7, 13, 14, 7]);
        core.cloud_repeat(-cx as i32 / 5, -cy as i32 / 5);
        core.palette_index(&[]);
        core.cloud_repeat(-cx as i32 / 3 + 100, -cy as i32 / 3 + 50);
        core.terrain1(-cx as i32 / 2, 30 - cy as i32 / 2, None);
        core.cart
            .borrow_mut()
            .rect(0, 64 + (-3 * (cy as i32 / 4)), 240, 80, 11);
        {
            let mut cart = core.cart.borrow_mut();
            if cy > -68.0 {
                cart.tri(
                    (104.0 - cx) as f32,
                    (68.0 - cy) as f32,
                    (120.0 - cx * 2.0) as f32,
                    (136.0 - cy * 2.0) as f32,
                    (170.0 - cx * 2.0) as f32,
                    (136.0 - cy * 2.0) as f32,
                    3,
                );
                cart.tri(
                    (136.0 - cx) as f32,
                    (68.0 - cy) as f32,
                    (220.0 - cx * 2.0) as f32,
                    (136.0 - cy * 2.0) as f32,
                    (170.0 - cx * 2.0) as f32,
                    (136.0 - cy * 2.0) as f32,
                    3,
                );
                cart.tri(
                    (104.0 - cx) as f32,
                    (68.0 - cy) as f32,
                    (136.0 - cx) as f32,
                    (68.0 - cy) as f32,
                    (170.0 - cx * 2.0) as f32,
                    (136.0 - cy * 2.0) as f32,
                    3,
                );
            }
            cart.map(92, 25, 4, 9, 104 - cx as i32, -cy as i32, 6, 1);
            let mut i = -cy as i32;
            while i > 0 {
                i -= 56;
                cart.map(92, 18, 4, 7, 104 - cx as i32, i, 255, 1);
            }
        }
        core.palette_index(&[]);
        self.t += 1;
    }
    fn scanline(&mut self, _row: i32) {
        self.core.borrow().gcore.scanline_offset(0);
    }
}

struct SceneTower {
    core: Rc<RefCell<FallSpireCore>>,
    t: i32,
}
impl SceneTower {
    const PALETTE_INDEXES: [[u8; 7]; 24] = [
        [8, 8, 8, 3, 4, 6, 3],
        [8, 2, 3, 4, 11, 6, 3],
        [8, 2, 4, 4, 11, 6, 4],
        [8, 2, 4, 4, 11, 6, 11],
        [8, 2, 4, 4, 11, 6, 7],
        [8, 2, 3, 11, 11, 6, 7],
        [8, 2, 3, 11, 4, 6, 11],
        [8, 2, 3, 4, 4, 6, 11],
        [8, 2, 3, 4, 4, 6, 11],
        [8, 2, 3, 4, 4, 6, 4],
        [8, 2, 3, 4, 3, 6, 4],
        [8, 2, 2, 3, 3, 6, 4],
        [8, 2, 2, 3, 2, 6, 4],
        [8, 2, 8, 2, 8, 6, 3],
        [8, 2, 8, 8, 8, 6, 3],
        [2, 2, 8, 8, 8, 6, 3],
        [2, 2, 8, 8, 8, 6, 3],
        [2, 2, 8, 8, 8, 6, 3],
        [2, 2, 8, 8, 8, 6, 3],
        [2, 2, 8, 8, 8, 6, 3],
        [2, 2, 8, 8, 8, 6, 3],
        [2, 2, 8, 8, 8, 6, 3],
        [8, 2, 8, 8, 8, 6, 3],
        [8, 8, 8, 8, 3, 6, 3],
    ];
    fn new(core: Rc<RefCell<FallSpireCore>>) -> Self {
        Self { core, t: 0 }
    }
}
impl FallSpireScene for SceneTower {
    fn update(&mut self) {
        self.t += 1;
        let core = &self.core.borrow().gcore;
        core.cart.borrow_mut().cls(6);
        let ang = 240.1 + 0.5 * self.t as Float;
        core.palette_index(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 7, 7, 13, 14, 7]);
        core.cloud_repeat((ang * 0.5 + 120.0) as i32, self.t / 5);
        core.palette_index(&[]);
        core.cloud_repeat((ang + self.t as Float * 0.3) as i32, self.t / 5);
        let rad = 76 / 2;
        let xoff = 120 - rad;
        let flicker = if core.cart.borrow().btn(4) { 0.0 } else { 0.5 };
        for i in 0..=rad * 2 {
            let as_ = (i as Float / rad as Float - 1.0).asin();
            let tx = 120.0 * 8.0 + (240.0 + rad as Float * as_ + ang) % 240.0;
            let ty = 136 - (self.t % 136);
            core.palette_index(
                &Self::PALETTE_INDEXES[((1.0 + (self.t % 2) as Float * flicker + 5.0 * as_).floor()
                    as i32
                    + 6) as usize
                    % Self::PALETTE_INDEXES.len()],
            );
            core.cart.borrow_mut().textri(
                (i + xoff) as f32,
                0.0,
                (i + xoff + 1) as f32,
                136.0,
                (i + xoff + 2) as f32,
                0.0,
                tx as f32,
                ty as f32,
                tx as f32,
                (ty + 136) as f32,
                tx as f32,
                ty as f32,
                true,
                255,
            );
        }
    }
    fn scanline(&mut self, _row: i32) {
        self.core.borrow().gcore.scanline_offset(0);
    }
    fn overlay(&mut self) {
        self.core.borrow().gcore.palette_index(&[]);
    }
}

struct SceneTowerTop {
    core: Rc<RefCell<FallSpireCore>>,
    t: i32,
}
impl SceneTowerTop {
    fn new(core: Rc<RefCell<FallSpireCore>>) -> Self {
        Self { core, t: 0 }
    }
}
impl FallSpireScene for SceneTowerTop {
    fn update(&mut self) {
        let core = &self.core.borrow().gcore;
        core.palette_index(&[]);
        core.cart.borrow_mut().cls(6);
        let interp = smootherstep((self.t + 400) as Float / 800.0);
        core.palette_index(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 7, 7, 13, 14, 7]);
        core.cloud_repeat(self.t / 4 + 120, (interp * 40.0) as i32);
        core.palette_index(&[]);
        core.cloud_repeat(self.t * 4 / 5, (interp * 100.0) as i32);
        core.tower_top(120 - 16, (200.0 * interp) as i32 - 160);
        self.t += 1;
    }
}

fn tree_distance_field(x: i32, y: i32, z: i32) -> Float {
    let y = (y - 20) as Float * 1.25;
    let dist1 = (((x + 15) * (x + 15) + z * z) as Float + y * y).sqrt() - 45.0;
    let dist2 = (((x - 15) * (x - 15) + (z - 10) * (z - 10)) as Float + (y - 30.0) * (y - 30.0))
        .sqrt()
        - 25.0;
    let dist3 = (((x - 15) * (x - 15) + (z + 15) * (z + 15)) as Float + (y - 40.0) * (y - 40.0))
        .sqrt()
        - 20.0;
    smoothmin(smoothmin(dist1, dist2, Some(10.0)), dist3, Some(10.0))
}
fn in_trunk(x: i32, y: i32, z: i32) -> bool {
    x * x + z * z + y < 160
}

struct TreeUnit {
    x: Float,
    y: Float,
    z: Float,
    sp: i32,
}
fn model_tree() -> Vec<TreeUnit> {
    let mut rng = rand::thread_rng(); // should be deterministic
    let res = 12;
    let mut tree_model = Vec::new();
    for x in (-60..=60).step_by(res) {
        for y in (-45..=0).step_by(5) {
            for z in (-60..=60).step_by(res) {
                let xx = (x * 20) as Float / (y + 60) as Float + rng.gen_range(-3..=3) as Float;
                let yy = (y + rng.gen_range(-1..=1)) as Float;
                let zz = (z * 20) as Float / (y + 60) as Float + rng.gen_range(-3..=3) as Float;
                if in_trunk(x, y + 45, z) {
                    tree_model.push(TreeUnit {
                        x: xx - 10.0,
                        y: yy,
                        z: zz,
                        sp: 352 + rng.gen_range(0..=1) * 2,
                    })
                }
            }
        }
    }
    for x in (-60..=60).step_by(res) {
        for y in (-45..=0).step_by(5) {
            for z in (-60..=60).step_by(res) {
                let xx = x + rng.gen_range(-5..=5);
                let yy = y + rng.gen_range(-5..=5);
                let zz = z + rng.gen_range(-5..=5);
                let dist = tree_distance_field(xx, yy, zz);
                if dist < 0.0 && dist > -20.0 {
                    let grad = tree_distance_field(xx + 1, yy - 2, zz + 1) - dist;
                    let sp = if grad > 0.5 + rng.r#gen::<Float>() * 3.0 {
                        320
                    } else {
                        if grad < -1.0 - rng.r#gen::<Float>() * 3.0 {
                            328
                        } else {
                            324
                        }
                    };
                    tree_model.push(TreeUnit {
                        x: xx as Float,
                        y: yy as Float,
                        z: zz as Float,
                        sp: sp + 2 * rng.gen_range(0..2),
                    });
                }
            }
        }
    }

    tree_model
}

struct SceneTree {
    core: Rc<RefCell<FallSpireCore>>,
    t: i32,
}
impl SceneTree {
    fn new(core: Rc<RefCell<FallSpireCore>>) -> Self {
        Self { core, t: 0 }
    }
    const OCTAGON_X: [Float; 8] = [68.0, 68.0, 28.0, -28.0, -68.0, -68.0, -28.0, 28.0];
    const BLOW_AWAY_TIME: Float = 1300.0;
}
impl FallSpireScene for SceneTree {
    fn update(&mut self) {
        let core = &mut self.core.borrow_mut().gcore;
        let camera = &mut core.camera;
        let cx = 0.0;
        let cy = easeout((self.t - 800) as Float, Some(100.0)) * 0.25
            - easeout((self.t - 400) as Float, Some(100.0));
        camera.x = cx;
        camera.y = cy;
        core.palette_index(&[]);
        core.cart.borrow_mut().cls(6);
        let theta = self.t as Float * 0.01;
        let phi = theta.sin() * 0.25 + 0.5;
        let cc = theta.cos();
        let ss = theta.sin();
        let cph = phi.cos();
        let sph = phi.sin();

        core.cloud_repeat((-120.0 * theta) as i32, (-120.0 * phi) as i32);

        let mut cart = core.cart.borrow_mut();

        let mut tower_coords = Vec::new();

        for i in 1..=8 {
            let x1 = Self::OCTAGON_X[i % 8];
            const Y1: Float = -45.0;
            let z1 = Self::OCTAGON_X[(i + 2) % 8];
            let x2 = x1 * cc - z1 * ss;
            let z2 = x1 * ss + z1 * cc;
            const Y2: Float = Y1;

            let y1 = Y2 * cph + z2 * sph;
            let z1 = -Y2 * sph + z2 * cph;
            let x1 = x2;
            tower_coords.push((120.0 + x1 - cx, 67.0 - y1 - cy, z1));
        }
        let mut x1 = tower_coords[0].0;
        let mut y1 = tower_coords[0].1;
        for i in 1..=8 {
            let x2 = tower_coords[i % 8].0;
            let y2 = tower_coords[i % 8].1;
            let texoffset = (i % 2) * 80;
            if x1 > x2 {
                cart.textri(
                    x1 as f32 + 1.0,
                    y1 as f32,
                    x2 as f32,
                    y2 as f32,
                    x1 as f32 + 1.0,
                    (y1 + 160.0 * cph) as f32,
                    (200 * 8 + texoffset) as f32,
                    0.0,
                    (207 * 8 + texoffset) as f32,
                    0.0,
                    (200 * 8 + texoffset) as f32,
                    160.0,
                    true,
                    255,
                );
                cart.textri(
                    x2 as f32,
                    (y2 + 160.0 * cph) as f32,
                    x2 as f32,
                    y2 as f32,
                    x1 as f32 + 1.0,
                    (y1 + 160.0 * cph) as f32,
                    (207 * 8 + texoffset) as f32,
                    160.0,
                    (207 * 8 + texoffset) as f32,
                    0.0,
                    (200 * 8 + texoffset) as f32,
                    160.0,
                    true,
                    6,
                );
            }
            x1 = x2;
            y1 = y2;
        }
        const FLOOR_WIDTH: Float = 68.0;
        let floorcoords = [
            (120.0 - cx + FLOOR_WIDTH * cc - FLOOR_WIDTH * ss) as f32,
            (68.0 - cy - (FLOOR_WIDTH * cc + FLOOR_WIDTH * ss) * sph + 45.0 * cph) as f32,
            (120.0 - cx + FLOOR_WIDTH * cc + FLOOR_WIDTH * ss) as f32,
            (68.0 - cy - (-FLOOR_WIDTH * cc + FLOOR_WIDTH * ss) * sph + 45.0 * cph) as f32,
            (120.0 - cx - FLOOR_WIDTH * cc + FLOOR_WIDTH * ss) as f32,
            (68.0 - cy - (-FLOOR_WIDTH * cc - FLOOR_WIDTH * ss) * sph + 45.0 * cph) as f32,
            (120.0 - cx - FLOOR_WIDTH * cc - FLOOR_WIDTH * ss) as f32,
            (68.0 - cy - (FLOOR_WIDTH * cc - FLOOR_WIDTH * ss) * sph + 45.0 * cph) as f32,
        ];
        cart.textri(
            floorcoords[0],
            floorcoords[1],
            floorcoords[2],
            floorcoords[3],
            floorcoords[6],
            floorcoords[7],
            180.0 * 8.0,
            0.0,
            197.0 * 8.0,
            0.0,
            180.0 * 8.0,
            17.0 * 8.0,
            true,
            6,
        );
        cart.textri(
            floorcoords[4],
            floorcoords[5],
            floorcoords[2],
            floorcoords[3],
            floorcoords[6],
            floorcoords[7],
            197.0 * 8.0,
            17.0 * 8.0,
            197.0 * 8.0,
            0.0,
            180.0 * 8.0,
            17.0 * 8.0,
            true,
            6,
        );

        let mut points = Vec::new();
        for p in core.tree_model.iter() {
            let mut x1 = p.x;
            let mut y1 = p.y;
            let mut z1 = p.z;
            if p.sp < 350 {
                x1 += (x1 / 30.0 + self.t as Float / 20.0).cos() * (y1 + 40.0) / 20.0
                    + easeout(
                        Self::BLOW_AWAY_TIME - 10.0 * x1 - self.t as Float,
                        Some(100.0),
                    );
                z1 += (x1 / 40.0 + z1 / 30.0 + self.t as Float / 30.0).sin() * (y1 + 40.0) / 20.0;
            }
            let x2 = x1 * cc - z1 * ss;
            let z2 = x1 * ss + z1 * cc;
            let y2 = y1;
            y1 = y2 * cph + z2 * sph;
            z1 = -y2 * sph + z2 * cph;
            x1 = x2;
            points.push(TreeUnit {
                x: x1 - cx,
                y: y1 + cy,
                z: z1,
                sp: p.sp,
            });
        }
        points.sort_by(|a, b| b.z.partial_cmp(&a.z).unwrap_or(std::cmp::Ordering::Equal));
        for p in points.iter() {
            cart.spr(p.sp, 112 + p.x as i32, 56 - p.y as i32, 0, 1, 0, 0, 2, 2);
        }
        self.t += 1;
    }
}

struct SceneTowerTop2 {
    core: Rc<RefCell<FallSpireCore>>,
    t: i32,
}
impl SceneTowerTop2 {
    const BLOW_AWAY_TIME: Float = -100.0;
    const SHAKE_TIME: i32 = 300;
    fn new(core: Rc<RefCell<FallSpireCore>>) -> Self {
        Self { core, t: 0 }
    }
}
impl FallSpireScene for SceneTowerTop2 {
    fn update(&mut self) {
        let core = &self.core.borrow().gcore;
        core.palette_index(&[]);
        core.cart.borrow_mut().cls(11);
        let interp_f = easeout((self.t - 100) as Float, Some(100.0)) * 0.5;
        let interp = interp_f as i32;
        core.palette_index(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 7, 7, 13, 14, 7]);
        core.cloud_repeat(self.t / 4 + 120, interp / 5);
        core.palette_index(&[]);
        core.cloud_repeat(self.t * 4 / 5, interp / 2);
        let shake = if self.t > Self::SHAKE_TIME {
            (self.t / 4) % 2
        } else {
            0
        };
        core.cart
            .borrow_mut()
            .map(98, 2, 4, 17, 120 - 16 + shake, 50 + interp, 6, 1);
        for p in core.tree_model.iter() {
            if p.sp < 350 {
                let blow = easeout(
                    Self::BLOW_AWAY_TIME - 10.0 * p.x - self.t as Float,
                    Some(100.0),
                );
                core.cart.borrow_mut().spr(
                    (p.sp - 320) / 2 + 306,
                    (p.x * 0.35 + 120.0 + blow * (1.0 + 0.5 * (self.t as Float / 60.0 + p.z).cos()))
                        as i32,
                    (-p.y * 0.35
                        + 62.0
                        + interp_f
                        + blow * (0.2 * (self.t as Float / 100.0 + 5.0 * p.z).sin()))
                        as i32,
                    0,
                    1,
                    0,
                    0,
                    1,
                    1,
                );
            }
        }
        self.t += 1;
    }
}

struct SceneTowerCollapse {
    core: Rc<RefCell<FallSpireCore>>,
    t: i32,
    plants: Vec<(Float, Float, Float)>,
}
impl SceneTowerCollapse {
    const COLOR_CHANGE_TIME: i32 = 300;
    fn new(core: Rc<RefCell<FallSpireCore>>) -> Self {
        Self {
            core,
            t: 0,
            plants: Vec::new(),
        }
    }
    fn plant(&mut self) {
        let rng = &mut self.core.borrow_mut().rng;
        let coord = (
            rng.gen_range(0..=240) as Float,
            rng.gen_range(104..=128) as Float,
            0.0,
        );
        self.plants.push(coord);
    }
}
impl FallSpireScene for SceneTowerCollapse {
    fn update(&mut self) {
        self.core.borrow().cart.borrow_mut().cls(11);
        self.core.borrow_mut().gcore.camera.x = 400.0;
        self.core.borrow_mut().gcore.camera.y = 0.0;
        {
            let core = &self.core.borrow().gcore;
            core.palette_index(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 7, 7, 13, 14, 7]);
            core.cart
                .borrow_mut()
                .map(21, 35, 26, 12, -100 + self.t / 20, 20, 6, 1);
            core.palette_index(&[1, 3, 4]);
            core.cart
                .borrow_mut()
                .map(0, 34, 20, 12, 120 - self.t / 10, 30, 6, 1);
            core.palette_index(&[1, 3, 4]);
            let angle = (-(self.t / 10) as Float / 200.0).max(-0.2);
            let towerx = (60 + (self.t / 4) % 2) as Float;
            let towery = 80.0 + ((self.t * self.t) as Float / 2000.0);
            let h = 40.0 * 8.0;
            let cc = -angle.cos();
            let ss = -angle.sin();
            let x1 = (towerx - ss * h) as f32;
            let y1 = (towery + cc * h) as f32;
            let x2 = (towerx + cc * 8.0 - ss * h) as f32;
            let y2 = (towery + ss * 8.0 + cc * h) as f32;
            let x3 = towerx as f32;
            let y3 = towery as f32;
            let x4 = (towerx + cc * 8.0) as f32;
            let y4 = (towery + ss * 8.0) as f32;
            core.cart.borrow_mut().textri(
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
                103.0 * 8.0,
                0.0,
                104.0 * 8.0,
                0.0,
                103.0 * 8.0,
                h as f32,
                true,
                6,
            );
            core.cart.borrow_mut().textri(
                x4,
                y4,
                x2,
                y2,
                x3,
                y3,
                104.0 * 8.0,
                h as f32,
                104.0 * 8.0,
                0.0,
                103.0 * 8.0,
                h as f32,
                true,
                6,
            );
            core.cart
                .borrow_mut()
                .map(21, 35, 26, 12, self.t / 8, 45, 6, 1);
            if self.t > Self::COLOR_CHANGE_TIME {
                core.palette_index(&[1, 2, 10, 10, 10]);
            } else {
                core.palette_index(&[1, 2, 4, 4, 4]);
            }
            core.terrain1(
                -core.camera.x as i32 / 4 + 100,
                80 - core.camera.y as i32 / 4,
                None,
            );
            if self.t > Self::COLOR_CHANGE_TIME + 20 {
                core.palette_index(&[1, 2, 9, 10, 10]);
            } else {
                core.palette_index(&[1, 2, 3, 4, 4]);
            }
            core.terrain1(
                -core.camera.x as i32 / 2,
                95 - core.camera.y as i32 / 2,
                None,
            );
        }

        if self.t > Self::COLOR_CHANGE_TIME + 60 {
            self.core.borrow().gcore.palette_index(&[]);
            if self.t % 20 == 0 && self.plants.len() < 500 {
                self.plant();
            }
        }
        for pl in self.plants.iter_mut() {
            self.core.borrow().cart.borrow_mut().line(
                pl.0 as f32,
                (pl.1 - pl.2) as f32,
                pl.0 as f32,
                pl.1 as f32,
                5,
            );
            pl.2 = (pl.2 + pl.1 * 0.0001).min(16.0);
        }
        {
            let core = &self.core.borrow().gcore;
            if self.t > Self::COLOR_CHANGE_TIME + 40 {
                core.palette_index(&[8, 9, 9, 10, 10]);
            } else {
                core.palette_index(&[]);
            }
            core.cart.borrow_mut().spr(
                257 + ((self.t / 15) % 3) * 3,
                600 - core.camera.x as i32,
                120 - 24 - core.camera.y as i32,
                6,
                1,
                0,
                0,
                3,
                3,
            );
            core.terrain2(-core.camera.x as i32, 120 - core.camera.y as i32, None);
            core.palette_index(&[]);
        }
        self.t += 1;
    }
}

enum FilterType {
    Lp,
    Hp,
    Bp,
}

struct Filter {
    tp: FilterType,
    g: Float,
    r: Float,
    h: Float,
    state: (Float, Float),
}
impl Filter {
    fn new(param: Option<(FilterType, Float, Float, Float)>) -> Self {
        let (tp, g, r, h) = param.unwrap_or((FilterType::Lp, 0.5, 0.5, 0.5));
        Self {
            tp,
            g,
            r,
            h,
            state: (0.0, 0.0),
        }
    }
    fn update(&mut self, fr: Float, res: Float) {
        let f = fr.min(0.49).max(0.0);
        let res = res.max(0.001);
        self.g = (f * std::f64::consts::PI as Float).tan();
        self.r = 1.0 / res;
        self.h = 1.0 / (1.0 + self.r * self.g + self.g * self.g);
    }
    fn process(&mut self, input: Float) -> Float {
        let hp = (input - self.r * self.state.0 - self.g * self.state.0 - self.state.1) * self.h;
        let bp = self.g * hp + self.state.0;
        self.state.0 = self.g * hp + bp;
        let lp = self.g * bp + self.state.1;
        self.state.1 = self.g * bp + lp;
        match self.tp {
            FilterType::Bp => bp * self.r,
            FilterType::Hp => hp,
            FilterType::Lp => lp,
        }
    }
    fn reset(&mut self) {
        self.state = (0.0, 0.0);
    }
}

struct Voice {
    index: usize,
    t: i32,
    wave: [Float; 32],
    volume: Float,
    freq: Float,
    algo: u8,
    decay: Float,
    vibd: Float,
    vibs: Float,
    filter: Filter,
}
impl Voice {
    fn new(index: usize) -> Self {
        Self {
            index,
            t: 0,
            wave: [8.0; 32],
            volume: 0.0,
            freq: 0.0,
            algo: 0,
            decay: 0.02,
            vibd: 0.0,
            vibs: 0.0,
            filter: Filter::new(None),
        }
    }
    fn init(&self, cart: &mut CartContext) {
        cart.sfx(0, 36 + self.index as u8 * 3, -1, self.index as u8, 15, 0);
    }
    fn play(&mut self, note: i32, algo: u8) {
        self.t = 0;
        self.volume = 1.0;
        self.freq = 440.0 * ((note - 10) as Float / 12.0).powi(2);
        self.algo = algo;
        self.vibd = 0.0;
        self.vibs = 0.4;
        self.decay = 0.02;
        match algo {
            0 => {
                self.decay = 0.05;
                self.vibd = 0.01;
            }
            1 => {
                self.decay = 0.02;
            }
            2 => {
                self.decay = 0.0;
                self.vibd = 0.007;
                self.vibs = 0.2;
            }
            3 => {
                self.decay = 0.03;
                self.vibd = 0.02;
                self.vibs = 0.8;
            }
            4 => {
                self.decay = 0.04;
                self.vibd = 0.01;
            }
            5 => {
                self.decay = 0.0;
            }
            6 => {
                self.decay = 0.005;
                self.vibd = 0.02;
                self.vibs = 0.8;
            }
            7 => {
                self.decay = 0.003;
            }
            _ => (),
        }
        self.wave.fill(0.0);
    }
}

struct Seq {
    numvc: usize,
    l: i32,
    t: Vec<[Option<i32>; 4]>,
    spd: Option<i32>,
}
impl Seq {
    fn new(numvc: usize, l: i32, t: Vec<[Option<i32>; 4]>, spd: Option<i32>) -> Self {
        Self { numvc, l, t, spd }
    }
}

pub struct FallSpire {
    core: Option<Rc<RefCell<FallSpireCore>>>,
    scenes: Vec<Box<dyn FallSpireScene>>,
}

impl FallSpire {
    pub fn new() -> Self {
        Self {
            core: None,
            scenes: Vec::new(),
        }
    }
}
impl InternalProgram for FallSpire {
    fn init(&mut self, cart: Rc<RefCell<CartContext>>, system: Rc<RefCell<SystemContext>>) {
        let data = crate::wheel_file::WheelFile::from_bytes(include_bytes!("fallspire.tic"))
            .expect("failed to load fallspire.tic");
        cart.borrow_mut().file_data = Rc::new(RefCell::new(data));
        cart.borrow_mut().load_all();
        let core = Rc::new(RefCell::new(FallSpireCore::new(cart, system)));
        self.scenes = vec![
            Box::new(ScenePond::new(core.clone())),
            Box::new(SceneForest::new(core.clone())),
            Box::new(SceneFlight::new(core.clone())),
            Box::new(SceneTowerBase::new(core.clone())),
            Box::new(SceneTower::new(core.clone())),
            Box::new(SceneTowerTop::new(core.clone())),
            Box::new(SceneTree::new(core.clone())),
            Box::new(SceneTowerTop2::new(core.clone())),
            Box::new(SceneTowerCollapse::new(core.clone())),
        ];
        self.core = Some(core);
    }
    fn update(&mut self) {
        {
            let core = &mut self.core.as_ref().unwrap().borrow_mut();
            if core.cart.borrow().keyp(Some(66)) {
                core.system.borrow_mut().exit();
            }
            core.update_music();
            if core.acore.context.fin {
                core.fade_amt -= 0.5;
                if core.fade_amt < -255.0 {
                    core.cart.borrow_mut().cls(0);
                    core.system.borrow_mut().trace("           ~fallspire~", 15);
                    core.system
                        .borrow_mut()
                        .trace("       by petet and sintel", 15);
                    core.system
                        .borrow_mut()
                        .trace("    thank you for watching <3", 15);
                    core.system.borrow_mut().exit();
                }
            } else {
                core.fade_amt = (core.fade_amt - 2.0).max(0.0);
            }
            core.gcore.fade(core.fade_amt as i32);
        }
        let n = self.core.as_ref().unwrap().borrow().scene_number;
        self.scenes[n].update();
        //self.core.as_ref().unwrap().borrow().cart.borrow_mut().map(60, 34, 30, 17, 0, 0, 6, 1);
    }
    fn scanline(&mut self, i: usize) {
        let n = self.core.as_ref().unwrap().borrow().scene_number;
        self.scenes[n].scanline(i as i32);
    }
    fn overlay(&mut self) {
        let n = self.core.as_ref().unwrap().borrow().scene_number;
        self.scenes[n].overlay();
    }
}
