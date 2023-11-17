/* Ooof, this is a big one, this is the "driver" for the vga. This "driver" allows the os to have Colors and shit like that */

use volatile::Volatile; // some imports
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use core::arch::asm;

lazy_static! { // this lazy static defines the default vga settings
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0, // start at collumn 0
        color_code: ColorCode::new(Color::White, Color::Black), // make the text white and the background black
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }, //  magic memory code
    });
}

#[allow(dead_code)] // Color Enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    Lightred = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)] // struct for color code
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode { // the implementation of the struct for color code
    pub fn new(foreground: Color, background: Color) -> ColorCode { // you can see that fore ground comes first and background second, this is why in the line 12 i have the first Color set to white and the second to black
        ColorCode((background as u8) << 4 | (foreground as u8)) 
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar { // screen characters
    pub ascii_character: u8,
    pub color_code: ColorCode,
}

pub const BUFFER_HEIGHT: usize = 25; // the screen resolution, it being 80x25px
pub const BUFFER_WIDTH: usize = 80;

#[repr(transparent)] // screen buffer
pub struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}


pub struct Writer { // Writer structs
    pub column_position: usize,
    pub color_code: ColorCode,
    pub buffer: &'static mut Buffer,
}

impl Writer { // implementation of the Writer
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });

                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) { // allows to write whole strings(println)
        if self.column_position >= BUFFER_WIDTH {
            self.new_line();
        }
    
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
    
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        // Shift characters up by one row, excluding the last row
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }

        // Clear the last row instead of the row above it
        self.clear_row(BUFFER_HEIGHT - 1);

        // Move the cursor to the beginning of the new line
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) { // clears the row
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn clear_character(&mut self, row: usize, col: usize) { // literally clears the character
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        self.buffer.chars[row][col].write(blank);
    }
}

pub fn print_something() { // i still use those functions cause you can change the color here!
    use core::fmt::Write;
    let mut writer = WRITER.lock();
    writer.color_code = ColorCode::new(Color::Cyan, Color::Black); // Customize the color if needed
    writer.write_string("Welcome to S.T.B. OS 0.9.8.5 by Admiralix!\n");
    writer.color_code = ColorCode::new(Color::White, Color::Black); // Restore the default color
}

pub fn ascii() {
    use core::fmt::Write;
    let mut writer = WRITER.lock();
    let ascii_art = r#"
    .d8888b.      88888888888     888888b.            .d88888b.   .d8888b.  
    d88P  Y88b         888         888  "88b          d88P" "Y88b d88P  Y88b 
    Y88b.              888         888  .88P          888     888 Y88b.      
     "Y888b.           888         8888888K.          888     888  "Y888b.   
        "Y88b.         888         888  "Y88b         888     888     "Y88b. 
          "888         888         888    888         888     888       "888 
    Y88b  d88P d8b     888     d8b 888   d88P d8b     Y88b. .d88P Y88b  d88P 
     "Y8888P"  Y8P     888     Y8P 8888888P"  Y8P      "Y88888P"   "Y8888P"  
    "#;
    writer.color_code = ColorCode::new(Color::LightGreen, Color::Black); // Customize the color if needed
    writer.write_string(ascii_art);
    writer.color_code = ColorCode::new(Color::White, Color::Black); // Restore the default color
}

pub fn print_error1() {
    use core::fmt::Write;
    let mut writer = WRITER.lock();
    writer.color_code = ColorCode::new(Color::Red, Color::Black); // Customize the color if needed
    writer.write_string("\nuse /syshelp to get a list of all possible commands\n");
    writer.color_code = ColorCode::new(Color::White, Color::Black); // Restore the default color 
}

pub fn OK() {
    use core::fmt::Write;
    let mut writer = WRITER.lock();
    writer.color_code = ColorCode::new(Color::LightGreen, Color::Black); // Customize the color if needed
    writer.write_string("\nOK\n");
    writer.color_code = ColorCode::new(Color::White, Color::Black); // Restore the default color 
}

pub fn print_shutdown() {
    use core::fmt::Write;
    
    {
        let mut writer = WRITER.lock();
        writer.color_code = ColorCode::new(Color::Yellow, Color::Black); // Customize the color if needed
        writer.write_string("It is now safe to turn off your computer\n");
    } // The lock is released here, and the changes to show_prompt and color_code are discarded.

    // Restore the default color
    let mut writer = WRITER.lock();
    writer.color_code = ColorCode::new(Color::White, Color::Black);
}

impl fmt::Write for Writer { // yes
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export] // this is some smart fuckery that allows to use print and println without having std, this also makes print and println use this Writer code.
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)] // some more shit to make print and println work
pub fn _print(args: fmt::Arguments) {
    use x86_64::instructions::interrupts;
    use core::fmt::Write;
    interrupts::without_interrupts(||{
        WRITER.lock().write_fmt(args).unwrap();

    });
    
}