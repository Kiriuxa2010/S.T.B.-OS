/* This is probably the most important code(except for vga buffer and main), this adds keyboard support and commands! */

// some imports
use crate::{print, println, task::getcpu::{get_cpu_name, print_cpu_name}, vga_buffer::{print_shutdown, ascii, print_error1}, stbfs::{ls, cd, mkdir, touch, cat}};
use conquer_once::spin::OnceCell;
use alloc::string::String;
use lazy_static::lazy_static;
use spin::Mutex;
use pc_keyboard::KeyCode;
use core::arch::asm;
use core::str::FromStr;
use alloc::string::ToString;
use alloc::format;
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
use crate::vga_buffer::WRITER;
use crate::vga_buffer::BUFFER_HEIGHT;
use bootloader::{BootInfo, entry_point};
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

const OSVER: &str = "0.9.8.5";

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
    let mut echo_text = String::new();
    let new_dir = String::new();

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => {
                        if character == '\n' {
                            // User pressed Enter
                            if user_input.trim() == "/shutdown" {
                                // User typed "/exit," execute shutdown logic
                                for n in 1..26 {
                                    println!("\n");
                                }
                                print_shutdown();
                                hlt();
                                loop {}
                            } else if user_input.trim() == "/sysinf" {
                                println!("\n=======System Information========\n");
                                ascii();
                                println!("OS: S.T.B. OS by Admiralix      ");
                                println!("OS VERSION: {} Build 09866      ", OSVER );
                                if let Some(cpu_name) = get_cpu_name() {
                                    print_cpu_name(&cpu_name);
                                } else {
                                    println!("Failed to retrieve CPU name.");
                                }
                                println!("RES: 80x25px                    ");
                                println!("RAM Size: UNKNOWN");
                                println!("=================================");
                            } else if user_input.trim() == "/syshelp" {
                                println!("\n");
                                println!("==========System Help==================   \n");
                                println!("/syshelp = Display's This Information    ");
                                println!("/cls = Clears the Screen               ");
                                println!("/sysinf = Shows System Information       ");
                                println!("/shutdown = 'Shuts' PC down              ");
                                println!("/echo = Echoes text                      ");
                                println!("/refr echo = references the echo input   ");
                                println!("EXPERIMENTAL:                             ");
                                println!("/cd = change dir.                         ");
                                println!("/lf = list files                          ");
                                println!("/sw = show content of files               ");
                                println!("/mkdir = makes a dir.                     ");
                                println!("/tch = makes a new file.                  ");
                                println!("=======================================   ");
                            } else if user_input.trim() == "/who" {
                                println!("\nUSER: AOS User");
                                println!("USER PRIVILEGES: Administrator");
                            } else if user_input.starts_with("/echo ") {
                                // Echo command
                                echo_text = user_input[6..].trim().to_string();
                                if !echo_text.is_empty() {
                                    println!("\n{}", echo_text);
                                }
                            } else if user_input.trim() == "/refr echo" {
                                println!("\n--->    {}", echo_text);
                            } else if user_input.trim() == "/cls" {
                                for n in 1..26 {
                                    println!("\n");
                                }
                            } else if user_input.trim() == "/lf" {
                                println!("\n");
                                ls();
                            } else if user_input.starts_with("/cd ") {
                                let new_directory = &user_input[4..].trim();
                                cd(new_directory);
                            } else if user_input.starts_with("/sw ") {
                                let filename = &user_input[4..].trim();
                                cat(filename);
                            } else if user_input.starts_with("/mkdir ") {
                                let filename = &user_input[6..].trim();
                                mkdir(filename);
                            } else if user_input.starts_with("/tch ") {
                                let input = &user_input[5..].trim(); // Trim additional spaces
                                let parts: Vec<&str> = input.splitn(2, ' ').collect();
                            
                                if parts.len() != 2 {
                                    println!("\nUsage: /tch <filename> <content>");
                                } else {
                                    let filename = parts[0];
                                    let content = parts[1];
                                    touch(filename, content);
                                }
                            } else if user_input.trim() == "/1000_1C3" {
                                println!("\nThanks for Using S.T.B. OS!\n");
                                println!("Admiralix Team:               ");
                                println!("icewallowpiz - Leader of Project and Lead Programmer\n");
                                println!("Contributors:                 ");
                                println!("DAWOOD - Lead Website Designer\n");
                                println!("Pr1thv1 - Fixed a Keyboard problem");
                                println!("Special Thanks to:");
                                println!("Snneezou");
                                

                            } else if character == '\u{0008}' {
                                // Handle Backspace key
                                if !user_input.is_empty() {
                                    // Remove the last character from the input buffer
                                    user_input.pop();
                                    // Print the backspace character to erase the character on the screen
                                    // print!("\u{0008} \u{0008}");
                                }
                            } else {
                                // Unknown command
                                println!("\nUnknown Command: '{}'", user_input.trim());
                                print_error1();
                            }

                            // Add a newline character to go to the next line
                            user_input.clear();  // Clear the input buffer

                        } else if character == '\u{0008}' {
                            // Handle Backspace key
                            if !user_input.is_empty() {
                                // Remove the last character from the input buffer
                                user_input.pop();
                                // Print the backspace character to erase the character on the screen
                                print!("\u{0008} \u{0008}");
                            }
                        } else {
                            // Append typed character to the input buffer
                            user_input.push(character);
                            print!("{}", character);
                        }
                    }
                    DecodedKey::RawKey(key) => { // Use RawKey from pc-keyboard crate
                        // Filter out unwanted keys
                        match key {
                            KeyCode::ArrowLeft | KeyCode::ArrowRight | KeyCode::ArrowUp | KeyCode::ArrowDown | KeyCode::Backspace => {
                                // Ignore arrow keys
                            }
                            _ => {
                                // Handle other raw keys
                                print!("{:?}", key);
                            }
                        }
                    }
                }
            }
        }
    }
}