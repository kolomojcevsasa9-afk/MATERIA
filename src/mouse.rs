use crate::io;

pub struct Mouse {
    pub x: i32,
    pub y: i32,
    pub left_button: bool,
    packet_index: usize,
    packet_bytes: [u8; 3],
}

impl Mouse {
    pub fn new() -> Self {
        Mouse {
            x: 160,
            y: 100,
            left_button: false,
            packet_index: 0,
            packet_bytes: [0; 3],
        }
    }

    pub fn init(&mut self) {
        unsafe {
            if wait_write() {
                io::outb(0x64, 0xA8);
            }
            if wait_write() {
                io::outb(0x64, 0x20);
            }
            if wait_read() {
                let mut status = io::inb(0x60);
                status |= 2;
                status &= !0x20;
                if wait_write() {
                    io::outb(0x64, 0x60);
                }
                if wait_write() {
                    io::outb(0x60, status);
                }
            }
            if wait_write() {
                io::outb(0x64, 0xD4);
            }
            if wait_write() {
                io::outb(0x60, 0xF4);
            }
        }
    }

    pub fn poll(&mut self) -> bool {
        let status = io::inb(0x64);
        if (status & 0x01) != 0 {
            let is_mouse = (status & 0x20) != 0;
            let data = io::inb(0x60);

            if is_mouse {
                if self.packet_index == 0 && (data & 0x08) == 0 {
                    return false;
                }

                self.packet_bytes[self.packet_index] = data;
                self.packet_index += 1;

                if self.packet_index >= 3 {
                    self.packet_index = 0;
                    let flags = self.packet_bytes[0];
                    let dx = self.packet_bytes[1] as i8 as i32;
                    let dy = self.packet_bytes[2] as i8 as i32;

                    self.x = (self.x + dx).clamp(0, 319);
                    self.y = (self.y - dy).clamp(0, 199);
                    self.left_button = (flags & 0x01) != 0;
                    return true;
                }
            }
        }
        false
    }
}

unsafe fn wait_write() -> bool {
    for _ in 0..1000 {
        if (io::inb(0x64) & 2) == 0 {
            return true;
        }
    }
    false
}

unsafe fn wait_read() -> bool {
    for _ in 0..1000 {
        if (io::inb(0x64) & 1) != 0 {
            return true;
        }
    }
    false
}


