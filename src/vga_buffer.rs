use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

/** A global writer instance used by print!() and println!() macros.
* The framebuffer is accessible just like normal RAM, at address 0xB8000.
* It is important to note, however, that it is not actually normal RAM.
* It is part of the VGA controller's dedicated video memory that has been memory-mapped via hardware into your linear address space.
* The framebuffer is just an array of 16-bit words, each 16-bit value representing the display of one character.
* In ASCII, 8 bits are used to represent a character.
* That gives us 8 more bits which are unused. The VGA hardware uses these to designate foreground and background colors (4 bits each).
*/
lazy_static! {
    pub static ref WRITER : Mutex<Writer> = Mutex::new(Writer {
        column_pos : 0,
        row_pos : 1,
        color_code : ColorCode::new(Colors::White, Colors::Black),
        buffer : unsafe {&mut *(0xB8000 as *mut Buffer) }
    });
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Colors {
    Black       = 0,
    Blue        = 1,
    Green       = 2,
    Cyan        = 3,
    Red         = 4,
    Magenta     = 5,
    Brown       = 6,
    LightGray   = 7,
    DarkGray    = 8,
    LightBlue   = 9,
    LightGreen  = 10,
    LightCyan   = 11,
    LightRed    = 12,
    Pink        = 13,
    Yellow      = 14,
    White       = 15
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Colors, background: Colors) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_char: u8,
    color_code: ColorCode
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

pub struct Writer {
    column_pos: usize,
    row_pos: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer
}

impl Writer {
    /**
     * Writes the given text to the screen in VGA-compatible Text Mode via memory-mapped i/o.
     * The writer will always write to the last line and shift lines up when a line is full (or \n is encountered).
     * The writer will print only ASCII and Code Page 437 characters.
     * Strings in Rust are UTF-8 by default, and might contain bytes that are unprintable.
     * In this case, it will print the ■ character instead.
    */
    pub fn write_string(&mut self, text: &str) {
        for byte in text.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            byte => {
                if self.column_pos >= BUFFER_WIDTH || self.row_pos >= BUFFER_HEIGHT {
                    self.newline();
                }

            let row = self.row_pos;
            let col = self.column_pos;
            let color_code = self.color_code;

            self.buffer.chars[row][col].write(ScreenChar {
                ascii_char: byte,
                color_code
            });
            self.column_pos += 1;
            }
        }
    }

    fn newline(&mut self) {
        if self.row_pos >= BUFFER_HEIGHT {
            self.row_pos = 1;
        }

        for row in self.row_pos..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row -1][col].write(character);
            }
        }
        self.clear_row(self.row_pos);
        self.column_pos = 0;
        self.row_pos = self.row_pos + 1;
    }

    /**
     * Replaces all the characters in the given row with a space character.
     */
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_char: b' ',
            color_code: self.color_code
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.write_string(text);
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
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}