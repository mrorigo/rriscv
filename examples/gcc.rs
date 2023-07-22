// /usr/local/homebrew/bin/riscv64-unknown-elf-gcc

use std::collections::HashMap;

use elfloader::{ElfBinary, VAddr};
use rriscv::{cpu, elf, memory::MemoryOperations, mmu::MMU, pipeline::Stage};

fn case(name: &str) -> bool {
    use std::fs;

    let vbase: u64 = 0x8000_0000;
    let mmu = &mut MMU::create();

    let binary_blob = fs::read(name).expect("Can't read binary");
    let binary = ElfBinary::new(binary_blob.as_slice()).expect("Got proper ELF file");
    let mut loader = elf::Loader::create(vbase, mmu);
    binary
        .load(&mut loader)
        .expect("Can't load the binary to memory?");

    let mut symbols: HashMap<u64, &str> = HashMap::new();

    binary
        .for_each_symbol(|sym| {
            if sym.name() != 0 {
                let sym_name = binary.symbol_name(sym);
                //println!("Adding symbol {:?} @ {:#x?}", sym_name, sym.value());
                symbols.insert(sym.value(), sym_name);
            }
        })
        .expect("No symbols in ELF file");

    // Start HART #0
    let mut cpu = cpu::Core::create(0x0);
    cpu.reset(vbase);

    let mut tohost_addr: u64 = 0;

    for sym in symbols.iter() {
        match sym.1.to_string().as_str() {
            "tohost" => tohost_addr = *sym.0,
            _ => {}
        }
        cpu.add_symbol(*sym.0 as VAddr, sym.1.to_string());
    }

    let mut num_tests = 0;
    let mut curr_test: Option<String> = None;

    let mut ticks = 0;
    loop {
        ticks = ticks + 1;
        if ticks > 2000000 {
            return false;
        }

        cpu.cycle(mmu);
        cpu.write_csr(
            cpu::CSRRegister::mip,
            mmu.tick(cpu.read_csr(cpu::CSRRegister::mip)),
        );
        //        println!("case: ticked");

        if tohost_addr != 0 {
            let tohost = mmu.read32(tohost_addr).unwrap();
            mmu.write32(tohost_addr, 0x0);

            if tohost != 0 {
                let payload = (tohost << 16 >> 16);
                if payload & 1 != 0 {
                    println!("tohost value: {:#x?}", tohost);
                    return false;
                }
                //return false;
            }
        }
        match cpu.stage {
            Stage::FETCH => {
                let key = cpu.pc() as u64;
                match symbols.get(&key) {
                    None => {}
                    Some(symbol) => match *symbol {
                        "fail" => {
                            println!("CASE: {}:\tFAIL ({:#?}) ", name, curr_test);
                            return false;
                        }
                        "pass" => {
                            println!("CASE: {}:\tOK ({:#} tests)", name, num_tests);
                            return true;
                        }
                        _ => {
                            if symbol.starts_with("test_") {
                                num_tests = num_tests + 1;
                                curr_test = Some(symbol.clone().to_string());
                                println!(" - TEST: {:?} ", symbol);
                            }
                        }
                    },
                }
            }

            _ => {}
        }
    }
    // cpu.cycle();
    // cpu.cycle();
}

fn main() {
    case("/usr/local/homebrew/bin/riscv64-unknown-elf-gcc");
}
