#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![allow(bad_asm_style)]
#![allow(dead_code)]

mod sdf_engine;
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
mod window;
mod terminal;
mod font_atlas;
mod interrupts;


use core::panic::PanicInfo;

bootloader::entry_point!(kernel_main);

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

pub fn kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
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
    // crate::interrupts::init_idt(); 
    // unsafe { x86_64::instructions::interrupts::enable(); } 
    // crate::pci::scan_pci();

        );
    }
    // crate::interrupts::init_idt(); // Закомментировали, чтобы временно отключить прерывания
    // Диагностика PCI
    crate::pci::scan_pci();

    // unsafe { x86_64::instructions::interrupts::enable(); } // И это тоже


    sdf::init();
    let mut mouse = crate::mouse::Mouse::new();
    mouse.init();
    let mut terminal = crate::terminal::Terminal::new();

    let mut win = crate::window::Window::new(100, 50, 120, 80);

    let mut old_mx = mouse.x;
    let mut old_my = mouse.y;
    let mut was_pressed = false;

    sdf::render_frame_with_terminal(win.x, win.y, win.w, win.h, win.is_fullscreen, mouse.x, mouse.y, &terminal);

    loop {
        let mut input_updated = false;
        
        // Читаем ТОЛЬКО клавиатурные пакеты с портов (проверяя статус бит 5), не трогая пакеты мыши
        while crate::keyboard::poll_keyboard(&mut terminal) {
            input_updated = true;
        }

        if mouse.poll() || input_updated {
            let dx = mouse.x - old_mx;
            let dy = mouse.y - old_my;

            let click_event = mouse.left_button && !was_pressed;
            was_pressed = mouse.left_button;

            if click_event {
                if mouse.x >= win.x + win.w - 16 && mouse.x <= win.x + win.w - 4 && mouse.y >= win.y && mouse.y <= win.y + 14 {
                    win.toggle_fullscreen();
                }
                else if !win.is_fullscreen && mouse.x >= win.x + win.w - 12 && mouse.x <= win.x + win.w && mouse.y >= win.y + win.h - 12 && mouse.y <= win.y + win.h {
                    win.resizing = true;
                }
                else if mouse.x >= win.x && mouse.x < win.x + win.w && mouse.y >= win.y && mouse.y < win.y + 14 {
                    if !win.is_fullscreen {
                        win.dragging = true;
                    }
                }
            }

            if mouse.left_button {
                if win.dragging {
                    win.x = (win.x + dx).clamp(0, 320 - win.w);
                    win.y = (win.y + dy).clamp(0, 180 - win.h);
                } else if win.resizing {
                    win.w = (win.w + dx).clamp(50, 320 - win.x);
                    win.h = (win.h + dy).clamp(35, 180 - win.y);
                }
            } else {
                win.dragging = false;
                win.resizing = false;
            }

            old_mx = mouse.x;
            old_my = mouse.y;

            sdf::render_frame_with_terminal(win.x, win.y, win.w, win.h, win.is_fullscreen, mouse.x, mouse.y, &terminal);
        }
    }
}
