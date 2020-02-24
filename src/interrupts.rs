use crate::println;
use crate::print;
use crate::gdt;
use lazy_static::lazy_static;
use pc_keyboard::{Keyboard, ScancodeSet1, layouts};
use pic8259_simple::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

static PICS: Mutex<ChainedPics> = Mutex::new(
    // wrong offsets leads to Undefined Behavior
    unsafe{ ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) }
);

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum InterruptIndex {
    // Intel 8253 timer uses line 0 of the primary PIC, but we remapped it, so it arrives to the CPU as interrupt 0 + 32 = 32
    Timer = PIC_1_OFFSET,
    Keyboard
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    // we loaded a valid TSS and interrupt stack table, we can set the stack index for our double fault handler in the IDT
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        // unsafe because the the caller must ensure that the used index is valid and not already used for another exception.
        // The CPU will switch to the double fault stack whenever a double fault occurs. Thus, we are able to catch all double faults, including kernel stack overflows.
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

lazy_static! {
    static ref KEYBOARD : Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1));
}

pub fn init_idt() {
    IDT.load();
    unsafe { PICS.lock().initialize(); }
    x86_64::instructions::interrupts::enable();
}

extern "x86-interrupt" fn timer_handler(_stack_frame: &mut InterruptStackFrame) {
    print!(".");
    eoi(InterruptIndex::Timer.as_u8());
}

extern "x86-interrupt" fn keyboard_handler(_stack_frame: &mut InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    use pc_keyboard::{DecodedKey};

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    eoi(InterruptIndex::Keyboard.as_u8());
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("Breakpoint exception occurred: {:#?}", stack_frame);
}

/**
 * Handles double fault exceptions.
 * IRQ index is 8, the error code is always 0.
 * A double fault exception can occur when a second exception occurs during the handling of a prior (first) exception handler.
 * 
 * Only very specific combinations of exceptions lead to a double fault. These combinations are:
 * First exception:
 * - Divide by zero,
 * - Invalid TSS,
 * - Segment Not Present,
 * - Stack Segment Fault,
 * - General Protection Fault,
 * - Page Fault.
 * 
 * Second exception:
 * - Invalid TSS,
 * - Segment Not Present,
 * - Stack Segment Fault,
 * - General Protection Fault
 * 
 *   if the first exception was Page Fault:
 * - Page Fault,
 * - Invalid TSS,
 * - Segment Not Present
 * - Stack Segment Fault,
 * - General Protection Fault.
 * This function is diverging because the x86_64 architecture does not permit returning from a double fault exception.
 */
extern "x86-interrupt" fn double_fault_handler(stack_frame: &mut InterruptStackFrame, _error_code: u64) -> !{
    panic!("Double Fault occurred: \n{:#?},\n, error code: {:#?} stopping kernel...", _error_code, stack_frame);
}

fn eoi(index : u8) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(index);
    }
}