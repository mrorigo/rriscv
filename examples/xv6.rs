use std::collections::HashMap;

use elfloader::{ElfBinary, VAddr};
use rriscv::{
    cpu::{self},
    elf,
    memory::MemoryOperations,
    mmu::{MemoryRange, MMU},
    pipeline::Stage,
};

fn main() {
    println!("R-RISCV Emulator: Initializing for XV6 kernel");
    use std::fs;

    let vbase: u64 = 0x8000_0000;

    let mmu = MMU::create();
    println!("DTB Device Table:");
    mmu.dump_device_table();

    let mmu = &mut MMU::create();

    let fs_contents = fs::read("examples/xv6/fs.img").expect("Can't read xv6 fs images");
    println!(
        "Virtio filesystem initialized ({} bytes)",
        fs_contents.len()
    );
    mmu.virtio_mut().load_fs(fs_contents);

    let binary_blob = fs::read("examples/xv6/kernel.min").expect("Can't read xv6 kernel binary");
    let binary = ElfBinary::new(binary_blob.as_slice()).expect("Got proper ELF file");
    let mut loader = elf::Loader::create(vbase, mmu);
    binary.load(&mut loader).expect("Can't load the binary?");

    let mut symbols: HashMap<u64, &str> = HashMap::new();
    binary
        .for_each_symbol(|sym| {
            if sym.name() != 0 {
                let sym_name = binary.symbol_name(sym);
                symbols.insert(sym.value(), sym_name);
            }
        })
        .expect("No symbols in ELF file");

    // Start HART #0
    let mut cpu = cpu::Core::create(0x0);
    cpu.set_pc(vbase);

    for sym in symbols.iter() {
        cpu.add_symbol(*sym.0 as VAddr, sym.1.to_string());
    }

    mmu.protect_range(MemoryRange {
        name: "code",
        start: 0x80000000,
        end: 0x80004000,
    });

    loop {
        cpu.cycle(mmu);
        mmu.tick();
        //println!("xv6: ticked");
        match cpu.stage {
            Stage::FETCH => {
                // let key = cpu.pc() as u64;
                // match symbols.get(&key) {
                //     None => {}
                //     Some(symbol) => {
                //         println!(
                //             "\n==== xv6: Executing from symbol {:?} @ {:#x?} ====\n",
                //             symbol, key
                //         )
                //     }
                // }

                //let instret = cpu.read_csr(CSRRegister::instret);
                // println!(
                //     "--- minstret: {}  instret: {}",
                //     cpu.read_csr(CSRRegister::minstret),
                //     instret
                // )
            }
            _ => {}
        }
    }
    // cpu.cycle();
    // cpu.cycle();
}
