#[allow(dead_code)]
pub fn outb(port: u16, val: u8) {
    unsafe {
        core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
    }
}


pub fn inb(port: u16) -> u8 {
    let result: u8;
    unsafe {
        core::arch::asm!("in al, dx", out("al") result, in("dx") port, options(nomem, nostack, preserves_flags));
    }
    result
}

pub fn outl(port: u16, val: u32) {
    unsafe {
        core::arch::asm!("out dx, eax", in("dx") port, in("eax") val, options(nomem, nostack, preserves_flags));
    }
}

pub fn inl(port: u16) -> u32 {
    let result: u32;
    unsafe {
        core::arch::asm!("in eax, dx", out("eax") result, in("dx") port, options(nomem, nostack, preserves_flags));
    }
    result
}


pub fn read_scancode() -> u8 {
    inb(0x60)
}
