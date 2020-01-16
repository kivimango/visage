#![no_std]
#![no_main]

mod vga_buffer;
use core::panic::PanicInfo;

/* Kernel entry point.
* Extern "C" for telling the compiler to use the C calling convention (at this time Rust has unspecified calling convention)
* no_mangle attribute disables the function name mangling, so the linker can find it by default.
* The ! return type means this is a diverging function: not allowed to ever return.
* This is required because the entry point is not called by any function, but invoked directly by the bootloader.
* Instead of returning, shutting down the machine could be a reasonable action, since there's nothing left to do if a freestanding binary returns.
* For now, we fulfill the requirement by looping endlessly. */
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("visage {}", "0.0.1");
    loop {}
}

/* The panic_handler attribute defines the function that the compiler should invoke when a panic occurs. 
 The standard library provides its own panic handler function, but in a freestanding environment we need to define it ourselves.
 The PanicInfo parameter contains the file and line where the panic happened and the optional panic message. 
 The function should never return, so it is marked as a diverging function by returning the “never” type "!"". */

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}
