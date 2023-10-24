use crate::{print, println, task::getcpu::{get_cpu_name, print_cpu_name}, vga_buffer::{print_shutdown}};
use conquer_once::spin::OnceCell;
use alloc::string::String;
use lazy_static::lazy_static;
use spin::Mutex;
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
// mod getcpu;
use bootloader::{BootInfo, entry_point};
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

const OSVER: &str = "0.9.8.5";

lazy_static! {
    pub static ref CURRENT_DIRECTORY: Mutex<String> = Mutex::new("/".to_string());
}

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

// Define a file structure
#[derive(Clone)]
struct File {
    name: String,
    content: String,
}

// Define a directory structure
#[derive(Clone)]
struct Directory {
    name: String,
    files: Vec<File>,
    subdirectories: Vec<Directory>,
    parent: Option<usize>,
}

// Create the root directory
lazy_static! {
    static ref ROOT: Mutex<Directory> = Mutex::new(Directory {
        name: "$/".to_string(),
        files: vec![
            File {
                name: "file1.txt".to_string(),
                content: "This is file 1.".to_string(),
            },
            File {
                name: "file2.txt".to_string(),
                content: "This is file 2.".to_string(),
            },
        ],
        subdirectories: vec![
            Directory {
                name: "kernl".to_string(),
                files: vec![File {
                    name: "stbos.uff".to_string(),
                    content: "This is the Unreadable File Format, UFF for short".to_string(),
                }],
                subdirectories: vec![],
                parent: Some(1), // Set the parent to the index of the parent directory ("$/")
            },
        ],
        parent: None, // The root directory has no parent.
    });
}

// Function to list files in the current directory
fn ls() {
    let current_directory = ROOT.lock();
    println!("Contents of directory '{}':", current_directory.name);
    for file in &current_directory.files {
        println!("File: {}", file.name);
    }
    for subdir in &current_directory.subdirectories {
        println!("Directory: {}", subdir.name);
    }
}

fn cat(filename: &str) {
    let current_directory = ROOT.lock();
    if let Some(file) = current_directory.files.iter().find(|f| f.name == filename) {
        println!("\n{}", file.content);
    } else {
        println!("\nFile '{}' not found.", filename);
    }
}

// Function to change the current directory
fn cd(new_directory: &str) {
    let mut current_directory = ROOT.lock();

    if new_directory == ".." {
        if let Some(parent_index) = current_directory.parent {
            // Print some debug information
            println!("Changing to parent directory (index: {})", parent_index);

            // Change the current directory to the parent directory
            current_directory.parent = current_directory.subdirectories[parent_index].parent;
            *current_directory = current_directory.subdirectories[parent_index].clone();
        } else {
            println!("\nAlready at the root directory.");
        }
    } else {
        if let Some(sub_index) = current_directory.subdirectories.iter().position(|dir| dir.name == new_directory) {
            // Print some debug information
            println!("Changing to subdirectory (index: {})", sub_index);

            // Change the current directory to the specified subdirectory
            current_directory.parent = Some(sub_index);
            *current_directory = current_directory.subdirectories[sub_index].clone();
        } else {
            println!("\nDirectory '{}' not found.", new_directory);
        }
    }
}

fn mkdir(new_directory: &str) {
    let mut current_directory = ROOT.lock();

    // Check if the new directory name is empty
    if new_directory.is_empty() {
        println!("\nDirectory name cannot be empty.");
        return;
    }

    // Check if the new directory name already exists
    if current_directory.subdirectories.iter().any(|dir| dir.name == new_directory) {
        println!("\nDirectory '{}' already exists.", new_directory);
        return;
    }

    // Create the new directory and add it to the current directory
    current_directory.subdirectories.push(Directory {
        name: new_directory.to_string(),
        files: vec![],
        subdirectories: vec![],
        parent: Some(0), // Set the parent to the current directory
    });

    println!("\nDirectory '{}' created.", new_directory);
}

fn touch(filename: &str, content: &str) {
    let mut current_directory = ROOT.lock();

    // Check if the new file name is empty
    if filename.is_empty() {
        println!("\nFile name cannot be empty.");
        return;
    }

    // Check if the new file name already exists
    if current_directory.files.iter().any(|file| file.name == filename) {
        println!("\nFile '{}' already exists.", filename);
        return;
    }

    // Create the new file and add it to the current directory
    current_directory.files.push(File {
        name: filename.to_string(),
        content: content.to_string(),
    });

    println!("\nFile '{}' created.", filename);
}

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);
    let mut user_input = String::new();  // Buffer to store user input
    let mut echo_text = String::new();
    let new_dir = String::new();

    let ascii_art =r#"
      ######  ######### #########          ###         ######
    ##           ###    ###   ###      ###     ###   ##
    ##           ###    ###    ###    ###       ###  ##
      ######     ###    ########      ###       ###    ######
           ##    ###    ###    ###    ###       ###         ##
           ##    ###    ###   ###       ###   ###           ##
     ######      ###    #########          ###         ######                
   
    "#;

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
                                println!("\n{}", ascii_art);
                                println!(" OS: S.T.B. OS by Admiralix      \n");
                                println!(" OS VERSION: {}                  \n", OSVER );
                                println!(" GPU: VGA                        \n");
                                if let Some(cpu_name) = get_cpu_name() {
                                    print_cpu_name(&cpu_name);
                                } else {
                                    println!("Failed to retrieve CPU name.");
                                }
                                println!(" RES: 80x25px                    \n");
                                println!(" RAM Size: UNKNOWN");
                                println!("=================================\n");
                            } else if user_input.trim() == "/syshelp" {
                                println!("\n");
                                println!("==========System Help==================   \n");
                                println!(" /syshelp = Display's This Information    \n");
                                println!(" /clear = Clears the Screen               \n");
                                println!(" /sysinf = Shows System Information       \n");
                                println!(" /shutdown = 'Shuts' PC down              \n");
                                println!(" /echo = Echoes text                      \n");
                                println!(" /refr echo = references the echo input   \n");
                                println!("EXPERIMENTAL:                             \n");
                                println!("/cd = change dir.                         \n");
                                println!("/lf = list files                          \n");
                                println!("/sw = show content of files               \n");
                                println!("/mkdir = makes a dir.                     \n");
                                println!("/tch = makes a new file.                  \n");
                                println!("=======================================   \n");
                            } else if user_input.starts_with("/echo ") {
                                // Echo command
                                echo_text = user_input[6..].trim().to_string();
                                if !echo_text.is_empty() {
                                    println!("\n{}", echo_text);
                                }
                            } else if user_input.trim() == "/refr echo" {
                                println!("\n--->    {}", echo_text);
                            } else if user_input.trim() == "/clear" {
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
                            } else if character == '\u{0008}' {
                                // Handle Backspace key
                                if !user_input.is_empty() {
                                    // Remove the last character from the input buffer
                                    user_input.pop();
                                    // Print the backspace character to erase the character on the screen
                                    print!("\u{0008} \u{0008}");
                                }
                            } else {
                                // Unknown command
                                println!("\nUnknown Command: '{}', use /syshelp to get a list of all possible commands", user_input.trim());
                            }

                            // Add a newline character to go to the next line
                            user_input.clear();  // Clear the input buffer

                        } else if character == '\u{0008}' {
                            // Handle Backspace key
                            if !user_input.is_empty() {
                                // Remove the last character from the input buffer
                                user_input.pop();
                                // Print the backspace character to erase the character on the screen
                                print!("\r");
                                // Print a space to erase the character
                                print!(" ");
                                // Move the cursor back again
                                print!("\r");
                            }
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