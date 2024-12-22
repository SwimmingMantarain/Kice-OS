use core::{arch::{asm, naked_asm}, mem::size_of};
use crate::Color;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    options: u16,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

#[repr(C, packed)]
struct Idt {
    entries: [IdtEntry; 256],
}

impl Idt {
    const fn new() -> Self {
        Idt {
            entries: [IdtEntry {
                offset_low: 0,
                selector: 0,
                options: 0,
                offset_mid: 0,
                offset_high: 0,
                reserved: 0,
            }; 256],
        }
    }

    fn set_handler(&mut self, vector: usize, handler: extern "C" fn()) {
        let handler_addr = handler as u64;
        self.entries[vector] = IdtEntry {
            offset_low: handler_addr as u16,
            selector: 0x08, // Code segment selector
            options: 0x8E00, // Present, Ring 0, 32-bit interrupt gate
            offset_mid: (handler_addr >> 16) as u16,
            offset_high: (handler_addr >> 32) as u32,
            reserved: 0,
        };
    }
}

static mut IDT: Idt = Idt::new();

/// Handlers


/// Divide by Zero handler
#[no_mangle]
extern "C" fn divide_by_zero_handler() {
    // Handle the divide-by-zero exception here
    println!(Color::Green, Color::Black, "Stop with the zeros my man!");
    loop {}
}


/// Double Fault Handler
#[no_mangle]
extern "C" fn double_fault_handler(stack_frame: &InterruptStackFrame, error_code: u64) {
    println!(Color::Green, Color::Black,"Double Fault Exception! ->");
    println!(Color::Green, Color::Black,"Stack Frame: {:?}", stack_frame);
    println!(Color::Green, Color::Black,"Error Code: {}", error_code);
    loop {}
}

/// Wrapper for the double faults handler
#[naked]
extern "C" fn double_fault_wrapper() {
    unsafe {
        naked_asm!(
            // Push the handler address
            "lea rdi, [rsp + 0x10]",    // First argument: pointer to InterruptStackFrame
            "mov rsi, [rsp + 0x18]",    // Second argument: error code
            "call {handler}",           // Call the handler
            handler = sym double_fault_handler,
            options(),
        );
    }
}

/// Breakpoint Exception Handler
extern "C" fn breakpoint_handler(stack_frame: &InterruptStackFrame) {
    println!(Color::Green, Color::Black,"BREAKPOINT ->");
    println!(Color::Green, Color::Black,"Stack Frame: {:?}", stack_frame);
    loop {}
}

/// Wrapper for the above function 
#[naked]
extern "C" fn breakpoint_wrapper() {
    unsafe {
        naked_asm!(
            // Push the handler address
            "lea rdi, [rsp + 0x10]",    // First argument: pointer to InterruptStackFrame
            "mov rsi, [rsp + 0x18]",    // Second argument: error code
            "call {handler}",           // Call the handler
            handler = sym breakpoint_handler,
            options(),
        );
    }
}


// Interrupt Stack Frame (defined by x86_64 crate or custom struct)
#[repr(C)]
#[derive(Debug)]
struct InterruptStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}


/// Load IDT


#[repr(C, packed)]
struct IdtPointer {
    size: u16,
    base: u64,
}

unsafe fn load_idt() {
    let idt_ptr = IdtPointer {
        size: (size_of::<Idt>() - 1) as u16,
        base: &IDT as *const _ as u64,
    };
    asm!(
        "lidt [{0}]",
        in(reg) &idt_ptr,
        options(nostack, preserves_flags)
    );
}

pub fn init_idt() {
    unsafe {
        IDT.set_handler(0, divide_by_zero_handler);
        IDT.set_handler(8, double_fault_wrapper);
        IDT.set_handler(3, breakpoint_wrapper);
        load_idt();
    }
}


