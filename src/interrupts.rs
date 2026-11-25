use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::io;

lazy_static::lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt[32].set_handler_fn(timer_interrupt_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
    // Настройка PIT (Channel 0, Square wave, Rate generator)
    let divisor = 1193180 / 100;
    unsafe {
        io::outb(0x43, 0x36);
        io::outb(0x40, (divisor & 0xFF) as u8);
        io::outb(0x40, ((divisor >> 8) & 0xFF) as u8);
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        io::outb(0x20, 0x20); // EOI
    }
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: PageFaultErrorCode)
{
    panic!("EXCEPTION: PAGE FAULT\n{:#?}", stack_frame);
}
