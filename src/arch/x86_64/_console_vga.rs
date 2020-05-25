use core::fmt;
use core::fmt::Write;
use core::iter::Skip;
use core::str::Bytes;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
    /// A global `Writer` instance that can be used for printing to the VGA text buffer.
    pub static ref CONSOLE_DISPLAY_GLOBAL: VgaImmutableWriter = VgaImmutableWriter {
     vga_writer: Mutex::new(VgaWriter {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    })};
}

/// The standard color palette in VGA text mode.
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
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// A combination of a foreground and a background color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    /// Create a new `ColorCode` with the given foreground and background colors.
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// A screen character in the VGA text buffer, consisting of an ASCII character and a `ColorCode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    /// The character on the screen.
    ascii_character: u8,

    /// The color with which the character should be displayed
    /// on the screen.
    color_code: ColorCode,
}

/// The height of the text buffer (normally 25 lines).
const BUFFER_HEIGHT: usize = 25;
/// The width of the text buffer (normally 80 columns).
const BUFFER_WIDTH: usize = 80;

/// A structure representing the VGA text buffer.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// A writer type that allows writing ASCII bytes and strings to an underlying `Buffer`.
///
/// Wraps lines at `BUFFER_WIDTH`. Supports newline characters and implements the
/// `Write` trait.
pub struct VgaWriter {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl VgaWriter {
    /// Writes an ASCII byte to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// Writes the given ASCII string to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character. Does **not**
    /// support strings with non-ASCII characters, since they can't be printed in the VGA text
    /// mode.
    fn write_string(&mut self, s: &str) {
        // Get only the last few relevant characters.
        let count = s.len();
        const MAX_CHAR_COUNT: usize = BUFFER_WIDTH * BUFFER_HEIGHT;
        let bytes: Skip<Bytes> = if count >= MAX_CHAR_COUNT {
            s.bytes().skip(count - MAX_CHAR_COUNT)
        } else {
            s.bytes().skip(0)
        };

        for byte in bytes {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Shifts all lines one line up and clears the last row.
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// Clears a row by overwriting it with blank characters.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    /// Clear all the rows by calling `clear_row` on all the rows.
    fn clear_all_rows(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
    }

    /// Write the arguments with color.
    fn write_args_with_color(&mut self, color_code: ColorCode, args: fmt::Arguments) {
        let initial_color = self.color_code;
        self.color_code = color_code;
        self.write_fmt(args).unwrap();
        self.color_code = initial_color;
    }
}

impl fmt::Write for VgaWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Immutable wrapper over VgaWriter to make it thread-safe.
pub struct VgaImmutableWriter {
    /// The mutex to write to vga_writer.
    vga_writer: Mutex<VgaWriter>,
}

#[allow(dead_code)]
impl VgaImmutableWriter {
    pub fn concurrent_write_color_fmt(&self, color: ColorCode, args: fmt::Arguments) {
        use x86_64::instructions::interrupts;

        interrupts::without_interrupts(|| {
            self.vga_writer.lock().write_args_with_color(color, args);
        });
    }

    /// Clear the console and reset it to the initial state.
    fn clear_console(&self) {
        self.vga_writer.lock().clear_all_rows();
    }

    ///  Write function to be implemented on the instance.
    ///
    /// # Arguments
    /// * `args` - The arguments to print.
    fn concurrent_write_fmt(&self, args: fmt::Arguments) {
        use x86_64::instructions::interrupts;

        interrupts::without_interrupts(|| {
            self.vga_writer.lock().write_fmt(args).unwrap();
        });
    }
}
