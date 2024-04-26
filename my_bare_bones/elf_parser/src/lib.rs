#![no_std]

/// Definition of ELF32 header, used with core::mem::offset_of!() macro
#[repr(C)]
struct ELF32Header {
    e_ident:     [u8; 16],
    e_type:      u16,
    e_machine:   u16,
    e_version:   u32,
    e_entry:     u32,
    e_phoff:     u32,
    e_shoff:     u32,
    e_flags:     u32,
    e_ehsize:    u16,
    e_phentsize: u16,
    e_phnum:     u16,
    e_shentsize: u16,
    e_shnum:     u16,
    e_shstrndx:  u16,
}

/// ELF32 file with some metadata
pub struct ELF32Parser<'a> {
    /// Raw ELF32 file
    bytes: &'a[u8],

    /// Entry point of program
    pub entry: u32,

    /// Number of program headers
    pub ph_num: u16,

    /// Offset into file where headers are located
    ph_offset: usize,

    /// Size of program header entry
    ph_entry_size: u16,
}

impl<'a> ELF32Parser<'a> {
    /// Returns validated ELF32_Parser struct
    pub fn new_from_bytes(bytes: &'a [u8]) -> Option<Self> {
        // Check ELF signature
        if bytes.get(0..4) != Some(&[127, 69, 76, 70]) {
            return None;
        }

        // Ensure 32-bit elf
        if bytes.get(4) != Some(&1) {
            return None;
        }

        // Ensure elf is little endian
        if bytes.get(5) != Some(&1) {
            return None;
        }

        // Get entry point
        let entry = u32::from_le_bytes(bytes.get(
                core::mem::offset_of!(ELF32Header, e_entry)..
                core::mem::offset_of!(ELF32Header, e_entry) + 4)?.try_into().ok()?);

        // Get program header table offset
        let ph_offset = u32::from_le_bytes(bytes.get(
                core::mem::offset_of!(ELF32Header, e_phoff)..
                core::mem::offset_of!(ELF32Header, e_phoff) + 4)?.try_into().ok()?) as usize;
        // println!("PH offset: {}", ph_offset);

        let ph_entry_size = u16::from_le_bytes(bytes.get(
                core::mem::offset_of!(ELF32Header, e_phentsize)..
                core::mem::offset_of!(ELF32Header, e_phentsize) + 2)?.try_into().ok()?);
        // println!("PH ent size: {}", ph_entry_size);

        let ph_num = u16::from_le_bytes(bytes.get(
                core::mem::offset_of!(ELF32Header, e_phnum)..
                core::mem::offset_of!(ELF32Header, e_phnum) + 2)?.try_into().ok()?);
        // println!("PH num: {}", ph_num);

        // TODO: validate file
        /*
        for segment_idx in 0..ph_num as u32 {
            // std::println!("Segment {segment_idx}: type: {segment_type}, p_addr: {p_addr:x}, v_addr: {v_addr:x}");
        }
        */

        Some(ELF32Parser {
            bytes,
            entry,
            ph_num,
            ph_offset,
            ph_entry_size,
        })
    }

    /// Returns a ProgramHeaderIterator
    pub fn program_headers(&'a self) -> ProgramHeaderIterator<'a> {
        ProgramHeaderIterator {
            parser: self,
            program_header_cur_index: 0,
            program_header_one_past_last_index: self.ph_num as usize,
        }
    }
}

/// Program header
pub struct ProgramHeader<'a> {
    /// Program header type
    pub header_type: u32,

    /// Raw bytes of segment
    pub bytes: &'a[u8],

    /// Virtual address where segment should be loaded
    pub v_addr: u32,

    /// Physical address where segment should be loaded
    pub p_addr: u32,

    /// File size of segment
    pub mem_size: u32,

    /// Segment flags
    pub flags: u32,

    /// Segment alignment
    pub alignment: u32,
}

/// Program header iterator
pub struct ProgramHeaderIterator<'a> {
    /// A reference to an ELF file from which we created this iterator
    parser: &'a ELF32Parser<'a>,

    /// Index of program header that will be returned upon calling 'next()' on iterator
    program_header_cur_index: usize,

    /// One past last index of program header
    program_header_one_past_last_index: usize,
}

impl<'a> Iterator for ProgramHeaderIterator<'a> {
    type Item = ProgramHeader<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.program_header_cur_index >= self.program_header_one_past_last_index {
            None
        } else {
            let bytes = self.parser.bytes;
            let program_header_entry_size = self.parser.ph_entry_size as usize;
            let header_offset = self.parser.ph_offset + self.program_header_cur_index * program_header_entry_size;
            let header_type = u32::from_le_bytes(bytes[
                header_offset + 0x0..header_offset + 0x4].try_into().ok()?);
            let offset = u32::from_le_bytes(bytes[
                header_offset + 0x4..header_offset + 0x8].try_into().ok()?) as usize;
            let v_addr = u32::from_le_bytes(bytes[
                header_offset + 0x8..header_offset + 0xC].try_into().ok()?);
            let p_addr = u32::from_le_bytes(bytes[
                header_offset + 0xC..header_offset + 0x10].try_into().ok()?);
            let file_size = u32::from_le_bytes(bytes[
                header_offset + 0x10..header_offset + 0x14].try_into().ok()?) as usize;
            let mem_size = u32::from_le_bytes(bytes[
                header_offset + 0x14..header_offset + 0x18].try_into().ok()?);
            let flags = u32::from_le_bytes(bytes[
                header_offset + 0x18..header_offset + 0x1C].try_into().ok()?);
            let alignment = u32::from_le_bytes(bytes[
                header_offset + 0x1C..header_offset + 0x20].try_into().ok()?);
            self.program_header_cur_index += 1;
            Some(ProgramHeader {
                header_type,
                bytes: bytes.get(offset..offset.checked_add(file_size)?)?,
                v_addr,
                p_addr,
                flags,
                alignment,
                mem_size,
            })
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::ELF32Parser;
    extern crate std;

    #[test]
    fn create_new_parser_and_iterate() {
        let bytes = std::fs::read("../build/i686-bare_metal_target/release/kernel").expect("Failed to read elf");
        let parser = ELF32Parser::new_from_bytes(bytes.as_slice()).unwrap();

        std::println!("Entry point: {:x}", parser.entry);
        std::println!("ph_num: {:x}", parser.ph_num);

        for (header_idx, header) in parser.program_headers().enumerate() {
            std::println!("Segment {header_idx}: type: {}, p_addr: {:x}, v_addr: {:x}",
                          header.header_type, header.p_addr, header.v_addr);
        }
    }
}
