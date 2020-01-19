use crate::println;
use crate::gdt;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    // we loaded a valid TSS and interrupt stack table, we can set the stack index for our double fault handler in the IDT
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        // unsafe because the the caller must ensure that the used index is valid and not already used for another exception.
        // The CPU will switch to the double fault stack whenever a double fault occurs. Thus, we are able to catch all double faults, including kernel stack overflows.
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
       
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("Breakpoint exception occured: {:#?}", stack_frame);
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
    panic!("Double Fault occured: \n{:#?},\n stopping kernel...", stack_frame);
}