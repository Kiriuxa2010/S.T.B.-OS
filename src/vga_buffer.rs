use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::LightGray, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        show_prompt: false, // Add a flag to control whether "C:" should be displayed
    });
}

#[allow(dead_code)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
    show_prompt: bool, // Flag to control whether "C:" should be displayed
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                if self.show_prompt {
                    self.write_string("user$aos: ");
                }

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });

                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        if self.column_position >= BUFFER_WIDTH {
            self.new_line();
        }
    
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
    
        if col < 11 {
            self.write_byte(b'u'); // Display 'u'
            self.write_byte(b's'); // Display 's'
            self.write_byte(b'e'); // Display 'e'
            self.write_byte(b'r'); // Display 'r'
            self.write_byte(b'$'); // Display '$'
            self.write_byte(b'a'); // Display 'a'
            self.write_byte(b'o'); // Display 'o'
            self.write_byte(b's'); // Display 's'
            self.write_byte(b':'); // Display ':'
            self.write_byte(b' '); // Display ' '
        }
    
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

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    fn clear_character(&mut self, row: usize, col: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        self.buffer.chars[row][col].write(blank);
    }
}

pub fn print_something() {
    use core::fmt::Write;
    let mut writer = WRITER.lock();
    writer.show_prompt = false; // Hide "C:" for this message
    writer.color_code = ColorCode::new(Color::Cyan, Color::Black); // Customize the color if needed
    writer.write_string("Welcome to S.T.B. OS by Admiralix!\n");
    writer.show_prompt = true; // Restore "C:" display
    writer.color_code = ColorCode::new(Color::LightGray, Color::Black); // Restore the default color
}

// pub fn print_bsod() {
//     use core::fmt::Write;
//     let mut writer = Writer {
//         column_position: 0,
//         color_code: ColorCode::new(Color::White, Color::Blue),
//         buffer:unsafe { &mut *(0xb8000 as *mut Buffer)},
//     };
//     writer.write_string("BSoD\n");
// }

pub fn print_shutdown() {
    use core::fmt::Write;
    
    {
        let mut writer = WRITER.lock();
        writer.show_prompt = false; // Hide "C:" for this message
        writer.color_code = ColorCode::new(Color::Yellow, Color::Black); // Customize the color if needed
        writer.write_string("It is now safe to turn off your computer\n");
    } // The lock is released here, and the changes to show_prompt and color_code are discarded.

    // Restore "C:" display and the default color
    let mut writer = WRITER.lock();
    writer.show_prompt = true;
    writer.color_code = ColorCode::new(Color::LightGray, Color::Black);
}


impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use x86_64::instructions::interrupts;
    use core::fmt::Write;
    interrupts::without_interrupts(||{
        WRITER.lock().write_fmt(args).unwrap();

    });
    
}