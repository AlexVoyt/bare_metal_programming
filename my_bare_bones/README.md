# What is this
This is my take on [OSdev bare bones tutorial]("https://wiki.osdev.org/Bare_Bones"). However, since I didn't want to copy entire tutorial, I wanted to make some things differently:
- I decided to write kernel in Rust instead of C. What's better way to learn language than writing OS?
I think I know C decently enough (although I can't recite all UBs from memory, I must admit),
and Rust always intrigued me with its high level abstractions and ability to target bare metal.
- Osdev uses grub to load kernel, and I decided to write my own (minimalistic) bootloader since I don't want to rely on third party dependencies
(this is learning project, after all).

# How this works
Build process is **heavily** inspired by [gamozolabs](https://github.com/gamozolabs) [chocolate_milk](https://github.com/gamozolabs/chocolate_milk). Basically, we have `#![no_std]` Rust program (kernel), which we compile and statically link as an ELF object. Then we extract all LOAD segments from ELF and directly append them to our little MBR assembly.
Again, I didn't want to copy entire thing so here are few changes:
- Gamozolabs uses PE format for kernel, I use ELF format.
- Gamozolabs uses PXE bootloader, I use MBR bootloader.

# Build system
We use Rust to build Rust: as stated above, our build system flattens kernel ELF, appends it to MBR assembly and runs `nasm` to assemble final binary.
Running `cargo run` should create `build` directory with build artifacts, there will be `mbr.bin` binary which is final kernel image.
If you have QEMU installed, you should be able to run `qemu-system-i386 -drive format=raw,file=mbr.bin` and see greetings from Rust on your screen.

# Bootloader
Out bootloader is simple `mbr.asm` file located in `kernel/src`. It sets up some stuff needed to go for 32-bit mode (A20 line, GDT) and then uses BIOS
`int=0x13, ah=0x42` function to load kernel. Additional information about bootloaders can be at OSDev:
[Bootloader](https://wiki.osdev.org/Bootloader) and [Rolling Your Own Bootloader](https://wiki.osdev.org/Rolling_Your_Own_Bootloader).

# Kernel
Kernel works in 32-bit mode. For now we are not able to do much - just printing to VGA text mode buffer.

# Things to add and improve
- Build system: Add checks that user system has all build dependencies (nightly rust toolchain, ld.lld and nasm).
- Bootloader: MBR assembly file does not have properly setup partition table entries. This is fine for QEMU, but 2 laptops I have both refused to load kernel without these entries, should fix it.
- Bootloader: Bootloader should perform memory detection and pass memory map to kernel.
- Kernel: I really want to do some PCI stuff - probably should start with some NIC drivers (OSDev recommends e1000), maybe NVME after?
- Kernel: For now we are running without interrupts - and af
- Kernel: At some point I want to do virtual memory support.
- Kernel: Once we have virtual memory up and running, how hard can it be to load statically built ELF's and run them?
- Kernel: For now we are running in 32-bit mode - time for 64-bit?
- ...and more and more...
