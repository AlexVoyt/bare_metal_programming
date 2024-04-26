#![no_std]
#![no_main]

use core::{isize, panic::PanicInfo};
use core::fmt::Write;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // TODO: implement panic (Print to vga? How can we get to vga structure from here?
    // Put vga to UnsafeCell and make it global?)
    loop {}
}

struct VGA {
    row: u8,
    col: u8,
    attribute_byte: u16,
}

impl core::fmt::Write for VGA {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        if s.is_ascii() {
            for char in s.bytes() {
                self.write_char(char);
            }
            return Ok(())
        } else {
            return Err(core::fmt::Error)
        }
    }
}

impl VGA {
    const fn new() -> Self {
        VGA {
            row: 0,
            col: 0,
            attribute_byte: 0x0F00,
        }
    }

    fn write_char(&mut self, char: u8) {
        match char {
            b'\n' => {
                self.col = 0;
                self.row += 1;
            }
            _ => {
                let vga_framebuffer = 0xb8000 as *mut u16;
                let val = self.attribute_byte + char as u16;
                let offset: isize = self.col as isize + (self.row as isize) * 80;
                // TODO: validate offset
                unsafe {
                    core::ptr::write(vga_framebuffer.offset(offset), val);
                }
                self.col += 1;
                if self.col == 80 {
                    self.row += 1;
                    self.col = 0;
                    // TODO: Scroll on row exceeding 25
                }
            }
        }
    }

    fn clear_screen(&self) {
        let vga_framebuffer = 0xb8000 as *mut u16;
        for offset in 0..25*80 {
            unsafe {
                core::ptr::write(vga_framebuffer.offset(offset), 0x0000);
            }
        }
    }

}

#[no_mangle]
#[link_section = ".kernel_entry"]
fn kernel_entry() -> ! {
    let mut vga = VGA::new();
    vga.clear_screen();
    let mut local_var = 100_u32;
    write!(&mut vga, "Hello from Rust\n").unwrap();
    write!(&mut vga, "Value of local_var: {local_var}, address: 0x{:p}\n", &local_var).unwrap();
    local_var += 123;
    write!(&mut vga, "Value of local_var: {local_var}, address: 0x{:p}\n", &local_var).unwrap();
    loop {}
}

/* `core` library dependencies */
#[no_mangle]
pub unsafe extern fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let c = c as u32 as u8;
    for i in 0..n {
        s.offset(i as isize).write(c);
    }
    s
}

#[no_mangle]
pub unsafe extern fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if dest < src as *mut u8 {
        for i in 0..n as isize {
            dest.offset(i).write(src.offset(i).read());
        }
    } else {
        for i in n..0 {
            let i = i as isize;
            dest.offset(i).write(src.offset(i).read());
        }
    }
    dest
}

#[no_mangle]
pub unsafe extern fn memcpy(dest: *mut u8, src: *mut u8, n: usize) -> *mut u8 {
    memmove(dest, src, n)
}

#[no_mangle]
pub unsafe extern fn memcmp(dest: *const u8, src: *const u8, n: usize) -> i32 {
    for i in 0..n as isize {
        let c1 = dest.offset(i).read();
        let c2 = src.offset(i).read();
        if c1 < c2 {
            return -1
        } else if c1 > c2 {
            return 1
        }
    }
    0
}
