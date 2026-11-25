// В no_std окружении f32 методы sqrt/powf недоступны напрямую, 
// используем libm функции.
use libm::{sqrtf, powf};

// Продвинутые SDF примитивы
pub fn sd_circle(p: [f32; 2], r: f32) -> f32 {
    sqrtf(p[0] * p[0] + p[1] * p[1]) - r
}

pub fn sd_box(p: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = p[0].abs() - b[0];
    let dy = p[1].abs() - b[1];
    let e_x = dx.max(0.0);
    let e_y = dy.max(0.0);
    let inside = dx.max(dy).min(0.0);
    sqrtf(e_x * e_x + e_y * e_y) + inside
}

pub fn sd_rounded_box(p: [f32; 2], b: [f32; 2], r: [f32; 4]) -> f32 {
    let k = [
        if p[0] > 0.0 { r[0] } else { r[1] },
        if p[0] > 0.0 { r[2] } else { r[3] },
    ];
    let d = [p[0].abs() - b[0] + k[0], p[1].abs() - b[1] + k[1]];
    let d_x_max = d[0].max(0.0);
    let d_y_max = d[1].max(0.0);
    sqrtf(d_x_max * d_x_max + d_y_max * d_y_max) + d[0].max(d[1]).min(0.0) - k[0].min(k[1])
}

// Операции над SDF (композиция)
pub fn op_union(d1: f32, d2: f32) -> f32 {
    d1.min(d2)
}

pub fn op_subtraction(d1: f32, d2: f32) -> f32 {
    (-d2).max(d1)
}

pub fn op_intersection(d1: f32, d2: f32) -> f32 {
    d1.max(d2)
}

// Плавное слияние (Smooth Union - для "мягкого" соединения объектов)
pub fn op_smooth_union(d1: f32, d2: f32, k: f32) -> f32 {
    let h = (0.5 + 0.5 * (d2 - d1) / k).clamp(0.0, 1.0);
    lerp(d2, d1, h) - k * h * (1.0 - h)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}
