use std::{collections::HashMap, time::Instant};

use elfloader::{ElfBinary, VAddr};
use rriscv::{
    cpu::{self, CSRRegister, CYCLES_PER_INSTRUCTION},
    elf,
    memory::MemoryOperations,
    mmu::MMU,
    pipeline::Stage,
};

const CASES: [&str; 69] = [
    // //"../../git/riscv-tests/isa/rv64mi-p-access",
    // //"../../git/riscv-tests/isa/rv64mi-p-breakpoint",
    // //"../../git/riscv-tests/isa/rv64mi-p-csr",
    // //"../../git/riscv-tests/isa/rv64mi-p-illegal",
    // // "../../git/riscv-tests/isa/rv64mi-p-ld-misaligned",
    // // "../../git/riscv-tests/isa/rv64mi-p-lh-misaligned",
    // // "../../git/riscv-tests/isa/rv64mi-p-lw-misaligned",
    // // "../../git/riscv-tests/isa/rv64mi-p-ma_addr",
    // // "../../git/riscv-tests/isa/rv64mi-p-ma_fetch",
    // // "../../git/riscv-tests/isa/rv64mi-p-mcsr",
    // // "../../git/riscv-tests/isa/rv64mi-p-sbreak",
    //    "../../git/riscv-tests/isa/rv64mi-p-scall",
    //    "../../git/riscv-tests/isa/rv64mi-p-csr",
    // // "../../git/riscv-tests/isa/rv64mi-p-sd-misaligned",
    // // "../../git/riscv-tests/isa/rv64mi-p-sh-misaligned",
    // // "../../git/riscv-tests/isa/rv64mi-p-sw-misaligned",
    "../../git/riscv-tests/isa/rv64mi-p-zicntr",
    // // "../../git/riscv-tests/isa/rv64si-p-dirty",
    // // "../../git/riscv-tests/isa/rv64si-p-icache-alias",
    // // "../../git/riscv-tests/isa/rv64si-p-ma_fetch",
    "../../git/riscv-tests/isa/rv64si-p-sbreak",
    "../../git/riscv-tests/isa/rv64si-p-scall",
    // // "../../git/riscv-tests/isa/rv64si-p-wfi",
    // // "../../git/riscv-tests/isa/rv64ssvnapot-p-napot",
    // // "../../git/riscv-tests/isa/rv64ua-p-amoadd_d",
    // "../../git/riscv-tests/isa/rv64ua-p-amoadd_w",
    // // "../../git/riscv-tests/isa/rv64ua-p-amoand_d",
    // // "../../git/riscv-tests/isa/rv64ua-p-amoand_w",
    // // "../../git/riscv-tests/isa/rv64ua-p-amomax_d",
    // // "../../git/riscv-tests/isa/rv64ua-p-amomax_w",
    // // "../../git/riscv-tests/isa/rv64ua-p-amomaxu_d",
    // // "../../git/riscv-tests/isa/rv64ua-p-amomaxu_w",
    // // "../../git/riscv-tests/isa/rv64ua-p-amomin_d",
    // // "../../git/riscv-tests/isa/rv64ua-p-amomin_w",
    // // "../../git/riscv-tests/isa/rv64ua-p-amominu_d",
    // // "../../git/riscv-tests/isa/rv64ua-p-amominu_w",
    // // "../../git/riscv-tests/isa/rv64ua-p-amoor_d",
    // // "../../git/riscv-tests/isa/rv64ua-p-amoor_w",
    // // "../../git/riscv-tests/isa/rv64ua-p-amoswap_d",
    "../../git/riscv-tests/isa/rv64ua-p-amoswap_w",
    // // "../../git/riscv-tests/isa/rv64ua-p-amoxor_d",
    // // "../../git/riscv-tests/isa/rv64ua-p-amoxor_w",
    // // "../../git/riscv-tests/isa/rv64ua-p-lrsc",
    "../../git/riscv-tests/isa/rv64uc-p-rvc",
    // // "../../git/riscv-tests/isa/rv64ud-p-fadd",
    // // "../../git/riscv-tests/isa/rv64ud-p-fclass",
    // // "../../git/riscv-tests/isa/rv64ud-p-fcmp",
    // // "../../git/riscv-tests/isa/rv64ud-p-fcvt",
    // // "../../git/riscv-tests/isa/rv64ud-p-fcvt_w",
    // // "../../git/riscv-tests/isa/rv64ud-p-fdiv",
    // // "../../git/riscv-tests/isa/rv64ud-p-fmadd",
    // // "../../git/riscv-tests/isa/rv64ud-p-fmin",
    // // "../../git/riscv-tests/isa/rv64ud-p-ldst",
    // // "../../git/riscv-tests/isa/rv64ud-p-move",
    // // "../../git/riscv-tests/isa/rv64ud-p-recoding",
    // // "../../git/riscv-tests/isa/rv64ud-p-structural",
    // // "../../git/riscv-tests/isa/rv64uf-p-fadd",
    // // "../../git/riscv-tests/isa/rv64uf-p-fclass",
    // // "../../git/riscv-tests/isa/rv64uf-p-fcmp",
    // // "../../git/riscv-tests/isa/rv64uf-p-fcvt",
    // // "../../git/riscv-tests/isa/rv64uf-p-fcvt_w",
    // // "../../git/riscv-tests/isa/rv64uf-p-fdiv",
    // // "../../git/riscv-tests/isa/rv64uf-p-fmadd",
    // // "../../git/riscv-tests/isa/rv64uf-p-fmin",
    // // "../../git/riscv-tests/isa/rv64uf-p-ldst",
    // // "../../git/riscv-tests/isa/rv64uf-p-move",
    // // "../../git/riscv-tests/isa/rv64uf-p-recoding",
    "../../git/riscv-tests/isa/rv64ui-p-add",
    "../../git/riscv-tests/isa/rv64ui-p-addi",
    "../../git/riscv-tests/isa/rv64ui-p-addiw",
    "../../git/riscv-tests/isa/rv64ui-p-addw",
    "../../git/riscv-tests/isa/rv64ui-p-and",
    "../../git/riscv-tests/isa/rv64ui-p-andi",
    "../../git/riscv-tests/isa/rv64ui-p-sub",
    "../../git/riscv-tests/isa/rv64ui-p-subw",
    "../../git/riscv-tests/isa/rv64ui-p-auipc",
    "../../git/riscv-tests/isa/rv64ui-p-beq",
    "../../git/riscv-tests/isa/rv64ui-p-bge",
    "../../git/riscv-tests/isa/rv64ui-p-bgeu",
    "../../git/riscv-tests/isa/rv64ui-p-blt",
    "../../git/riscv-tests/isa/rv64ui-p-bltu",
    "../../git/riscv-tests/isa/rv64ui-p-bne",
    "../../git/riscv-tests/isa/rv64ui-p-fence_i",
    "../../git/riscv-tests/isa/rv64ui-p-jal",
    "../../git/riscv-tests/isa/rv64ui-p-jalr",
    "../../git/riscv-tests/isa/rv64ui-p-lb",
    "../../git/riscv-tests/isa/rv64ui-p-lbu",
    "../../git/riscv-tests/isa/rv64ui-p-ld",
    "../../git/riscv-tests/isa/rv64ui-p-lh",
    "../../git/riscv-tests/isa/rv64ui-p-lhu",
    "../../git/riscv-tests/isa/rv64ui-p-lui",
    "../../git/riscv-tests/isa/rv64ui-p-lw",
    "../../git/riscv-tests/isa/rv64ui-p-lwu",
    // "../../git/riscv-tests/isa/rv64ui-p-ma_data",
    "../../git/riscv-tests/isa/rv64ui-p-or",
    "../../git/riscv-tests/isa/rv64ui-p-ori",
    "../../git/riscv-tests/isa/rv64ui-p-sb",
    "../../git/riscv-tests/isa/rv64ui-p-sd",
    "../../git/riscv-tests/isa/rv64ui-p-sh",
    "../../git/riscv-tests/isa/rv64ui-p-sw",
    "../../git/riscv-tests/isa/rv64ui-p-simple",
    "../../git/riscv-tests/isa/rv64ui-p-sll",
    "../../git/riscv-tests/isa/rv64ui-p-slli",
    "../../git/riscv-tests/isa/rv64ui-p-slliw",
    "../../git/riscv-tests/isa/rv64ui-p-sllw",
    "../../git/riscv-tests/isa/rv64ui-p-slt",
    "../../git/riscv-tests/isa/rv64ui-p-slti",
    "../../git/riscv-tests/isa/rv64ui-p-sltiu",
    "../../git/riscv-tests/isa/rv64ui-p-sltu",
    "../../git/riscv-tests/isa/rv64ui-p-sra",
    "../../git/riscv-tests/isa/rv64ui-p-srai",
    "../../git/riscv-tests/isa/rv64ui-p-sraiw",
    "../../git/riscv-tests/isa/rv64ui-p-sraw",
    "../../git/riscv-tests/isa/rv64ui-p-srl",
    "../../git/riscv-tests/isa/rv64ui-p-srli",
    "../../git/riscv-tests/isa/rv64um-p-mul",
    "../../git/riscv-tests/isa/rv64um-p-mulh",
    "../../git/riscv-tests/isa/rv64ui-p-xor",
    "../../git/riscv-tests/isa/rv64ui-p-xori",
    "../../git/riscv-tests/isa/rv64ui-p-srliw",
    "../../git/riscv-tests/isa/rv64ui-p-srlw",
    "../../git/riscv-tests/isa/rv64um-p-remuw",
    "../../git/riscv-tests/isa/rv64um-p-mulw",
    "../../git/riscv-tests/isa/rv64um-p-div",
    "../../git/riscv-tests/isa/rv64um-p-divu",
    "../../git/riscv-tests/isa/rv64um-p-divuw",
    "../../git/riscv-tests/isa/rv64um-p-divw",
    "../../git/riscv-tests/isa/rv64um-p-mulhsu",
    "../../git/riscv-tests/isa/rv64um-p-mulhu",
    "../../git/riscv-tests/isa/rv64um-p-rem",
    "../../git/riscv-tests/isa/rv64um-p-remu",
    "../../git/riscv-tests/isa/rv64um-p-remw",
];

