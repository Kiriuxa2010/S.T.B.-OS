/*This is the Main.rs file, it brings all the stuff from the other .rs files into one file. */

#![no_std] // allows the code to compile to baremetal
#![no_main] // no fn main, instead it uses fn kernel_main and an entry point.
#![reexport_test_harness_main = "test_main"] // i dont think thats needed
#![feature(custom_test_frameworks)] // allows for use of custom libs(lib.rs)
#![feature(asm)] // allows for inline assembly
#![test_runner(crate::test_runner)] // crate

use core::panic::PanicInfo; // imports
mod vga_buffer; // literally the vga buffer import
use bootloader::{BootInfo, entry_point}; // bootloader(duh)
use crate::vga_buffer::{Writer, WRITER}; // import Writer an WRITER from vga_buffer
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc}; // allocation stuff
use admiralix_os::task::{executor::Executor, keyboard, Task}; // these are the lib.rs imports
use core::arch::asm; // inline assembly
use x86_64::instructions::port::Port; // what do you think this is?
use x86_64::instructions::hlt; // oh what could this possibly be?

extern crate alloc; // alloc
entry_point!(kernel_main); // this is the entry point for the os

fn kernel_main(boot_info: &'static BootInfo) -> ! { // entry point, boots from here
    use admiralix_os::memory::BootInfoFrameAllocator; // some more imports from lib.rs like memory management, allocations, and keyboard
    use admiralix_os::allocator;
    use admiralix_os::memory;
    use admiralix_os::task::{executor::Executor, keyboard, Task};
    use x86_64::{structures::paging::Page, VirtAddr}; 

    let osname = "S.T.B."; 
    println!("Starting {} OS...\n", osname);

    delay(5); // make the os have a delay of 5 seconds to make it look bussier
    vga_buffer::print_something(); // this is the "Welcome to STB OS" text
    admiralix_os::init(); // initalize the stuff from lib.rs
    
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset); // some memory stuff
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) }; // this allocates the frame memory system time at 0x8493 and boot memory map, it also boot_info
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let mut executor = Executor::new(); // task executor spawner

    executor.spawn(Task::new(keyboard::print_keypresses())); // this here spawns the keyboard task
    executor.run();

    admiralix_os::hlt_loop();
}

fn delay(seconds: u32) { // this is the delay(to make the os look bussier), and it also has some inline assembly here
    let iterations_per_second = 1000000;
    let total_iterations = seconds * iterations_per_second;

    for _ in 0..total_iterations {
        unsafe{
            asm!
            (
            "nop",
            options(nostack)
            );
        }
    }
}

#[cfg(not(test))] // tester, never used it
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    admiralix_os::hlt_loop();
    loop {}
}