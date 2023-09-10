use crate::{print, println, task::getcpu::{get_cpu_name, print_cpu_name}, vga_buffer::{print_shutdown}};
use conquer_once::spin::OnceCell;
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

const OSVER: &str = "0.9.3";

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
    if scancode == 0x3B { // F1 Key
        println!("==========System Help============ \n");
        // println!("                                  \n");
        println!(" F1 = Dispalys This Information   \n");
        println!(" TAB = Clears the Screen          \n");
        println!(" CRTL = Shows System Information  \n");
        println!(" F10 = 'Shuts' PC down            \n");
        println!("================================= \n");
    }
    if scancode == 0x0F { // TAB Key
        for n in 1..26 {
            println!("                          ");
        }
    }
    if scancode == 0x1D {
        println!("=======System Information========\n");
        // println!("                                 \n");
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
    if scancode == 0x44 { // F10 Key    
        // TODO: Add a PC restart  
        // for n in 1..26 {
        //     println!("                          ");
        // }
        print_shutdown();
        hlt();
        loop{}
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

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}
