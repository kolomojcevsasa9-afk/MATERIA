
pub fn read_config_u32(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    use crate::io;
    let address = ((bus as u32) << 16) | ((slot as u32) << 11) | ((func as u32) << 8) | ((offset as u32) & 0xfc) | 0x80000000;
    io::outl(0xCF8, address);
    io::inl(0xCFC)
}

pub fn check_device(bus: u8, slot: u8) -> Option<(u16, u16)> {
    let vendor_id = (read_config_u32(bus, slot, 0, 0) & 0xFFFF) as u16;
    if vendor_id == 0xFFFF {
        return None;
    }
    let device_id = ((read_config_u32(bus, slot, 0, 0) >> 16) & 0xFFFF) as u16;
    Some((vendor_id, device_id))
}

pub fn scan_pci() {
    use crate::shell; // Предполагая, что у вас есть какой-то вывод текста, если нет — используйте println! или аналог
    for bus in 0..8 {
        for slot in 0..32 {
            if let Some((vendor, device)) = check_device(bus, slot) {
                // Если нет прямого вывода, можно использовать что-то вроде sfs или просто отладочный вывод, если он есть.
                // Для простоты, выведем в наш "терминал" (предполагаем, что вывод есть)
                // Если `println!` нет, создадим лог-функцию.
            }
        }
    }
}

