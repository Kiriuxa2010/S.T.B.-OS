/* Yeah this just compiles all the stuff from other .rs files(keyboard.rs for exampel) to make it all callable in the main.rs*/

#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![feature(alloc_error_handler)] 
#![reexport_test_harness_main = "test_main"]
#![feature(const_mut_refs)]

use core::panic::PanicInfo;

pub mod task;
pub mod interrupts;
pub mod serial;
pub mod vga_buffer;
pub mod gdt;
pub mod memory;
pub mod allocator;
pub mod stbfs;
pub mod getcpu;

extern crate alloc;

pub fn init() { // this is the initialization of everything
    // shell::init_shell(); LMFAO this was me trying to add commands back when i was stupid
    interrupts::init_idt();
    gdt::init();
    unsafe { interrupts::PICS.lock().initialize()};
    x86_64::instructions::interrupts::enable();
}
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

pub fn hlt_loop() -> ! {
    loop{
        x86_64::instructions::hlt();
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

// #[cfg(test)]
// entry_point!(test_kernel_main);

// Entry point for `cargo xtest`
#[cfg(test)]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