//];

#[test]
fn all_cases() {
    let mut failed = false;
    for argument in CASES.iter() {
        println!("========== {:?} ==========", argument);
        failed |= !case(&argument.to_string());
        println!("");
    }
    assert!(!failed);
}

#[test]
fn benchmark() {
    let towers = "./tests/benchmark/towers.riscv";
    let qsort = "./tests/benchmark/qsort.riscv";

    {
        let start = Instant::now();
        !bench(towers);
        let duration = start.elapsed();
        println!("Towers of Hanoi took {:#?}", duration);
    }
    {
        let start = Instant::now();
        !bench(qsort);
        let duration = start.elapsed();
        println!("Qsort took {:#?}", duration);
    }
    assert!(false, "So?")
}

// #[test]
// pub fn instruction() {
//     let vbase: u64 = 0x8000_0000;
//     let mmu = &mut MMU::create();
//     mmu.write32(vbase, 0x12000073); // sfence.vma
//                                     //    mmu.write_32(vbase + 4, 0x302064c5); // c.lui	x9,0x21

//     // Start HART #0
//     let mut cpu = cpu::Core::create(0x0);
//     cpu.set_pc(vbase);

//     for _i in 0..6 {
//         cpu.cycle(mmu);
//         cpu.write_csr(
//             cpu::CSRRegister::mip,
//             mmu.tick(cpu.read_csr(cpu::CSRRegister::mip)),
//         );
//     }
// }

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
            mmu.write32(tohost_addr, 0x01);

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
                            println!("Cycles: {:#?}", cpu.cycles);
                            println!("Instret: {:#?}", cpu.read_csr(CSRRegister::instret));
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

