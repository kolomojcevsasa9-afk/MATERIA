use crate::io;

fn sqrt_f32(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    let mut guess = x;
    for _ in 0..8 {
        guess = (guess + x / guess) * 0.5;
    }
    guess
}

fn abs_f32(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

fn max_f32(a: f32, b: f32) -> f32 {
    if a > b { a } else { b }
}

fn min_f32(a: f32, b: f32) -> f32 {
    if a < b { a } else { b }
}

fn cos_f32(mut x: f32) -> f32 {
    const PI: f32 = 3.14159265;
    const TWO_PI: f32 = 6.28318530;
    
    while x > PI { x -= TWO_PI; }
    while x < -PI { x += TWO_PI; }

    let x2 = x * x;
    let x4 = x2 * x2;
    let x6 = x4 * x2;

    1.0 - (x2 / 2.0) + (x4 / 24.0) - (x6 / 720.0)
}

const WIDTH: usize = 320;
const HEIGHT: usize = 200;

static mut BACKGROUND_BUFFER: [u8; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

// Компактный курсор в стиле Windows (9x12 пикселей)
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

// Преобразование RGB (0.0..1.0) в индекс 6x6x6 RGB палитры VGA
fn rgb_to_vga(r: f32, g: f32, b: f32) -> u8 {
    let r_idx = ((r.clamp(0.0, 1.0) * 5.0) as usize).min(5);
    let g_idx = ((g.clamp(0.0, 1.0) * 5.0) as usize).min(5);
    let b_idx = ((b.clamp(0.0, 1.0) * 5.0) as usize).min(5);
    (1 + r_idx * 36 + g_idx * 6 + b_idx) as u8
}

fn init_vga_palette() {
    // Устанавливаем 6x6x6 RGB куб в палитре VGA (индексы 1..216)
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

    // Индекс 0 = Черный контур
    io::outb(0x3C8, 0);
    io::outb(0x3C9, 0); io::outb(0x3C9, 0); io::outb(0x3C9, 0);

    // Индекс 255 = Белое тело курсора
    io::outb(0x3C8, 255);
    io::outb(0x3C9, 63); io::outb(0x3C9, 63); io::outb(0x3C9, 63);
}

pub fn init() {
    init_vga_palette();

    let cx = 160.0f32; // Квадрат строго по центру экрана
    let cy = 100.0f32;
    let box_size = 55.0f32;

    // Генерируем красивый ShaderToy фон с формулой: vec3 col = 0.5 + 0.5*cos(uv.xyx + vec3(0,2,4))
    unsafe {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let px = x as f32 - cx;
                let py = y as f32 - cy;

                let qx = abs_f32(px) - box_size;
                let qy = abs_f32(py) - box_size;

                let outside_x = max_f32(qx, 0.0);
                let outside_y = max_f32(qy, 0.0);
                let outside_dist = sqrt_f32(outside_x * outside_x + outside_y * outside_y);
                let inside_dist = min_f32(max_f32(qx, qy), 0.0);
                let dist = outside_dist + inside_dist;

                let color_index = if dist <= 1.5 && dist >= -1.5 {
                    255 // Белый контур
                } else if dist < 0.0 {
                    // Внутри SDF квадрата: волновая палитра Inigo Quilez
                    let t = dist * 0.08;
                    let r = 0.5 + 0.5 * cos_f32(t + 0.0);
                    let g = 0.5 + 0.5 * cos_f32(t + 2.0);
                    let b = 0.5 + 0.5 * cos_f32(t + 4.0);
                    rgb_to_vga(r, g, b)
                } else {
                    // Вне квадрата: формульный ShaderToy градиент uv.xyx + vec3(0, 2, 4)
                    let uv_x = x as f32 / WIDTH as f32;
                    let uv_y = y as f32 / HEIGHT as f32;

                    let r = 0.5 + 0.5 * cos_f32(uv_x * 6.28 + 0.0);
                    let g = 0.5 + 0.5 * cos_f32(uv_y * 6.28 + 2.0);
                    let b = 0.5 + 0.5 * cos_f32((uv_x + uv_y) * 6.28 + 4.0);
                    rgb_to_vga(r, g, b)
                };

                BACKGROUND_BUFFER[y * WIDTH + x] = color_index;
            }
        }

        // Быстрое 64-битное SIMD копирование фона в VRAM
        let vram = 0xA0000 as *mut u64;
        let bg_ptr = core::ptr::addr_of!(BACKGROUND_BUFFER) as *const u64;
        for i in 0..(WIDTH * HEIGHT / 8) {
            *vram.add(i) = *bg_ptr.add(i);
        }
    }
}

        // Draw Taskbar
        for y in (HEIGHT - 25)..HEIGHT {
            for x in 0..WIDTH {
                BACKGROUND_BUFFER[y * WIDTH + x] = 16; // Greyish taskbar
            }
        }
        
        let vram = 0xA0000 as *mut u64;
        let bg_ptr = core::ptr::addr_of!(BACKGROUND_BUFFER) as *const u64;
        for i in 0..(WIDTH * HEIGHT / 8) {
            *vram.add(i) = *bg_ptr.add(i);
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
            }
        }
    }
}










