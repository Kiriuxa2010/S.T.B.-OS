#![no_std]
#![no_main]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]


use core::panic::PanicInfo;
mod vga_buffer;
// mod serial;

//static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // vga_buffer::clear();
    admiralix_os::init();
    println!("▄▄▄▄ ADMIRALIX OS 0.7 ▄▄▄▄\n");
    admiralix_os::hlt_loop();


    // x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    // println!("It did not crash!");
    // println!("                                                                                                        ");
    // println!("                                                                                                        ");
    
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    admiralix_os::hlt_loop();
    loop {}
}



#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}
#[test_case]
fn trivial_assertion() {
    print!("trivial assertion...");
    assert_eq!(1,1);
    println!("[ok]");
}

