use elfloader::VAddr;
use quark::Signs;

use crate::{
    cpu::{CSRRegister, Core, PrivMode, Register, RegisterValue, TrapCause, Xlen},
    instructions::{
        decoder::{DecodedInstruction, InstructionDecoder},
        InstructionExcecutor, InstructionSelector,
    },
    memory::{MemoryOperations, RAMOperations},
    mmu::MMU,
};

macro_rules! pipeline_trace {
    ($instr:expr) => {
        // print!("P: ");
        // $instr;
    };
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum MemoryAccess {
    AMOSWAP_W(VAddr, RegisterValue, Register),
    AMOSWAP_D(VAddr, VAddr, Register),
    READ8(VAddr, Register, bool),
    READ16(VAddr, Register, bool),
    READ32(VAddr, Register, bool),
    READ64(VAddr, Register, bool),
    WRITE8(VAddr, u8),
    WRITE16(VAddr, u16),
    WRITE32(VAddr, u32),
    WRITE64(VAddr, u64),
}

#[derive(Debug, Copy, Clone)]
pub enum MemoryAccessWidth {
    BYTE,     // 8 bits
    HALFWORD, // 16 bits
    WORD,     // 32 bits
    LONG,
}

#[derive(Debug, Copy, Clone)]
pub struct RawInstruction {
    pub compressed: bool,
    pub word: u32,
    pub pc: u64,
}

#[derive(Debug, Copy, Clone)]
pub struct WritebackStage {
    pub register: Register,
    pub value: u64,
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Stage {
    ENTER_TRAP(TrapCause),
    EXIT_TRAP,
    FETCH,
    DECODE(RawInstruction),
    EXECUTE(DecodedInstruction),       // May be skipped (by eg NOP)
    MEMORY(MemoryAccess),              // May be skipped
    WRITEBACK(Option<WritebackStage>), // This stage is ALWAYS executed
}

pub trait CacheableInstruction {}

//const INSTRUCTION_CACHE: HashMap<u32, Box<dyn CacheableInstruction>> = HashMap::new();

impl Stage {
    pub fn writeback(register: Register, value: RegisterValue) -> Stage {
        Stage::WRITEBACK(Some(WritebackStage { register, value }))
    }
}

pub trait PipelineStages {
    fn fetch(&mut self, mmu: &mut MMU) -> Stage;
    fn decode(&mut self, instruction: &RawInstruction) -> Stage;
    fn execute(&mut self, decoded: &DecodedInstruction) -> Stage;
    fn memory(&mut self, mmu: &mut MMU, memory_access: &MemoryAccess) -> Stage;

    //    fn memory(&mut self, memory_access: &MemoryAccess) -> Stage;
    fn writeback(&mut self, writeback: Option<WritebackStage>) -> Stage;
    fn enter_trap(&mut self, cause: TrapCause) -> Stage;
    fn exit_trap(&mut self) -> Stage;
}

impl PipelineStages for Core {
    fn fetch(&mut self, mmu: &mut MMU) -> Stage {
        let word = mmu.fetch(self.pc());

        let instruction;
        if word.is_some() {
            instruction = word.unwrap()
        } else {
            return Stage::ENTER_TRAP(TrapCause::InstructionAccessFault);
        }
        // println!("f: pc={:#x?}", self.pc());
        // Determine if instruction is compressed
        let instruction = match (instruction & 0x3) == 0x03 {
            true => {
                pipeline_trace!(println!(
                    "f:     instruction @ {:#x}: {:#x}",
                    self.pc(),
                    instruction
                ));
                self.add_pc(4);
                RawInstruction {
                    compressed: false,
                    word: instruction,
                    pc: self.pc() - 4,
                }
            }
            false => {
                pipeline_trace!(println!(
                    "f:     instruction @ {:#x?}: {:#x?} (C)",
                    self.pc(),
                    instruction & 0xffff,
                ));
                self.add_pc(2);
                RawInstruction {
                    compressed: true,
                    word: instruction & 0xffff,
                    pc: self.pc() - 2,
                }
            }
        };
        Stage::DECODE(instruction)
    }

    fn decode(&mut self, instruction: &RawInstruction) -> Stage {
        self.prev_pc = instruction.pc;
        let decoded = (self as &dyn InstructionDecoder).decode_instruction(*instruction);
        pipeline_trace!(println!("d:    {:?}", decoded));

        Stage::EXECUTE(decoded)
    }

    fn execute(&mut self, decoded: &DecodedInstruction) -> Stage {
        match *decoded {
            DecodedInstruction::I(inst) => inst.select(self.xlen).run(self),
            DecodedInstruction::U(inst) => inst.select(self.xlen).run(self),
            DecodedInstruction::CI(param) => param.select(self.xlen).run(self),
            DecodedInstruction::J(param) => param.select(self.xlen).run(self),
            DecodedInstruction::CR(param) => param.select(self.xlen).run(self),
            DecodedInstruction::B(param) => param.select(self.xlen).run(self),
            DecodedInstruction::S(param) => param.select(self.xlen).run(self),
            DecodedInstruction::R(param) => param.select(self.xlen).run(self),
            DecodedInstruction::CSS(param) => param.select(self.xlen).run(self),
            DecodedInstruction::CIW(param) => param.select(self.xlen).run(self),
            DecodedInstruction::CL(param) => param.select(self.xlen).run(self),
            DecodedInstruction::CS(param) => param.select(self.xlen).run(self),
            DecodedInstruction::CB(param) => param.select(self.xlen).run(self),
            DecodedInstruction::CJ(param) => param.select(self.xlen).run(self),
        }
    }

    fn memory(&mut self, mmu: &mut MMU, memory_access: &MemoryAccess) -> Stage {
        match *memory_access {
            MemoryAccess::READ8(offset, register, sign_extend) => {
                let value = mmu.read8(offset);
                if value.is_err() {
                    return Stage::ENTER_TRAP(value.err().unwrap());
                }
                // pipeline_trace!(println!("m:    READ8 @ {:#x?}: {:#x?}", offset, value));
                let value = value.unwrap();
                Stage::WRITEBACK(Some(WritebackStage {
                    register: register,
                    value: match sign_extend {
                        false => value as u64,
                        true => value as i8 as i16 as i32 as u64,
                    },
                }))
            }
            MemoryAccess::READ16(offset, register, sign_extend) => {
                let h = mmu.read8(offset + 1).unwrap() as u16;
                let l = mmu.read8(offset).unwrap() as u16;
                let value = (h << 8 | l) as i16 as u64;
                // pipeline_trace!(println!("m:    READ16 @ {:#x?}: {:#x?}", offset, value));

                Stage::WRITEBACK(Some(WritebackStage {
                    register: register,
                    value: match sign_extend {
                        false => (value & 0xffff) as u64,
                        true => value as i16 as i32 as u64,
                    },
                }))
                // let v = self.mmu.read_16(offset).unwrap();
                // Stage::WRITEBACK(Some(WritebackStage {
                //     register: register,
                //     value: v as u64,
                // }))
            }
            MemoryAccess::READ32(offset, register, sign_extend) => {
                let value = mmu.read_32(offset).unwrap();
                // pipeline_trace!(println!("m:    READ32 @ {:#x?}: {:#x?}", offset, value));
                Stage::WRITEBACK(Some(WritebackStage {
                    register: register,
                    value: match sign_extend {
                        true => value as i32 as i64 as u64,
                        false => (value & 0xffffffff) as u64,
                    },
                }))
            }
            MemoryAccess::READ64(offset, register, sign_extend) => {
                let l = match mmu.read_32(offset) {
                    None => return Stage::ENTER_TRAP(TrapCause::LoadAccessFault(offset)),
                    Some(val) => val,
                };
                let h = match mmu.read_32(offset + 4) {
                    None => return Stage::ENTER_TRAP(TrapCause::LoadAccessFault(offset + 4)),
                    Some(val) => val,
                };
                let comp = ((h as u64) << 32) | l as u64;
                let value = match sign_extend {
                    true => comp.sign_extend(64 - 32),
                    false => comp as u64,
                };
                // pipeline_trace!(println!(
                //     "m:    READ64 @ {:#x?}: {:#x?} ({:?})",
                //     offset, value, sign_extend
                // ));
                Stage::WRITEBACK(Some(WritebackStage {
                    register: register,
                    value,
                }))
            }
            MemoryAccess::WRITE8(offset, value) => {
                pipeline_trace!(println!("m:    WRITE8 @ {:#x?}: {:#x}", offset, value));
                mmu.write8(offset, value);
                Stage::WRITEBACK(None)
            }
            MemoryAccess::WRITE16(offset, value) => {
                mmu.write8(offset + 1, (value >> 8) as u8);
                mmu.write8(offset, (value & 0xff) as u8);
                Stage::WRITEBACK(None)
            }
            MemoryAccess::WRITE32(offset, value) => {
                pipeline_trace!(println!("m:    WRITE32 @ {:#x?}: {:#x?}", offset, value));
                mmu.write_32(offset, value);
                Stage::WRITEBACK(None)
            }
            MemoryAccess::WRITE64(offset, value) => {
                pipeline_trace!(println!("m:    WRITE64 @ {:#x?}: {:#x?}", offset, value));
                mmu.write_32(offset + 0, value as u32);
                mmu.write_32(offset + 4, (value >> 32) as u32);
                Stage::WRITEBACK(None)
            }
            MemoryAccess::AMOSWAP_W(from, rs2v, rd) => {
                let v1 = mmu.read_32(from).unwrap();
                //let v2 = mmu.read_32(to).unwrap();
                mmu.write_32(from, rs2v as u32);
                pipeline_trace!(println!(
                    "m:    AMOSWAP.W @ {:#x?} = {:#x?} now {:#x?}",
                    from, v1, rs2v
                ));

                // "AMOs can be used to implement parallel reduction operations,
                //   where typically the return value would be discarded by writing to x0."
                if rd == 0 {
                    Stage::WRITEBACK(None)
                } else {
                    //  AMOs can either operate on 64-bit (RV64 only) or 32-bit words in memory.
                    // For RV64, 32-bit AMOs always sign-extend the value placed in rd. T
                    let value = match self.xlen {
                        Xlen::Bits32 => v1 as u64,
                        Xlen::Bits64 => v1 as i32 as i64 as u64,
                    };
                    Stage::writeback(rd, value as u64)
                }
            }
            MemoryAccess::AMOSWAP_D(_from, _to, _rd) => todo!(),
        }
    }

    fn writeback(&mut self, writeback: Option<WritebackStage>) -> Stage {
        match writeback {
            Some(wb) if wb.register > 0 => {
                pipeline_trace!(println!("w: x{} = {:#x?}", wb.register, wb.value));

                self.write_register(wb.register, wb.value)
            }
            Some(wb) if wb.register == 0 => { /*warn*/ }
            _ => {}
        }

        // Update the instret CSR based on what PrivMode we are in

        let (instretcsr, instrethcsr) = match self.pmode() {
            PrivMode::Machine => (CSRRegister::minstret, CSRRegister::minstreth),
            PrivMode::Supervisor => (CSRRegister::instret, CSRRegister::instreth),
            PrivMode::User => (CSRRegister::instret, CSRRegister::instreth),
        };
        let instret = self.read_csr(instretcsr);
        if instret.wrapping_add(1) < instret {
            self.write_csr(instrethcsr, self.read_csr(instrethcsr).wrapping_add(1));
        }
        self.write_csr(instretcsr, instret.wrapping_add(1));

        // Update clint

        Stage::FETCH
    }

    fn enter_trap(&mut self, cause: TrapCause) -> Stage {
        self.debug_breakpoint(cause);
        panic!("ENTER_TRAP {:#x?}", cause);
        let causereg = match self.pmode() {
            PrivMode::Machine => CSRRegister::mcause,
            PrivMode::User => CSRRegister::mcause,
            PrivMode::Supervisor => CSRRegister::scause,
        };
        todo!();

        // match cause as u16 & 0x100 {
        //     0x100 => {
        //         // interrupt
        //         self.write_csr(causereg, cause as u8 as u64);
        //         self.write_csr(CSRRegister::mtval, 0);
        //         self.set_pc(self.pc() + 4);
        //     }
        //     _ => {
        //         self.write_csr(causereg, (cause as u8 - 1) as u64);
        //         // @TODO: mtval should be set to writeback-value if LoadAccessFault
        //         self.write_csr(CSRRegister::mtval, self.pc());
        //     }
        // }
        self.write_csr(CSRRegister::mepc, self.pc());

        // TODO: Handle WFI
        self.write_csr(
            CSRRegister::mstatus,
            (self.read_csr(CSRRegister::mstatus) & 0x08) << 4,
        );

        self.set_pc(self.read_csr(CSRRegister::mtvec) - 4);

        panic!("TRAP");
        Stage::EXIT_TRAP
    }

    fn exit_trap(&mut self) -> Stage {
        Stage::FETCH
    }
}
