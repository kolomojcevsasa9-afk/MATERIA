use crate::io;

fn sqrt_f32(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    let mut guess = x;
    for _ in 0..8 { guess = (guess + x / guess) * 0.5; }
    guess
}

fn abs_f32(x: f32) -> f32 { if x < 0.0 { -x } else { x } }
fn max_f32(a: f32, b: f32) -> f32 { if a > b { a } else { b } }
fn min_f32(a: f32, b: f32) -> f32 { if a < b { a } else { b } }

// Функция для плавного сглаживания (Anti-aliasing) для скругленных углов
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

const WIDTH: usize = 320;
const HEIGHT: usize = 200;

static mut WALLPAPER_BUFFER: [u8; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];
static mut FRAME_BUFFER: [u8; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

const MOUSE_CURSOR_WIDTH: usize = 9;
const MOUSE_CURSOR_HEIGHT: usize = 12;
const MOUSE_CURSOR: [[u8; 9]; 12] = [
    [1, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 1, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 1, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 1, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 1, 0, 0, 0],
    [1, 2, 2, 2, 2, 2, 1, 0, 0],
    [1, 2, 2, 1, 1, 1, 1, 1, 0],
    [1, 2, 1, 1, 2, 1, 0, 0, 0],
    [1, 1, 0, 0, 1, 2, 1, 0, 0],
    [1, 0, 0, 0, 0, 1, 2, 1, 0],
    [0, 0, 0, 0, 0, 0, 1, 1, 0],
];

fn rgb_to_vga(r: f32, g: f32, b: f32) -> u8 {
    let r_idx = ((r.clamp(0.0, 1.0) * 5.0) as usize).min(5);
    let g_idx = ((g.clamp(0.0, 1.0) * 5.0) as usize).min(5);
    let b_idx = ((b.clamp(0.0, 1.0) * 5.0) as usize).min(5);
    (1 + r_idx * 36 + g_idx * 6 + b_idx) as u8
}

fn init_vga_palette() {
    for r in 0..6 {
        for g in 0..6 {
            for b in 0..6 {
                let idx = 1 + r * 36 + g * 6 + b;
                io::outb(0x3C8, idx as u8);
                io::outb(0x3C9, (r * 12) as u8);
                io::outb(0x3C9, (g * 12) as u8);
                io::outb(0x3C9, (b * 12) as u8);
            }
        }
    }
    io::outb(0x3C8, 0); io::outb(0x3C9, 0); io::outb(0x3C9, 0); io::outb(0x3C9, 0);
    io::outb(0x3C8, 255); io::outb(0x3C9, 63); io::outb(0x3C9, 63); io::outb(0x3C9, 63);
}

// Улучшенный SDF для скругленного прямоугольника
fn sd_rounded_box_aa(px: f32, py: f32, w: f32, h: f32, r: f32) -> f32 {
    let qx = abs_f32(px) - w + r;
    let qy = abs_f32(py) - h + r;
    sqrt_f32(max_f32(qx, 0.0) * max_f32(qx, 0.0) + max_f32(qy, 0.0) * max_f32(qy, 0.0)) + min_f32(max_f32(qx, qy), 0.0) - r
}

pub fn init() {
    init_vga_palette();
    let cx = 160.0f32; 
    let cy = 100.0f32;
    let box_size = 60.0f32;
    let corner_r = 10.0f32;

    unsafe {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let px = x as f32 - cx;
                let py = y as f32 - cy;

                let qx = abs_f32(px) - box_size + corner_r;
                let qy = abs_f32(py) - box_size + corner_r;
                let dist = sqrt_f32(max_f32(qx, 0.0) * max_f32(qx, 0.0) + max_f32(qy, 0.0) * max_f32(qy, 0.0)) + min_f32(max_f32(qx, qy), 0.0) - corner_r;

                let color = if dist <= 0.0 {
                    15 
                } else if dist < 2.0 {
                    20
                } else {
                    10
                };

                WALLPAPER_BUFFER[y * WIDTH + x] = color;
            }
        }
        
        let vram = 0xA0000 as *mut u64;
        let bg_ptr = core::ptr::addr_of!(WALLPAPER_BUFFER) as *const u64;
        for i in 0..(WIDTH * HEIGHT / 8) {
            *vram.add(i) = *bg_ptr.add(i);
        }
    }
}

pub fn render_frame_with_terminal(x: i32, y: i32, w: i32, h: i32, fullscreen: bool, mx: i32, my: i32, term: &crate::terminal::Terminal) {
    unsafe {
        // Clear frame buffer with background color
        for i in 0..(WIDTH * HEIGHT) {
            FRAME_BUFFER[i] = WALLPAPER_BUFFER[i];
        }

        // Draw window
        for wy in 0..h {
            for wx in 0..w {
                let px = (x + wx) as usize;
                let py = (y + wy) as usize;
                if px < WIDTH && py < HEIGHT {
                    let color = if (wx == 0 || wx == w - 1 || wy == 0 || wy == h - 1) { 15 } else { 0 };
                    FRAME_BUFFER[py * WIDTH + px] = color;
                }
            }
        }

        // Draw terminal text
        for ty in 0..crate::terminal::ROWS {
            for tx in 0..crate::terminal::COLS {
                let ch = term.buffer[ty][tx];
                let px = (x + 4 + tx as i32 * 6) as usize;
                let py = (y + 16 + ty as i32 * 8) as usize;
                if px + 5 < WIDTH && py + 7 < HEIGHT {
                    crate::graphics::draw_char_bitmap(&mut FRAME_BUFFER, px, py, ch, 15);
                }
            }
        }

        // Draw mouse cursor
        for my_pos in 0..MOUSE_CURSOR_HEIGHT {
            for mx_pos in 0..MOUSE_CURSOR_WIDTH {
                let px = (mx + mx_pos as i32) as usize;
                let py = (my + my_pos as i32) as usize;
                if px < WIDTH && py < HEIGHT {
                    match MOUSE_CURSOR[my_pos][mx_pos] {
                        1 => FRAME_BUFFER[py * WIDTH + px] = 15,
                        2 => FRAME_BUFFER[py * WIDTH + px] = 0,
                        _ => {}
                    }
                }
            }
        }

        // Copy to VRAM
        let vram = 0xA0000 as *mut u64;
        let fb_ptr = core::ptr::addr_of!(FRAME_BUFFER) as *const u64;
        for i in 0..(WIDTH * HEIGHT / 8) {
            *vram.add(i) = *fb_ptr.add(i);
        }
    }
}

