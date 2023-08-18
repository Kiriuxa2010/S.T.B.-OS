// this code allows keyboard input and shortcuts such as system information

use crate::{print, println, task::getcpu::{get_cpu_name, print_cpu_name}}; // imports
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

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

const osver: &str = "0.9.4";

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate.
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input"); // if the scancodes are full, the keyboard input drops thus you cant type anymore
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
    if scancode == 0x3B { // F1 Key, prints system help(duh)
        println!("==========System Help============ \n");
        println!("                                  \n");
        println!(" F1 = Dispalys This Information   \n");
        println!(" TAB = Clears the Screen          \n");
        println!(" CRTL = Shows System Information  \n");
        println!("================================= \n");
    }
    if scancode == 0x0F { // TAB Key, clears the screen by 26 rows 
        for n in 1..26 {
            println!("                          ");
        }
    }
    if scancode == 0x1D { // prints the system informarion(duh)
        println!("=======System Information========\n");
        println!("                                 \n");
        println!(" OS: S.T.B. OS by Admiralix      \n");
        println!(" OS VERSION: {}                  \n", osver );
        println!("                                 \n ");
        println!(" GPU: VGA                        \n");
        if let Some(cpu_name) = get_cpu_name() {
            print_cpu_name(&cpu_name);
        } else {
            println!("FAILED TO GET CPU NAME AND INFORMATION ERROR CODE:");
        }
        println!(" RES: 80x25                      \n");
        println!(" RAM: UNKNOWN                    \n");
        println!("=================================\n");
    }
    if scancode == 0x44 { // F10 Key    
        // shutdown function isnt here cause i forgot to update repository 
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

pub async fn print_keypresses() { // this code is the main part, this code is responsible for keyboard input
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
