use elfloader::VAddr;

use crate::{
    cpu::{CSRRegister, Core, PrivMode, Register, RegisterValue},
    instructions::{
        decoder::{DecodedInstruction, InstructionDecoder},
        InstructionExcecutor, InstructionSelector,
    },
    memory::{MemoryOperations, RAMOperations},
    mmu::MMU,
};

macro_rules! pipeline_trace {
    ($instr:expr) => {
        print!("PIPELINE: ");
        $instr;
    };
}

#[derive(Debug, Copy, Clone)]
pub enum MemoryAccess {
    AMOSWAP_W(VAddr, VAddr, Register),
    AMOSWAP_D(VAddr, VAddr, Register),
    READ8(VAddr, Register),
    READ16(VAddr, Register),
    READ32(VAddr, Register),
    READ64(VAddr, Register),
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
    ENTER_TRAP,
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
    fn enter_trap(&mut self) -> Stage;
    fn exit_trap(&mut self) -> Stage;
}

impl PipelineStages for Core {
    fn fetch(&mut self, mmu: &mut MMU) -> Stage {
        let instruction = mmu.read_32(self.pc).unwrap();

        // Determine if instruction is compressed
        let instruction = match (instruction & 0x3) == 0x03 {
            true => {
                pipeline_trace!(println!(
                    "fetch:     instruction @ {:#x}: {:#x}",
                    self.pc, instruction
                ));
                self.pc += 4;
                RawInstruction {
                    compressed: false,
                    word: instruction,
                    pc: self.pc - 4,
                }
            }
            false => {
                pipeline_trace!(println!(
                    "fetch:     instruction @ {:#x?}: {:#x?} (compressed)",
                    self.pc,
                    instruction & 0xffff,
                ));
                self.pc += 2;
                RawInstruction {
                    compressed: true,
                    word: instruction & 0xffff,
                    pc: self.pc - 2,
                }
            }
        };
        Stage::DECODE(instruction)
    }

    fn decode(&mut self, instruction: &RawInstruction) -> Stage {
        self.prev_pc = instruction.pc;
        let decoded = (self as &dyn InstructionDecoder).decode_instruction(*instruction);
        pipeline_trace!(println!("decode:    {:?}", decoded));
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
            MemoryAccess::READ8(offset, register) => {
                let value = mmu.read_single(offset).unwrap();
                pipeline_trace!(println!("memory:    READ8 @ {:#x?}: {:#x?}", offset, value));

                Stage::WRITEBACK(Some(WritebackStage {
                    register: register,
                    value: value as u64,
                }))
            }
            MemoryAccess::READ16(_offset, _register) => {
                todo!()
                // let v = self.mmu.read_16(offset).unwrap();
                // Stage::WRITEBACK(Some(WritebackStage {
                //     register: register,
                //     value: v as u64,
                // }))
            }
            MemoryAccess::READ32(offset, register) => {
                let value = mmu.read_32(offset).unwrap();
                pipeline_trace!(println!(
                    "memory:    READ32 @ {:#x?}: {:#x?}",
                    offset, value
                ));
                Stage::WRITEBACK(Some(WritebackStage {
                    register: register,
                    value: value as u64,
                }))
            }
            MemoryAccess::READ64(offset, register) => {
                let l = mmu.read_32(offset).unwrap();
                let h = mmu.read_32(offset + 4).unwrap();
                let value = ((h as u64) << 32) | l as u64;
                pipeline_trace!(println!(
                    "memory:    READ64 @ {:#x?}: {:#x?}",
                    offset, value
                ));
                Stage::WRITEBACK(Some(WritebackStage {
                    register: register,
                    value,
                }))
            }
            MemoryAccess::WRITE8(offset, value) => {
                pipeline_trace!(println!("memory:    WRITE8 @ {:#x?}: {:#x}", offset, value));
                mmu.write_single(offset, value);
                Stage::WRITEBACK(None)
            }
            MemoryAccess::WRITE16(_offset, _value) => {
                todo!();
                // mmu.write_single(offset + 1, (value & 0xff) as u8);
                // mmu.write_single(offset, (value >> 8) as u8);
                // Stage::WRITEBACK(None)
            }
            MemoryAccess::WRITE32(offset, value) => {
                pipeline_trace!(println!(
                    "memory:    WRITE32 @ {:#x?}: {:#?}",
                    offset, value
                ));
                mmu.write_32(offset, value);
                Stage::WRITEBACK(None)
            }
            MemoryAccess::WRITE64(offset, value) => {
                pipeline_trace!(println!(
                    "memory:    WRITE64 @ {:#x?}: {:#x?}",
                    offset, value
                ));
                mmu.write_32(offset, value as u32);
                mmu.write_32(offset + 4, (value >> 32) as u32);
                // pipeline_trace!(println!("memory::WRITE64: {:#x?} @ {:#x?}", value, offset));
                // self.mmu
                //     .write_single(offset, value as u64, MemoryAccessWidth::LONG);
                Stage::WRITEBACK(None)
            }
            MemoryAccess::AMOSWAP_W(from, to, rd) => {
                let v1 = mmu.read_32(from).unwrap();
                let v2 = mmu.read_32(to).unwrap();
                mmu.write_32(to, v1);
                mmu.write_32(from, v2);
                Stage::writeback(rd, v1 as u64)
            }
            MemoryAccess::AMOSWAP_D(_from, _to, _rd) => todo!(),
        }
    }

    fn writeback(&mut self, writeback: Option<WritebackStage>) -> Stage {
        match writeback {
            Some(wb) => {
                pipeline_trace!(println!("writeback: x{} = {:#x?}", wb.register, wb.value));

                self.write_register(wb.register, wb.value)
            }
            None => {}
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

        Stage::FETCH
    }

    fn enter_trap(&mut self) -> Stage {
        Stage::EXIT_TRAP
    }

    fn exit_trap(&mut self) -> Stage {
        Stage::FETCH
    }
}
