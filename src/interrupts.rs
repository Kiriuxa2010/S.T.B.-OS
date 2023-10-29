#![feature(asm)]
#![feature(llvm_asm)]

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;
use crate::print;
use crate::gdt;
use pic8259::ChainedPics;
use crate::vga_buffer;
use spin;
use lazy_static::lazy_static;
// pub mod cpuid;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()]
           .set_handler_fn(timer_interrupt_handler);

        idt[InterruptIndex::Keyboard.as_usize()]
           .set_handler_fn(keyboard_interrupt_handler);

        idt.page_fault.set_handler_fn(page_fault_handler);

        idt
    };
 }

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

    pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}


extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

use x86_64::structures::idt::PageFaultErrorCode;
use crate::hlt_loop;

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame,error_code: PageFaultErrorCode,) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}



extern "x86-interrupt" fn timer_interrupt_handler( _stack_frame: InterruptStackFrame) {
    // print!(".");

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

fn get_cpu_name() -> Option<&'static str> {
    const MP_SPEC_BASE_ADDRESS: usize = 0x0000_0000;
    const MP_SPEC_SIGNATURE: u32 = 0x5F50_324D; // "_P32M" in little-endian

    let mp_spec_base_ptr: *const u32 = MP_SPEC_BASE_ADDRESS as *const u32;

    // Verify if MP Spec table exists at the specified base address
    if unsafe { core::ptr::read(mp_spec_base_ptr) } == MP_SPEC_SIGNATURE {
        let name_start_ptr = unsafe { mp_spec_base_ptr.add(11) }; // Offset for the CPU name within the MP Spec table
        let name_end_ptr = unsafe { name_start_ptr.add(20) }; // CPU name is 20 bytes long

        // Convert the CPU name to a string
        let cpu_name_bytes: &[u8] = unsafe { core::slice::from_raw_parts(name_start_ptr as *const u8, 20) };
        let cpu_name_str = core::str::from_utf8(cpu_name_bytes).ok()?;

        Some(cpu_name_str.trim())
    } else {
        None
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
        );
    }
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}