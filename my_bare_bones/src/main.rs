use std::{error::Error, process::Command, path::Path};

use elf_parser::ELF32Parser;

/// Returns flattend image
fn flatten_elf<P: AsRef<Path>>(path: P) -> Option<Vec<u8>> {
    let bytes = std::fs::read(path).ok()?;
    let parser = ELF32Parser::new_from_bytes(bytes.as_slice())?;

    // std::println!("Entry point: {:x}", parser.entry);
    // std::println!("ph_num: {:x}", parser.ph_num);

    let mut image = Vec::new();
    for (_header_idx, header) in parser.program_headers().enumerate() {
        // std::println!("Segment {header_idx}: type: {}, p_addr: 0x{:x}, v_addr: 0x{:x}, size: 0x{:x}",
        //               header.header_type, header.p_addr, header.v_addr, header.bytes.len());

        // Only process LOAD segments
        if header.header_type != 1 {
            continue;
        }

        // Add padding to respect segment alignment
        let alignment = header.alignment as usize;
        let rem = image.len() % alignment;
        if rem != 0 {
            image.extend_from_slice(vec![0; alignment - rem].as_slice());
            assert!(image.len() % alignment == 0)
        }
        image.extend_from_slice(header.bytes);
    }

    Some(image)
}

fn main() -> Result<(), Box<dyn Error>>{
    // Create build directory
    std::fs::create_dir_all("build")?;
    let build_dir = Path::new("build").canonicalize()?;

    // Build kernel
    let kernel_dir = Path::new("kernel");

    if !Command::new("cargo")
        .current_dir(kernel_dir)
        .args(["build", "-r", "--target-dir", build_dir.to_str().unwrap()])
        .status()?.success() {
            return Err("Could not build the kernel".into())
        };

    // Flatten kernel
    let kernel_elf_path = build_dir
        .join("i686-bare_metal_target")
        .join("release")
        .join("kernel");
    let flattened = flatten_elf(kernel_elf_path).ok_or("Could not flatten elf")?;
    let sectors = (flattened.len() + 511) / 512;

    std::println!("Kernel size: {} bytes, sectors: {}", flattened.len(), sectors);
    std::fs::write(build_dir.join("kernel.flat"), flattened)?;

    if !Command::new("nasm")
        .args([&format!("-DSECTORS_TO_READ={}", sectors),
              "-o", build_dir.join("mbr.bin").to_str().unwrap(),
              Path::new("kernel").join("src").join("mbr.asm").to_str().unwrap()])
        .status()?.success() {
            return Err("Could not build mbr".into())
        }

    Ok(())
}
