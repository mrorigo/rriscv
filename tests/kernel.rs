use elfloader::ElfBinary;
use rriscv::{
    cpu, elf,
    memory::{Memory, MemoryAccessWidth, MemoryOperations},
};

#[test]
fn main() {
    use std::fs;

    let vbase: u64 = 0x8000_0000;
    let memory = &mut Memory::create(vbase, 138560);

    let binary_blob = fs::read("tests/xv6/kernel").expect("Can't read kernel binary");
    //let binary_blob = fs::read("./test").expect("Can't read kernel binary");
    let binary = ElfBinary::new(binary_blob.as_slice()).expect("Got proper ELF file");
    let mut loader = elf::Loader::create(vbase, memory);
    binary.load(&mut loader).expect("Can't load the binary?");

    // Start HART #0
    let mut cpu = cpu::Core::create(0x0, memory);
    cpu.set_pc(vbase);

    loop {
        cpu.cycle();
        println!("---");
    }
    // cpu.cycle();
    // cpu.cycle();
}
