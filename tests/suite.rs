use std::{collections::HashMap, env};

use elfloader::{ElfBinary, VAddr};
use rriscv::{
    cpu::{self},
    elf,
    mmu::MMU,
    pipeline::Stage,
};

const CASES: [&str; 30] = [
    //"../../git/riscv-tests/isa/rv64mi-p-access",
    //"../../git/riscv-tests/isa/rv64mi-p-breakpoint",
    //"../../git/riscv-tests/isa/rv64mi-p-csr",
    //"../../git/riscv-tests/isa/rv64mi-p-illegal",
    // "../../git/riscv-tests/isa/rv64mi-p-ld-misaligned",
    // "../../git/riscv-tests/isa/rv64mi-p-lh-misaligned",
    // "../../git/riscv-tests/isa/rv64mi-p-lw-misaligned",
    // "../../git/riscv-tests/isa/rv64mi-p-ma_addr",
    // "../../git/riscv-tests/isa/rv64mi-p-ma_fetch",
    // "../../git/riscv-tests/isa/rv64mi-p-mcsr",
    // "../../git/riscv-tests/isa/rv64mi-p-sbreak",
    // "../../git/riscv-tests/isa/rv64mi-p-scall",
    // "../../git/riscv-tests/isa/rv64mi-p-sd-misaligned",
    // "../../git/riscv-tests/isa/rv64mi-p-sh-misaligned",
    // "../../git/riscv-tests/isa/rv64mi-p-sw-misaligned",
    // "../../git/riscv-tests/isa/rv64mi-p-zicntr",
    // "../../git/riscv-tests/isa/rv64si-p-csr",
    // "../../git/riscv-tests/isa/rv64si-p-dirty",
    // "../../git/riscv-tests/isa/rv64si-p-icache-alias",
    // "../../git/riscv-tests/isa/rv64si-p-ma_fetch",
    // "../../git/riscv-tests/isa/rv64si-p-sbreak",
    // "../../git/riscv-tests/isa/rv64si-p-scall",
    // "../../git/riscv-tests/isa/rv64si-p-wfi",
    // "../../git/riscv-tests/isa/rv64ssvnapot-p-napot",
    // "../../git/riscv-tests/isa/rv64ua-p-amoadd_d",
    // "../../git/riscv-tests/isa/rv64ua-p-amoadd_w",
    // "../../git/riscv-tests/isa/rv64ua-p-amoand_d",
    // "../../git/riscv-tests/isa/rv64ua-p-amoand_w",
    // "../../git/riscv-tests/isa/rv64ua-p-amomax_d",
    // "../../git/riscv-tests/isa/rv64ua-p-amomax_w",
    // "../../git/riscv-tests/isa/rv64ua-p-amomaxu_d",
    // "../../git/riscv-tests/isa/rv64ua-p-amomaxu_w",
    // "../../git/riscv-tests/isa/rv64ua-p-amomin_d",
    // "../../git/riscv-tests/isa/rv64ua-p-amomin_w",
    // "../../git/riscv-tests/isa/rv64ua-p-amominu_d",
    // "../../git/riscv-tests/isa/rv64ua-p-amominu_w",
    // "../../git/riscv-tests/isa/rv64ua-p-amoor_d",
    // "../../git/riscv-tests/isa/rv64ua-p-amoor_w",
    // "../../git/riscv-tests/isa/rv64ua-p-amoswap_d",
    "../../git/riscv-tests/isa/rv64ua-p-amoswap_w",
    // "../../git/riscv-tests/isa/rv64ua-p-amoxor_d",
    // "../../git/riscv-tests/isa/rv64ua-p-amoxor_w",
    // "../../git/riscv-tests/isa/rv64ua-p-lrsc",
    // "../../git/riscv-tests/isa/rv64ua-v-amoadd_d",
    // "../../git/riscv-tests/isa/rv64ua-v-amoadd_w",
    // "../../git/riscv-tests/isa/rv64ua-v-amoand_d",
    // "../../git/riscv-tests/isa/rv64ua-v-amoand_w",
    // "../../git/riscv-tests/isa/rv64ua-v-amomax_d",
    // "../../git/riscv-tests/isa/rv64ua-v-amomax_w",
    // "../../git/riscv-tests/isa/rv64ua-v-amomaxu_d",
    // "../../git/riscv-tests/isa/rv64ua-v-amomaxu_w",
    // "../../git/riscv-tests/isa/rv64ua-v-amomin_d",
    // "../../git/riscv-tests/isa/rv64ua-v-amomin_w",
    // "../../git/riscv-tests/isa/rv64ua-v-amominu_d",
    // "../../git/riscv-tests/isa/rv64ua-v-amominu_w",
    // "../../git/riscv-tests/isa/rv64ua-v-amoor_d",
    // "../../git/riscv-tests/isa/rv64ua-v-amoor_w",
    // "../../git/riscv-tests/isa/rv64ua-v-amoswap_d",
    // "../../git/riscv-tests/isa/rv64ua-v-amoswap_w",
    // "../../git/riscv-tests/isa/rv64ua-v-amoxor_d",
    // "../../git/riscv-tests/isa/rv64ua-v-amoxor_w",
    // "../../git/riscv-tests/isa/rv64ua-v-lrsc",
    // "../../git/riscv-tests/isa/rv64uc-p-rvc",
    // "../../git/riscv-tests/isa/rv64uc-v-rvc",
    // "../../git/riscv-tests/isa/rv64ud-p-fadd",
    // "../../git/riscv-tests/isa/rv64ud-p-fclass",
    // "../../git/riscv-tests/isa/rv64ud-p-fcmp",
    // "../../git/riscv-tests/isa/rv64ud-p-fcvt",
    // "../../git/riscv-tests/isa/rv64ud-p-fcvt_w",
    // "../../git/riscv-tests/isa/rv64ud-p-fdiv",
    // "../../git/riscv-tests/isa/rv64ud-p-fmadd",
    // "../../git/riscv-tests/isa/rv64ud-p-fmin",
    // "../../git/riscv-tests/isa/rv64ud-p-ldst",
    // "../../git/riscv-tests/isa/rv64ud-p-move",
    // "../../git/riscv-tests/isa/rv64ud-p-recoding",
    // "../../git/riscv-tests/isa/rv64ud-p-structural",
    // "../../git/riscv-tests/isa/rv64ud-v-fadd",
    // "../../git/riscv-tests/isa/rv64ud-v-fclass",
    // "../../git/riscv-tests/isa/rv64ud-v-fcmp",
    // "../../git/riscv-tests/isa/rv64ud-v-fcvt",
    // "../../git/riscv-tests/isa/rv64ud-v-fcvt_w",
    // "../../git/riscv-tests/isa/rv64ud-v-fdiv",
    // "../../git/riscv-tests/isa/rv64ud-v-fmadd",
    // "../../git/riscv-tests/isa/rv64ud-v-fmin",
    // "../../git/riscv-tests/isa/rv64ud-v-ldst",
    // "../../git/riscv-tests/isa/rv64ud-v-move",
    // "../../git/riscv-tests/isa/rv64ud-v-recoding",
    // "../../git/riscv-tests/isa/rv64ud-v-structural",
    // "../../git/riscv-tests/isa/rv64uf-p-fadd",
    // "../../git/riscv-tests/isa/rv64uf-p-fclass",
    // "../../git/riscv-tests/isa/rv64uf-p-fcmp",
    // "../../git/riscv-tests/isa/rv64uf-p-fcvt",
    // "../../git/riscv-tests/isa/rv64uf-p-fcvt_w",
    // "../../git/riscv-tests/isa/rv64uf-p-fdiv",
    // "../../git/riscv-tests/isa/rv64uf-p-fmadd",
    // "../../git/riscv-tests/isa/rv64uf-p-fmin",
    // "../../git/riscv-tests/isa/rv64uf-p-ldst",
    // "../../git/riscv-tests/isa/rv64uf-p-move",
    // "../../git/riscv-tests/isa/rv64uf-p-recoding",
    // "../../git/riscv-tests/isa/rv64uf-v-fadd",
    // "../../git/riscv-tests/isa/rv64uf-v-fclass",
    // "../../git/riscv-tests/isa/rv64uf-v-fcmp",
    // "../../git/riscv-tests/isa/rv64uf-v-fcvt",
    // "../../git/riscv-tests/isa/rv64uf-v-fcvt_w",
    // "../../git/riscv-tests/isa/rv64uf-v-fdiv",
    // "../../git/riscv-tests/isa/rv64uf-v-fmadd",
    // "../../git/riscv-tests/isa/rv64uf-v-fmin",
    // "../../git/riscv-tests/isa/rv64uf-v-ldst",
    // "../../git/riscv-tests/isa/rv64uf-v-move",
    // "../../git/riscv-tests/isa/rv64uf-v-recoding",
    // "../../git/riscv-tests/isa/rv64ui-p-add",
    "../../git/riscv-tests/isa/rv64ui-p-addi",
    "../../git/riscv-tests/isa/rv64ui-p-addiw",
    //"../../git/riscv-tests/isa/rv64ui-p-addw",
    "../../git/riscv-tests/isa/rv64ui-p-and",
    "../../git/riscv-tests/isa/rv64ui-p-andi",
    "../../git/riscv-tests/isa/rv64ui-p-sub",
    "../../git/riscv-tests/isa/rv64ui-p-auipc",
    "../../git/riscv-tests/isa/rv64ui-p-beq",
    "../../git/riscv-tests/isa/rv64ui-p-bge",
    "../../git/riscv-tests/isa/rv64ui-p-bgeu",
    "../../git/riscv-tests/isa/rv64ui-p-blt",
    "../../git/riscv-tests/isa/rv64ui-p-bltu",
    "../../git/riscv-tests/isa/rv64ui-p-bne",
    // "../../git/riscv-tests/isa/rv64ui-p-fence_i",
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
    // "../../git/riscv-tests/isa/rv64ui-p-sb",
    // "../../git/riscv-tests/isa/rv64ui-p-sd",
    // "../../git/riscv-tests/isa/rv64ui-p-sh",
    // "../../git/riscv-tests/isa/rv64ui-p-simple",
    ////"../../git/riscv-tests/isa/rv64ui-p-sll",
    ////"../../git/riscv-tests/isa/rv64ui-p-slli",
    ////"../../git/riscv-tests/isa/rv64ui-p-slliw",
    //"../../git/riscv-tests/isa/rv64ui-p-sllw",
    //"../../git/riscv-tests/isa/rv64ui-p-slt",
    //"../../git/riscv-tests/isa/rv64ui-p-slti",
    //"../../git/riscv-tests/isa/rv64ui-p-sltiu",
    ////"../../git/riscv-tests/isa/rv64ui-p-sltu",
    //"../../git/riscv-tests/isa/rv64ui-p-sra",
    //"../../git/riscv-tests/isa/rv64ui-p-srai",
    ////"../../git/riscv-tests/isa/rv64ui-p-sraiw",
    //"../../git/riscv-tests/isa/rv64ui-p-sraw",
    //"../../git/riscv-tests/isa/rv64ui-p-srl",
    "../../git/riscv-tests/isa/rv64ui-v-sb",
    "../../git/riscv-tests/isa/rv64ui-v-sd",
    "../../git/riscv-tests/isa/rv64ui-v-sh",
    "../../git/riscv-tests/isa/rv64ui-p-sw",
    "../../git/riscv-tests/isa/rv64ui-p-srli",
];

