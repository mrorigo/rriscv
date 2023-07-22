use elfloader::ElfBinary;
use rriscv::{
    cpu::{self, CSRRegister},
    elf,
    mmu::MMU,
    pipeline::Stage,
};

#[test]
fn main() {
    use std::fs;

    let vbase: u64 = 0x8000_0000;
    let clint_base = 0x2000000;
    let plic_base = 0x0c000000;

    let mmu = MMU::create();
    mmu.dump_device_table();

    let memory = &mut MMU::create();
    // memory.add_segment(vbase, 138560);
    // memory.add_segment(clint_base, 0xc000);
    // memory.add_segment(plic_base, 0x200000 + 0x2000 * 8);

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
        match cpu.stage {
            Stage::FETCH => {
                println!("--- minstret: {}", cpu.read_csr(CSRRegister::minstret))
            }
            _ => {}
        }
    }
    // cpu.cycle();
    // cpu.cycle();
}
