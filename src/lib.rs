// use core::str::FromStr;
// use std::{fmt::Display, mem, sync::atomic::AtomicI8};

//use elfloader::ElfBinary;
#[macro_use]
extern crate num_derive;

// use crate::memory::{Memory, MemoryOperations};

pub mod cpu;
pub mod decoder;
pub mod elf;
pub mod instruction_format;
pub mod memory;
pub mod opcodes;
pub mod pipeline;

// fn main() {
//     use std::fs;

//     let vbase: u64 = 0x8000_0000;
//     let mut memory = Memory::create(vbase, 138560);

//     let binary_blob = fs::read("kernel").expect("Can't read binary");
//     let binary = ElfBinary::new(binary_blob.as_slice()).expect("Got proper ELF file");
//     let mut loader = elf::Loader::create(vbase, &mut memory);
//     binary.load(&mut loader).expect("Can't load the binary?");

//     for i in (0..16).step_by(1) {
//         let u = memory.read_single((vbase + i) as usize, memory::MemoryAccessWidth::BYTE);
//         println!("{}: {:#x?}", i, u.unwrap())
//     }
//     println!("---");

//     // Start HART #0
//     let mut cpu = cpu::Core::create(0x0, &memory);
//     cpu.set_pc(vbase);

//     loop {
//         cpu.cycle();
//         println!("---");
//     }
//     // cpu.cycle();
//     // cpu.cycle();
// }