///"../../git/riscv-tests/isa/rv64ui-p-srliw",
//"../../git/riscv-tests/isa/rv64ui-p-srlw",
// "../../git/riscv-tests/isa/rv64ui-p-subw",
// "../../git/riscv-tests/isa/rv64ui-p-sw",
// "../../git/riscv-tests/isa/rv64ui-p-xor",
// "../../git/riscv-tests/isa/rv64ui-p-xori",
// "../../git/riscv-tests/isa/rv64ui-v-add",
// "../../git/riscv-tests/isa/rv64ui-v-addi",
// "../../git/riscv-tests/isa/rv64ui-v-addiw",
// "../../git/riscv-tests/isa/rv64ui-v-addw",
// "../../git/riscv-tests/isa/rv64ui-v-and",
// "../../git/riscv-tests/isa/rv64ui-v-andi",
// "../../git/riscv-tests/isa/rv64ui-v-auipc",
// "../../git/riscv-tests/isa/rv64ui-v-beq",
// "../../git/riscv-tests/isa/rv64ui-v-bge",
// "../../git/riscv-tests/isa/rv64ui-v-bgeu",
// "../../git/riscv-tests/isa/rv64ui-v-blt",
// "../../git/riscv-tests/isa/rv64ui-v-bltu",
// "../../git/riscv-tests/isa/rv64ui-v-bne",
// "../../git/riscv-tests/isa/rv64ui-v-fence_i",
// "../../git/riscv-tests/isa/rv64ui-v-jal",
// "../../git/riscv-tests/isa/rv64ui-v-jalr",
// "../../git/riscv-tests/isa/rv64ui-v-lb",
// "../../git/riscv-tests/isa/rv64ui-v-lbu",
// "../../git/riscv-tests/isa/rv64ui-v-ld",
// "../../git/riscv-tests/isa/rv64ui-v-lh",
// "../../git/riscv-tests/isa/rv64ui-v-lhu",
// "../../git/riscv-tests/isa/rv64ui-v-lui",
// "../../git/riscv-tests/isa/rv64ui-v-lw",
// "../../git/riscv-tests/isa/rv64ui-v-lwu",
// "../../git/riscv-tests/isa/rv64ui-v-ma_data",
// "../../git/riscv-tests/isa/rv64ui-v-or",
// "../../git/riscv-tests/isa/rv64ui-v-ori",
// "../../git/riscv-tests/isa/rv64ui-v-simple",
// "../../git/riscv-tests/isa/rv64ui-v-sll",
// "../../git/riscv-tests/isa/rv64ui-v-slli",
// "../../git/riscv-tests/isa/rv64ui-v-slliw",
// "../../git/riscv-tests/isa/rv64ui-v-sllw",
// "../../git/riscv-tests/isa/rv64ui-v-slt",
// "../../git/riscv-tests/isa/rv64ui-v-slti",
// "../../git/riscv-tests/isa/rv64ui-v-sltiu",
// "../../git/riscv-tests/isa/rv64ui-v-sltu",
// "../../git/riscv-tests/isa/rv64ui-v-sra",
// "../../git/riscv-tests/isa/rv64ui-v-srai",
// "../../git/riscv-tests/isa/rv64ui-v-sraiw",
// "../../git/riscv-tests/isa/rv64ui-v-sraw",
// "../../git/riscv-tests/isa/rv64ui-v-srl",
// "../../git/riscv-tests/isa/rv64ui-v-srli",
// "../../git/riscv-tests/isa/rv64ui-v-srliw",
// "../../git/riscv-tests/isa/rv64ui-v-srlw",
// "../../git/riscv-tests/isa/rv64ui-v-sub",
// "../../git/riscv-tests/isa/rv64ui-v-subw",
// "../../git/riscv-tests/isa/rv64ui-v-sw",
// "../../git/riscv-tests/isa/rv64ui-v-xor",
// "../../git/riscv-tests/isa/rv64ui-v-xori",
// "../../git/riscv-tests/isa/rv64um-p-div",
// "../../git/riscv-tests/isa/rv64um-p-divu",
// "../../git/riscv-tests/isa/rv64um-p-divuw",
// "../../git/riscv-tests/isa/rv64um-p-divw",
// "../../git/riscv-tests/isa/rv64um-p-mul",
// "../../git/riscv-tests/isa/rv64um-p-mulh",
// "../../git/riscv-tests/isa/rv64um-p-mulhsu",
// "../../git/riscv-tests/isa/rv64um-p-mulhu",
// "../../git/riscv-tests/isa/rv64um-p-mulw",
// "../../git/riscv-tests/isa/rv64um-p-rem",
// "../../git/riscv-tests/isa/rv64um-p-remu",
// "../../git/riscv-tests/isa/rv64um-p-remuw",
// "../../git/riscv-tests/isa/rv64um-p-remw",
// "../../git/riscv-tests/isa/rv64um-v-div",
// "../../git/riscv-tests/isa/rv64um-v-divu",
// "../../git/riscv-tests/isa/rv64um-v-divuw",
// "../../git/riscv-tests/isa/rv64um-v-divw",
// "../../git/riscv-tests/isa/rv64um-v-mul",
// "../../git/riscv-tests/isa/rv64um-v-mulh",
// "../../git/riscv-tests/isa/rv64um-v-mulhsu",
// "../../git/riscv-tests/isa/rv64um-v-mulhu",
// "../../git/riscv-tests/isa/rv64um-v-mulw",
// "../../git/riscv-tests/isa/rv64um-v-rem",
// "../../git/riscv-tests/isa/rv64um-v-remu",
// "../../git/riscv-tests/isa/rv64um-v-remuw",
// "../../git/riscv-tests/isa/rv64um-v-remw",
//];

