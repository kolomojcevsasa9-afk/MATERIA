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
// Обновленный курсор: 1 = Black inner, 2 = White outline
const MOUSE_CURSOR: [[u8; 9]; 12] = [
    [1, 2, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 2, 0, 0, 0, 0, 0, 0],
    [1, 2, 1, 2, 0, 0, 0, 0, 0],
    [1, 2, 2, 1, 2, 0, 0, 0, 0],
    [1, 2, 2, 2, 1, 2, 0, 0, 0],
    [1, 2, 2, 2, 2, 1, 2, 0, 0],
    [1, 2, 2, 2, 2, 2, 1, 2, 0],
    [1, 2, 2, 1, 1, 1, 1, 1, 2],
    [1, 2, 1, 1, 2, 1, 0, 0, 0],
    [1, 1, 0, 0, 1, 2, 1, 0, 0],
    [1, 0, 0, 0, 0, 1, 2, 1, 0],
    [0, 0, 0, 0, 0, 0, 1, 1, 0],
];
// 1 = Black inner, 2 = White outline

fn rgb_to_vga(r: f32, g: f32, b: f32) -> u8 {
    let r_idx = ((r.clamp(0.0, 1.0) * 5.0) as usize).min(5);
    let g_idx = ((g.clamp(0.0, 1.0) * 5.0) as usize).min(5);
    let b_idx = ((b.clamp(0.0, 1.0) * 5.0) as usize).min(5);
    (1 + r_idx * 36 + g_idx * 6 + b_idx) as u8
}

