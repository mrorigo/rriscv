use elfloader::VAddr;
use rustyline::error::ReadlineError;
use rustyline::Result;

use crate::cpu::{CSRRegister, Core, Register, TrapCause, CYCLES_PER_INSTRUCTION};
use crate::disassembler::Disassembler;
use crate::memory::MemoryOperations;
use crate::mmu::MMU;
use crate::pipeline::RawInstruction;

pub struct Debugger {}

pub enum DebuggerResult {
    Step(usize),
    Continue,
    ContinueUntil(VAddr),
    Quit(String),
}

impl Debugger {
    pub fn create() -> Debugger {
        Debugger {}
    }

    pub fn enter(&self, core: &mut Core, mmu: &mut MMU, cause: TrapCause) -> DebuggerResult {
        println!("----- DEBUG BREAKPOINT: HALTING CPU\nCause: {:?}", cause);

        self.dump_status(core, mmu);

        match self.main(core, mmu) {
            Ok(result) => result,
            Err(er) => DebuggerResult::Quit(er.to_string()),
        }
    }

    fn parse_addr(core: &Core, s: &str) -> Option<VAddr> {
        match s {
            "pc" => Some(core.pc()),
            "mepc" => Some(core.read_csr(CSRRegister::mepc)),
            "sepc" => Some(core.read_csr(CSRRegister::sepc)),
            "sp" => Some(core.read_register(2)),
            _ => match u64::from_str_radix(s.trim_start_matches("0x"), 16) {
                Ok(val) => Some(val),
                Err(err) => {
                    println!("Debugger: Error: Invalid address {:#x?}: {:#?} ", s, err);
                    None
                }
            },
        }
    }

    fn main(&self, core: &mut Core, mmu: &mut MMU) -> Result<DebuggerResult> {
        let mut rl = rustyline::Editor::<()>::new()?;
        //#[cfg(feature = "with-file-history")]
        if rl.load_history("history.txt").is_err() {
            println!("No previous history.");
        }
        loop {
            let readline = rl.readline(">> ");
            let result = match readline {
                Ok(line) => {
                    rl.add_history_entry(line.as_str());

                    // match single word commands
                    let split = line.split_whitespace().collect::<Vec<&str>>();
                    if split.len() == 0 {
                        None
                    } else {
                        match split[0] {
                            "quit" => Some(DebuggerResult::Quit(String::from("Bye"))),
                            "bt" => {
                                for i in 0..core.symboltrace.len() {
                                    println!("{:x?}", core.symboltrace[i]);
                                }
                                None
                            }
                            "d" => {
                                let addr = match split.len() > 0 {
                                    true => Debugger::parse_addr(core, split[1]),
                                    false => {
                                        println!("Debugger: Error: No address supplied");
                                        None
                                    }
                                };

                                match addr {
                                    Some(addr) => {
                                        let mut offs = 0;
                                        for i in 0..10 {
                                            let word = mmu.read32(addr.wrapping_add(offs));

                                            let poffs = offs;
                                            let s = match word {
                                                Ok(word) => {
                                                    let raw = RawInstruction::from_word(word, 0);
                                                    offs += raw.size_in_bytes();
                                                    Disassembler::disassemble(word, core.xlen)
                                                }
                                                Err(_) => String::from("Invalid Address"),
                                            };
                                            println!("{:#x?}: {:?}", addr.wrapping_add(poffs), s);
                                        }
                                        None
                                    }
                                    None => None,
                                }
                            }
                            "b" => {
                                let addr = match split.len() > 0 {
                                    true => Debugger::parse_addr(core, split[1]),
                                    false => {
                                        println!("Debugger: Error: No address supplied");
                                        None
                                    }
                                };

                                match addr {
                                    Some(addr) => Some(DebuggerResult::ContinueUntil(addr)),
                                    None => None,
                                }
                            }
                            "m" => {
                                let addr = match split.len() > 0 {
                                    true => Debugger::parse_addr(core, split[1]),
                                    false => {
                                        println!("Debugger: Error: No address supplied");
                                        None
                                    }
                                };
                                match addr {
                                    Some(addr) => self.dump_memory(core, addr),
                                    _ => {}
                                }
                                None
                            }
                            "s" => {
                                let steps = match split.len() > 0 {
                                    true => match u64::from_str_radix(split[1], 10) {
                                        Ok(value) => value,
                                        Err(_) => 1,
                                    },
                                    false => 1,
                                } as usize
                                    * CYCLES_PER_INSTRUCTION;
                                println!(
                                    "Debugger: Stepping {:x?} cycles from  {:x?}",
                                    steps,
                                    core.pc()
                                );
                                Some(DebuggerResult::Step(steps))
                            }
                            "c" => {
                                println!("Debugger: Continuing from  {:x?} ", core.pc());
                                Some(DebuggerResult::Continue)
                            }
                            _ => None,
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    Some(DebuggerResult::Quit(String::from("CTRL-C")))
                }
                Err(ReadlineError::Eof) => Some(DebuggerResult::Continue),
                Err(err) => Some(DebuggerResult::Quit(err.to_string())),
            };
            if result.is_some() {
                match rl.save_history("history.txt") {
                    _ => {}
                }
                return Ok(result.unwrap());
            }
        }
    }

    fn dump_memory(&self, _core: &mut Core, _addr: VAddr) {}

    fn dump_status(&self, core: &mut Core, mmu: &mut MMU) {
        const STEP: usize = 4;
        for i in (0..32).step_by(STEP) {
            print!("x{}-{}:  ", i, i + (STEP - 1));
            if i == 0 {
                print!(" ");
            }
            if i < 10 {
                print!(" ");
            }
            for reg in i..i + STEP {
                print!("{:#020x?} ", core.read_register(reg as Register));
            }
            println!("");
        }
        let word1 = mmu.read32(core.prev_pc);
        let disasm1 = Disassembler::disassemble(word1.unwrap(), core.xlen);
        let word2 = mmu.read32(core.pc());
        let disasm2 = Disassembler::disassemble(word2.unwrap(), core.xlen);
        println!(
            "{:#10x}: {:?}\n{:#10x}: {:?}\nmstatus: {:#6x?}  sstatus: {:#6x?}\npc: {:#10x?} mepc: {:#10x?} sepc: {:#10x?}  pmode: {:?}",
            core.prev_pc, disasm1,
            core.pc(), disasm2,
            core.read_csr(CSRRegister::mstatus),
            core.read_csr(CSRRegister::sstatus),
            core.pc(),
            core.read_csr(CSRRegister::mepc),
            core.read_csr(CSRRegister::sepc),
            core.pmode()
        );
    }
}
