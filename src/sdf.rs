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
<<<<<<< HEAD
    unsafe {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                WALLPAPER_BUFFER[y * WIDTH + x] = 0; 
            }
        }
        for y in (HEIGHT - 20)..HEIGHT {
            for x in 0..WIDTH {
                WALLPAPER_BUFFER[y * WIDTH + x] = rgb_to_vga(0.05, 0.05, 0.1);
=======

    let cx = 160.0f32; 
    let cy = 100.0f32;
    let box_size = 60.0f32;
    let corner_r = 10.0f32;

    unsafe {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let px = x as f32 - cx;
                let py = y as f32 - cy;

                // SDF для скругленного прямоугольника
                let qx = abs_f32(px) - box_size + corner_r;
                let qy = abs_f32(py) - box_size + corner_r;
                let dist = sqrt_f32(max_f32(qx, 0.0) * max_f32(qx, 0.0) + max_f32(qy, 0.0) * max_f32(qy, 0.0)) + min_f32(max_f32(qx, qy), 0.0) - corner_r;

                let color = if dist <= 0.0 {
                    // Внутри: темно-серый
                    15 
                } else if dist < 2.0 {
                    // Рамка: светло-серый
                    20
                } else {
                    // Фон: средний серый
                    10
                };

                BACKGROUND_BUFFER[y * WIDTH + x] = color;
            }
        }
        
        let vram = 0xA0000 as *mut u64;
        let bg_ptr = core::ptr::addr_of!(BACKGROUND_BUFFER) as *const u64;
        for i in 0..(WIDTH * HEIGHT / 8) {
            *vram.add(i) = *bg_ptr.add(i);
        }
    }
}
pub fn render_cursor(old_x: i32, old_y: i32, new_x: i32, new_y: i32) {
    let vram = 0xA0000 as *mut u8;

    unsafe {
        // 1. Восстанавливаем фоновый ShaderToy рисунок под СТАРЫМ курсором
        for cy in 0..MOUSE_CURSOR_HEIGHT {
            let py = old_y as usize + cy;
            if py >= HEIGHT { break; }

            for cx in 0..MOUSE_CURSOR_WIDTH {
                let px = old_x as usize + cx;
                if px >= WIDTH { break; }

                if MOUSE_CURSOR[cy][cx] != 0 {
                    let idx = py * WIDTH + px;
                    *vram.add(idx) = BACKGROUND_BUFFER[idx];
                }
            }
        }

        // 2. Рисуем курсор Windows под НОВЫМИ координатами
        for cy in 0..MOUSE_CURSOR_HEIGHT {
            let py = new_y as usize + cy;
            if py >= HEIGHT { break; }

            for cx in 0..MOUSE_CURSOR_WIDTH {
                let px = new_x as usize + cx;
                if px >= WIDTH { break; }

                let pixel_type = MOUSE_CURSOR[cy][cx];
                if pixel_type == 1 {
                    *vram.add(py * WIDTH + px) = 0;   // Черная каемка
                } else if pixel_type == 2 {
                    *vram.add(py * WIDTH + px) = 255; // Белая стрелочка
                }
>>>>>>> 994f2afb (SDF interface 0.14)
            }
        }
    }
}

pub fn render_frame(win_x: i32, win_y: i32, win_w: i32, win_h: i32, is_fullscreen: bool, mouse_x: i32, mouse_y: i32) {
    unsafe {
        let src_ptr = core::ptr::addr_of!(WALLPAPER_BUFFER) as *const u64;
        let dst_ptr = core::ptr::addr_of_mut!(FRAME_BUFFER) as *mut u64;
        for i in 0..(WIDTH * HEIGHT / 8) {
            *dst_ptr.add(i) = *src_ptr.add(i);
        }

        let fb_ptr = core::ptr::addr_of_mut!(FRAME_BUFFER) as *mut u8;

        let half_w = win_w as f32 / 2.0;
        let half_h = win_h as f32 / 2.0;
        let win_cx = win_x as f32 + half_w;
        let win_cy = win_y as f32 + half_h;
        let corner_r = if is_fullscreen { 0.0 } else { 12.0 }; // Увеличенный радиус для красоты

        for wy in 0..win_h {
            let py = win_y + wy;
            if py < 0 || py >= HEIGHT as i32 { continue; }
            for wx in 0..win_w {
                let px = win_x + wx;
                if px < 0 || px >= WIDTH as i32 { continue; }
                
                let d = sd_rounded_box_aa(px as f32 - win_cx, py as f32 - win_cy, half_w, half_h, corner_r);
                
                // Anti-aliasing с помощью smoothstep
                let alpha = 1.0 - smoothstep(0.0, 1.5, d);
                
                let idx = py as usize * WIDTH + px as usize;
                if d < 0.0 {
                    let col = if wy < 14 { rgb_to_vga(0.2, 0.3, 0.5) } else { rgb_to_vga(0.1, 0.1, 0.2) };
                    *fb_ptr.add(idx) = col;
                } else if alpha > 0.0 {
                    // Смешивание для плавности
                    *fb_ptr.add(idx) = 255;
                }
            }
        }

        for cy in 0..MOUSE_CURSOR_HEIGHT {
            let py = mouse_y + cy as i32;
            if py < 0 || py >= HEIGHT as i32 { continue; }
            for cx in 0..MOUSE_CURSOR_WIDTH {
                let px = mouse_x + cx as i32;
                if px < 0 || px >= WIDTH as i32 { continue; }
                let ptype = MOUSE_CURSOR[cy][cx];
                let idx = py as usize * WIDTH + px as usize;
                if ptype == 1 { *fb_ptr.add(idx) = 0; }
                else if ptype == 2 { *fb_ptr.add(idx) = 255; }
            }
        }

        let vram = 0xA0000 as *mut u64;
        let frame_ptr = core::ptr::addr_of!(FRAME_BUFFER) as *const u64;
        for i in 0..(WIDTH * HEIGHT / 8) {
            *vram.add(i) = *frame_ptr.add(i);
        }
    }
}

pub fn render_frame_with_terminal(win_x: i32, win_y: i32, win_w: i32, win_h: i32, is_fullscreen: bool, mouse_x: i32, mouse_y: i32, terminal: &crate::terminal::Terminal) {
    render_frame(win_x, win_y, win_w, win_h, is_fullscreen, mouse_x, mouse_y);
    render_terminal_contents(win_x, win_y, win_w, win_h, terminal);
    
    unsafe {
        let vram = 0xA0000 as *mut u64;
        let frame_ptr = core::ptr::addr_of!(FRAME_BUFFER) as *const u64;
        for i in 0..(WIDTH * HEIGHT / 8) {
            *vram.add(i) = *frame_ptr.add(i);
        }
    }
}

pub fn render_terminal_contents(win_x: i32, win_y: i32, win_w: i32, win_h: i32, terminal: &crate::terminal::Terminal) {
    let fb_ptr = core::ptr::addr_of_mut!(crate::sdf::FRAME_BUFFER) as *mut u8;
    let fb_slice = unsafe { core::slice::from_raw_parts_mut(fb_ptr, WIDTH * HEIGHT) };
    
    // Отрисовка символов в окне
    for ty in 0..crate::terminal::ROWS {
        for tx in 0..crate::terminal::COLS {
            let ch = terminal.buffer[ty][tx];
            let px = win_x + 5 + (tx * 6) as i32;
            let py = win_y + 16 + (ty * 10) as i32;
            if px < win_x + win_w && py < win_y + win_h && px >= 0 && py >= 0 {
                crate::graphics::draw_char_bitmap(fb_slice, px as usize, py as usize, ch, 255);
            }
        }
    }
}

