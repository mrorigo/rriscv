use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use elfloader::{ElfBinary, VAddr};
use rriscv::cpu::{PrivMode, TrapCause};
use rriscv::elf;
use rriscv::{
    cpu::{self},
    mmu::MMU,
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

    let binary_blob = fs::read("examples/xv6/kernel").expect("Can't read xv6 kernel binary");
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
    cpu.reset(vbase);

    for sym in symbols.iter() {
        cpu.add_symbol(*sym.0 as VAddr, sym.1.to_string());
    }

    let stop: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let stop_me = stop.clone();

    ctrlc::set_handler(move || {
        stop_me.store(true, std::sync::atomic::Ordering::Relaxed)
        //        cpu.debug_breakpoint(cpu::TrapCause::Breakpoint)
    })
    .expect("Error setting Ctrl-C handler");

    let mut oldx10 = 0;
    loop {
        cpu.cycle(mmu);
        match cpu.stage {
            Stage::FETCH => {
                if stop.load(std::sync::atomic::Ordering::Relaxed) == true {
                    stop.store(false, std::sync::atomic::Ordering::Relaxed);
                    cpu.debug_breakpoint(cpu::TrapCause::Breakpoint, mmu);
                }

                cpu.write_csr(
                    cpu::CSRRegister::mip,
                    mmu.tick(cpu.read_csr(cpu::CSRRegister::mip)),
                );

                if cpu.pmode() == PrivMode::Supervisor {
                    let x10 = cpu.read_register(10);
                    if x10 != oldx10 {
                        //println!("{:#x?}: x10:{:#x?}", cpu.pc(), x10);
                        oldx10 = x10;
                    }
                    //                    Debugger::dump_status(&mut cpu, mmu);
                }
            }
            Stage::TRAP(cause) => match cause {
                TrapCause::Breakpoint => {
                    stop.store(true, std::sync::atomic::Ordering::Relaxed);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