fn bench(name: &str) -> bool {
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

    loop {
        if cpu.cycles > 20000000 {
            println!("Cycles: {:#?}", cpu.cycles);
            println!("Instret: {:#?}", cpu.read_csr(CSRRegister::minstret));
            return false;
        }

        cpu.cycle(mmu);
        cpu.write_csr(
            cpu::CSRRegister::mip,
            mmu.tick(cpu.read_csr(cpu::CSRRegister::mip)),
        );
        //        println!("case: ticked");

        if tohost_addr != 0 {
            let magic = mmu.read64(tohost_addr).unwrap();

            if magic != 0x00 {
                let which = mmu.read64(magic).unwrap();
                // syscall
                let arg0 = mmu.read64(magic.wrapping_add(8)).unwrap();
                let arg1 = mmu.read64(magic.wrapping_add(16)).unwrap();
                let arg2 = mmu.read64(magic.wrapping_add(24)).unwrap();
                let arg3 = mmu.read64(magic.wrapping_add(32)).unwrap();

                // Ack
                mmu.write32(tohost_addr, 0x00);

                match which {
                    0x40 => {
                        // printf
                        println!("PRINTF");
                    }
                    _ => {}
                }

                println!(
                    "Syscall: {:#x?} ({:#x?},{:#x?},{:#x?},{:#x?})",
                    which, arg0, arg1, arg2, arg3
                );
                //                return false;
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
                        "printstr" | "exit" | "tohost_exit" => {
                            let insts = cpu.read_csr(CSRRegister::minstret);
                            println!("Cycles: {:#?}", cpu.cycles);
                            println!("Instret: {:#?}", insts);
                            println!(
                                "Instructions per cycle: {:#?}",
                                (insts as f32)
                                    / ((cpu.cycles as f32 / CYCLES_PER_INSTRUCTION as f32) as f32)
                            );
                            println!("CASE: {}:\tOK ({:#} tests)", name, num_tests);
                            return true;
                        }
                        _ => {
                            // if symbol.starts_with("test_") {
                            //     num_tests = num_tests + 1;
                            //     curr_test = Some(symbol.clone().to_string());
                            //     println!(" - TEST: {:?} ", symbol);
                            // }
                            //                            println!("{:#x?}: {:?}", cpu.pc(), symbol);
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
