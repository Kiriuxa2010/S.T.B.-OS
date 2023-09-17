use crate::{print, println, task::getcpu::{get_cpu_name, print_cpu_name}, vga_buffer::{print_shutdown}};
use conquer_once::spin::OnceCell;
use alloc::string::String;
use core::arch::asm;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker,
};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use x86_64::instructions::hlt;

// mod getcpu;
use bootloader::{BootInfo, entry_point};
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};


static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

const OSVER: &str = "0.9.8";

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate.
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
    if scancode == 0x0F { // TAB Key
        for n in 1..26 {
            println!("                          ");
        }
    }  
    // if scancode == 0x0EF {
    //     clear_character();
       
    // }
}


pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        // fast path
        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

    let mut user_input = String::new();  // Buffer to store user input

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => {
                        if character == '\n' {
                            // User pressed Enter
                            if user_input.trim() == "/exit" {
                                // User typed "/exit," execute shutdown logic
                                for n in 1..26 {
                                    println!("                          ");
                                }
                                print_shutdown();
                                hlt();
                                loop{}
                            }
                            if user_input.trim() == "/sysinf" {
                                println!("                          ");
                                println!("=======System Information========\n");
                                println!(" OS: S.T.B. OS by Admiralix      \n");
                                println!(" OS VERSION: {}                  \n", OSVER );
                                println!(" GPU: VGA                        \n");
                                if let Some(cpu_name) = get_cpu_name() {
                                    print_cpu_name(&cpu_name);
                                } else {
                                    println!("Failed to retrieve CPU name.");
                                }
                                println!(" RES: 80x25                      \n");
                                println!(" RAM: UNKNOWN                    \n");
                                println!("=================================\n");
                            }
                            if user_input.trim() == "/syshelp" {
                                println!("==========System Help================   \n");
                                println!(" /syshelp = Displays This Information   \n");
                                println!(" TAB = Clears the Screen                \n");
                                println!(" /sysinf = Shows System Information     \n");
                                println!(" /exit = 'Shuts' PC down                \n");
                                println!("=====================================   \n");
                            }
                            // if user_input.trim() == "/echo" {
                            //     let echo_message = " Enter text to echo:";
                            //     let mut echo_input = String::new();
                            
                            //     // Print the echo message
                            //     println!("{}", echo_message);
                            
                            //     // Read user input until they press Enter
                            //     loop {
                            //         let scancode = scancodes.next().await.unwrap();
                            //         if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                            //             if let Some(key) = keyboard.process_keyevent(key_event) {
                            //                 match key {
                            //                     pc_keyboard::DecodedKey::Unicode(character) => {
                            //                         if character == '\n' {
                            //                             break;
                            //                         }
                            //                         echo_input.push(character);
                            //                         print!("{}", character);
                            //                     }
                            //                     _ => {}
                            //                 }
                            //             }
                            //         }
                            //     }
                            
                            //     // Print the echoed text
                            //     println!("\nYou typed: {}", echo_input);
                            // }
                            if user_input.starts_with("/echo") {
                                let echo_text = user_input[5..].trim();
                                if !echo_text.is_empty() {
                                    println!("\n {}", echo_text);
                                }
                            }
                            

                            // Add a newline character to go to the next line
                            print!("{}", character);
                            println!();  // Add this line

                            user_input.clear();  // Clear the input buffer
                        } else {
                            // Append typed character to the input buffer
                            user_input.push(character);
                            print!("{}", character);
                        }
                    }
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}
