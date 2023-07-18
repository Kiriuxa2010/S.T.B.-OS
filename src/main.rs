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
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use admiralix_os::task::{executor::Executor, keyboard, Task};
use core::arch::asm;

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

    vga_buffer::print_something();
    admiralix_os::init();
    
    //let mut writer = vga_buffer::Writer::new(vga_buffer::Color::Yellow, vga_buffer::BUFFER.lock());
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let mut executor = Executor::new();
    //executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    admiralix_os::hlt_loop();
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