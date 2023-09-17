#![no_std] // allows the code to compile to baremetal
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![feature(asm)]
#![test_runner(crate::test_runner)]

use core::panic::PanicInfo; // imports
mod vga_buffer;
// mod getcpu;
use bootloader::{BootInfo, entry_point};
use crate::vga_buffer::{Writer, WRITER};
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use admiralix_os::task::{executor::Executor, keyboard, Task};
use core::arch::asm;
use x86_64::instructions::port::Port;
use x86_64::instructions::hlt;

extern crate alloc;
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! { // main function, boots from here
    use admiralix_os::memory::BootInfoFrameAllocator;
    use admiralix_os::allocator;
    use admiralix_os::memory;
    use admiralix_os::task::{executor::Executor, keyboard, Task};
    use x86_64::{structures::paging::Page, VirtAddr}; 

    let osname = "S.T.B.";
    println!("Starting {} OS...\n", osname);

    delay(5);
    
    play_beep();

    vga_buffer::print_something();
    admiralix_os::init();
    
    //let mut writer = vga_buffer::Writer::new(vga_buffer::Color::Yellow, vga_buffer::BUFFER.lock());
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) }; // this allocates the frame memory system time at 0x8493 and boot memory map, it also boot_info
 
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let mut executor = Executor::new();
    //executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    admiralix_os::hlt_loop();
}

fn delay(seconds: u32) {
    let iterations_per_second = 1000000;
    let total_iterations = seconds * iterations_per_second;

    for _ in 0..total_iterations {
        unsafe{
            asm!
            ("nop",
            
            options(nostack)
            );
        }
    }
}

pub fn play_beep() {
    // Define the I/O ports for the speaker
    const SPEAKER_CONTROL_PORT: u16 = 0x61;
    const SPEAKER_DATA_PORT: u16 = 0x42;

    // Disable the speaker by clearing the least significant bit of the control port
    let mut control_port = Port::new(SPEAKER_CONTROL_PORT);
    let control_byte: u8 = unsafe { control_port.read() };
    unsafe {
        control_port.write(control_byte & 0xFE);
    }

    // Set the frequency for the beep (adjust this to change the pitch)
    let mut data_port = Port::new(SPEAKER_DATA_PORT);
    let frequency: u32 = 1000; // Adjust this for the desired frequency
    let divisor: u32 = 1193180 / frequency;

    // Send the command to the speaker
    unsafe {
        data_port.write((divisor & 0xFF) as u8);
        data_port.write((divisor >> 8) as u8);
    }

    // Enable the speaker by setting the least significant bit of the control port
    unsafe {
        control_port.write(control_byte | 0x01);
    }

    // Wait for a short duration to allow the beep to play (you can adjust this)
    for _ in 0..10000 {
        unsafe {
            asm!
            (
                "nop",
                options(nostack) 
            );
        }
    }

    // Disable the speaker again
    unsafe {
        control_port.write(control_byte & 0xFE);
    }
}



#[cfg(not(test))] // tester 
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    admiralix_os::hlt_loop();
    loop {}
}

// async fn async_number() -> u32 {
//     42
// }

//async fn example_task() {
//    let number = async_number().await;
//    println!("async number: {}", number);
//}