// the x86-interrupt calling convention is still unstable
// TODO: remove the annotation when it is stable
#![no_std]
#![feature(abi_x86_interrupt)]
pub mod interrupts;
pub mod vga_buffer;
pub mod gdt;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
}