#[test]
fn all_cases() {
    for argument in CASES.iter() {
        println!("========== {:?} ==========", argument);
        case(&argument.to_string());
        println!("");
    }
    assert!(false);
}

fn case(name: &str) -> bool {
    // println!("R-RISCV Emulator: Initializing for XV6 kernel");
    use std::fs;

    let vbase: u64 = 0x8000_0000;
    //    let mmu = MMU::create();
    // println!("DTB Device Table:");
    // mmu.dump_device_table();

    let mmu = &mut MMU::create();

    //    let binary_blob = fs::read("examples/xv6/kernel.min").expect("Can't read kernel binary");
    let binary_blob = fs::read(name).expect("Can't read kernel binary");
    //let binary_blob = fs::read("./test").expect("Can't read kernel binary");
    let binary = ElfBinary::new(binary_blob.as_slice()).expect("Got proper ELF file");
    let mut loader = elf::Loader::create(vbase, mmu);
    binary.load(&mut loader).expect("Can't load the binary?");

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
    cpu.set_pc(vbase);

    for sym in symbols.iter() {
        cpu.add_symbol(*sym.0 as VAddr, sym.1.to_string());
    }

    loop {
        cpu.cycle(mmu);
        mmu.tick();
        //println!("xv6: ticked");
        match cpu.stage {
            Stage::FETCH => {
                let key = cpu.pc() as u64;
                match symbols.get(&key) {
                    None => {}
                    Some(symbol) => match *symbol {
                        "fail" => {
                            println!("CASE: {}: fail ", name);
                            return false;
                        }
                        "pass" => {
                            println!("CASE: {}: pass", name);
                            return true;
                        }
                        _ => {
                            if symbol.starts_with("test_") {
                                //println!(" - CASE: {:?}", symbol);
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
