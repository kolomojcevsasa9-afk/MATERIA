#![no_std]
#![no_main]
#![allow(bad_asm_style)]
#![allow(dead_code)]


mod vga_buffer;
extern crate rlibc;
mod io;
mod keyboard;
mod sfs;
mod shell;
mod pci;
mod nvme;
mod allocator;
mod mouse;
mod graphics;
mod sdf;

use core::panic::PanicInfo;

bootloader::entry_point!(kernel_main);

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

pub fn kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    // Включаем FPU/SSE
    unsafe {
        core::arch::asm!(
            "mov rax, cr0",
            "and ax, 0xFFFB",
            "or ax, 0x0002",
            "mov cr0, rax",
            "mov rax, cr4",
            "or ax, 0x0600",
            "mov cr4, rax",
            options(nostack, preserves_flags)
        );
    }

    sdf::init();
    let mut mouse = crate::mouse::Mouse::new();
    mouse.init();

    let mut old_x = mouse.x;
    let mut old_y = mouse.y;

    // Первоначальный рендеринг курсора в начальной позиции
    sdf::render_cursor(old_x, old_y, mouse.x, mouse.y);

    // Ультра-быстрый цикл: перерисовываем только dirty-область при движении курсора
    loop {
        if mouse.poll() || old_x != mouse.x || old_y != mouse.y {
            sdf::render_cursor(old_x, old_y, mouse.x, mouse.y);
            old_x = mouse.x;
            old_y = mouse.y;
        }
    }
}




