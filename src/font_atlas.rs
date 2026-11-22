pub const RAW_ATLAS_DATA: &str = include_str!("../output-onlinepngtools.bin");

pub fn get_atlas_pixel(index: usize) -> [u8; 3] {
    // В файле данные в формате "R, G, B, ..."
    // Парсим строку для получения RGB значений
    let mut r = 0;
    let mut g = 0;
    let mut b = 0;
    
    let mut current_part = 0;
    let mut val = 0;
    let mut started = false;
    
    let mut parts_passed = 0;
    let target_part = index * 3;
    
    for byte in RAW_ATLAS_DATA.bytes() {
        if byte >= b'0' && byte <= b'9' {
            val = val * 10 + (byte - b'0');
            started = true;
        } else if started {
            if parts_passed == target_part { r = val as u8; }
            else if parts_passed == target_part + 1 { g = val as u8; }
            else if parts_passed == target_part + 2 { b = val as u8; return [r, g, b]; }
            
            parts_passed += 1;
            val = 0;
            started = false;
        }
    }
    [r, g, b]
}
