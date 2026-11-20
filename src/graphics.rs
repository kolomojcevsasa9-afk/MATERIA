pub fn draw_cursor(x: i32, y: i32) {
    // Рисуем курсор в VGA буфере (символом или изменением цвета фона)
    // Упрощенно: пишем '@' в позицию (x, y)
    let buffer = 0xb8000 as *mut u16;
    unsafe {
        *buffer.add((y * 80 + x) as usize) = 0x0F00 | b'@' as u16;
    }
}