// Инициализация палитры с пользовательскими цветами
fn init_vga_palette() {
    let colors = [
        (0, 0, 0),       // 0 - Черный
        (63, 63, 63),    // 1 - Белый
        (63, 0, 0),      // 2 - Красный
        (0, 63, 0),      // 3 - Зеленый
        (0, 0, 63),      // 4 - Синий
        (63, 63, 0),     // 5 - Желтый
        (0, 63, 63),     // 6 - Голубой
        (63, 0, 63),     // 7 - Пурпурный
    ];

    for (i, &(r, g, b)) in colors.iter().enumerate() {
        io::outb(0x3C8, i as u8);
        io::outb(0x3C9, r);
        io::outb(0x3C9, g);
        io::outb(0x3C9, b);
    }
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


static mut CURSOR_SAVE_BUFFER: [u8; 108] = [0; 108]; // 9 * 12
static mut PREV_MX: i32 = -1;
static mut PREV_MY: i32 = -1;


// Оптимизированное разбиение на задачи для многопоточности
// Мы будем использовать константу для количества "полос" (задач)
const NUM_TASKS: usize = 4; // Можно увеличить в зависимости от ядер CPU

pub fn render_frame_with_terminal(x: i32, y: i32, w: i32, h: i32, _fullscreen: bool, mx: i32, my: i32, term: &crate::terminal::Terminal) {

// Глобальный флаг "грязности" экрана
static mut DIRTY: bool = true; 
static mut PREV_X: i32 = -1;
static mut PREV_Y: i32 = -1;

pub fn render_frame_with_terminal(x: i32, y: i32, w: i32, h: i32, _fullscreen: bool, mx: i32, my: i32, term: &crate::terminal::Terminal) {
    unsafe {
        // Проверка: изменилось ли что-то с прошлого кадра (окно переместилось или мышь)
        if PREV_X != x || PREV_Y != y {
            DIRTY = true;
            PREV_X = x;
            PREV_Y = y;
        }

        if DIRTY {
            let task_height = HEIGHT / NUM_TASKS;
            for task_id in 0..NUM_TASKS {
                let y_start = task_id * task_height;
                let y_end = if task_id == NUM_TASKS - 1 { HEIGHT } else { (task_id + 1) * task_height };
                render_subframe(y_start, y_end, x, y, w, h, mx, my, term);
            }
            DIRTY = false; // Очищаем флаг после полной перерисовки
        }

        // Курсор всегда требует обновления, так как он перемещается
        update_cursor_overlay(mx, my);

        // Copy to VRAM
        let vram = 0xA0000 as *mut u64;
        let fb_ptr = FRAME_BUFFER.as_ptr() as *const u64;
        core::ptr::copy_nonoverlapping(fb_ptr, vram, (WIDTH * HEIGHT) / 8);
    }
}

    let task_height = HEIGHT / NUM_TASKS;

    // В будущем здесь будет запуск параллельных задач на ядрах CPU
    // Сейчас мы эмулируем это последовательным выполнением полос
    for task_id in 0..NUM_TASKS {
        let y_start = task_id * task_height;
        let y_end = if task_id == NUM_TASKS - 1 { HEIGHT } else { (task_id + 1) * task_height };
        
        render_subframe(y_start, y_end, x, y, w, h, mx, my, term);
    }

    unsafe {
        // "Аппаратное" восстановление и сохранение курсора (должно быть атомарным)
        update_cursor_overlay(mx, my);

        // Copy to VRAM (Оптимизировано: копируем по 8 байт за итерацию)
        let vram = 0xA0000 as *mut u64;
        let fb_ptr = FRAME_BUFFER.as_ptr() as *const u64;
        core::ptr::copy_nonoverlapping(fb_ptr, vram, (WIDTH * HEIGHT) / 8);
    }
}

fn render_subframe(y_start: usize, y_end: usize, x: i32, y: i32, w: i32, h: i32, _mx: i32, _my: i32, term: &crate::terminal::Terminal) {
    unsafe {
        // 1. Отрисовка фона и окна для полосы
        for py in y_start..y_end {
            for px in 0..WIDTH {
                // Фон - теперь более нейтральный (например, темно-серый или черный)
                FRAME_BUFFER[py * WIDTH + px] = 8; // Темно-серый
                
                // Окно (рамка + фон окна)
                if px >= x as usize && px < (x + w) as usize && py >= y as usize && py < (y + h) as usize {
                    let wx = px - x as usize;
                    let wy = py - y as usize;
                    
                    // Фон окна черный
                    if wy >= 16 { // Оставляем 16 пикселей под заголовок
                        FRAME_BUFFER[py * WIDTH + px] = 0; // Черный фон окна
                    } else {
                        FRAME_BUFFER[py * WIDTH + px] = 7; // Серый заголовок
                    }

                    // Рамка окна
                    if wx == 0 || wx == w as usize - 1 || wy == 0 || wy == h as usize - 1 {
                        FRAME_BUFFER[py * WIDTH + px] = 15; // Белая рамка
                    }
                }
            }
        }

        // 2. Отрисовка текста (если текст попадает в эту полосу)
        // Упрощенная логика: каждая строка текста - 8 пикселей в высоту
        for ty in 0..crate::terminal::ROWS {
            let text_y_start = 16 + ty * 8;
            let text_y_end = text_y_start + 8;

            if text_y_end > y_start && text_y_start < y_end {
                for tx in 0..crate::terminal::COLS {
                    let ch = term.buffer[ty][tx];
                    let px = (x + 4 + tx as i32 * 6) as usize;
                    let py = text_y_start;
                    if px + 5 < WIDTH {
                        crate::graphics::draw_char_bitmap(&mut FRAME_BUFFER, px, py, ch, 15);
                    }
                }
            }
        }
    }
}

fn update_cursor_overlay(mx: i32, my: i32) {
    unsafe {
        // Восстановление фона после старого шлейфа и курсора
        if PREV_MX != -1 {
            let pmx = PREV_MX.clamp(0, WIDTH as i32 - MOUSE_CURSOR_WIDTH as i32);
            let pmy = PREV_MY.clamp(0, HEIGHT as i32 - MOUSE_CURSOR_HEIGHT as i32);
            for my_pos in 0..MOUSE_CURSOR_HEIGHT {
                let row_offset = (pmy + my_pos as i32) as usize * WIDTH;
                for mx_pos in 0..MOUSE_CURSOR_WIDTH {
                    FRAME_BUFFER[row_offset + (pmx + mx_pos as i32) as usize] = CURSOR_SAVE_BUFFER[my_pos * MOUSE_CURSOR_WIDTH + mx_pos];
                }
            }
        }

        // Вычисление предиктивной позиции
        let dx = mx - PREV_MX.max(0);
        let dy = my - PREV_MY.max(0);
        let pred_x = mx + (dx * 2);
        let pred_y = my + (dy * 2);
        
        let cx = mx.clamp(0, WIDTH as i32 - MOUSE_CURSOR_WIDTH as i32);
        let cy = my.clamp(0, HEIGHT as i32 - MOUSE_CURSOR_HEIGHT as i32);
        let px_pred = pred_x.clamp(0, WIDTH as i32 - MOUSE_CURSOR_WIDTH as i32);
        let py_pred = pred_y.clamp(0, HEIGHT as i32 - MOUSE_CURSOR_HEIGHT as i32);

        // Сохранение фона перед отрисовкой
        for my_pos in 0..MOUSE_CURSOR_HEIGHT {
            let row_offset = (cy + my_pos as i32) as usize * WIDTH;
            for mx_pos in 0..MOUSE_CURSOR_WIDTH {
                CURSOR_SAVE_BUFFER[my_pos * MOUSE_CURSOR_WIDTH + mx_pos] = FRAME_BUFFER[row_offset + (cx + mx_pos as i32) as usize];
            }
        }

        // УБРАН ШЛЕЙФ: Убрали цикл отрисовки предсказания (предикт)

        // Рисуем основной курсор
        for my_pos in 0..MOUSE_CURSOR_HEIGHT {
            let row_offset = (cy + my_pos as i32) as usize * WIDTH;
            for mx_pos in 0..MOUSE_CURSOR_WIDTH {
                let px = (cx + mx_pos as i32) as usize;
                match MOUSE_CURSOR[my_pos][mx_pos] {
                    1 => FRAME_BUFFER[row_offset + px] = 0,  // Black inner
                    2 => FRAME_BUFFER[row_offset + px] = 15, // White outline
                    _ => {}
                }
            }
        }
        PREV_MX = mx;
        PREV_MY = my;
    }
}

