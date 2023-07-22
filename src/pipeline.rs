use crate::{
    cpu::{CSRRegister, Core, PrivMode, Register},
    instructions::{
        decoder::{DecodedInstruction, InstructionDecoder},
        InstructionExcecutor, InstructionSelector,
    },
    memory::MemoryAccessWidth,
};

#[derive(Debug, Copy, Clone)]
pub enum MemoryAccess {
    READ8(usize, Register),
    READ16(usize, Register),
    READ32(usize, Register),
    READ64(usize, Register),
    WRITE8(usize, u8),
    WRITE16(usize, u16),
    WRITE32(usize, u32),
    WRITE64(usize, u64),
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

pub trait PipelineStages {
    fn fetch(&mut self) -> Stage;
    fn decode(&mut self, instruction: &RawInstruction) -> Stage;
    fn execute(&mut self, decoded: &DecodedInstruction) -> Stage;
    fn memory(&mut self, memory_access: &MemoryAccess) -> Stage;
    fn writeback(&mut self, writeback: Option<WritebackStage>) -> Stage;
    fn enter_trap(&mut self) -> Stage;
    fn exit_trap(&mut self) -> Stage;
}

impl PipelineStages for Core<'_> {
    fn fetch(&mut self) -> Stage {
        let instruction = self
            .memory
            .read_single(self.pc, MemoryAccessWidth::WORD)
            .unwrap() as u32;

        println!("fetch: instruction @ {:#x}: {:#x}", self.pc, instruction);

        // Determine if instruction is compressed
        let instruction = match (instruction & 0x3) == 0x03 {
            true => {
                self.pc += 4;
                RawInstruction {
                    compressed: false,
                    word: instruction,
                    pc: self.pc - 4,
                }
            }
            false => {
                self.pc += 2;
                println!("fetch: compressed instruction {:#x?}", instruction & 0xffff);
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
        println!("decode: decoded: {:?}", decoded);
        Stage::EXECUTE(decoded)
    }

    fn execute(&mut self, decoded: &DecodedInstruction) -> Stage {
        match *decoded {
            DecodedInstruction::I(inst) => inst.select().run(self),
            DecodedInstruction::U(inst) => inst.select().run(self),
            DecodedInstruction::CI(param) => param.select().run(self),
            DecodedInstruction::J(param) => param.select().run(self),
            DecodedInstruction::CR(param) => param.select().run(self),
            DecodedInstruction::B(param) => param.select().run(self),
            DecodedInstruction::S(param) => param.select().run(self),
            DecodedInstruction::R(param) => param.select().run(self),
            DecodedInstruction::CSS(param) => param.select().run(self),
            DecodedInstruction::CIW(param) => param.select().run(self),
            DecodedInstruction::CL(param) => param.select().run(self),
            DecodedInstruction::CS(param) => param.select().run(self),
            DecodedInstruction::CB(param) => param.select().run(self),
            DecodedInstruction::CJ(param) => param.select().run(self),
        }
    }

    fn memory(&mut self, memory_access: &MemoryAccess) -> Stage {
        match *memory_access {
            MemoryAccess::READ8(offset, register) => Stage::WRITEBACK(Some(WritebackStage {
                register: register,
                value: self
                    .memory
                    .read_single(
                        offset as u64 + self.memory.get_base_address(),
                        MemoryAccessWidth::BYTE,
                    )
                    .unwrap(),
            })),
            MemoryAccess::READ16(offset, register) => Stage::WRITEBACK(Some(WritebackStage {
                register: register,
                value: self
                    .memory
                    .read_single(
                        offset as u64 + self.memory.get_base_address(),
                        MemoryAccessWidth::HALFWORD,
                    )
                    .unwrap(),
            })),
            MemoryAccess::READ32(offset, register) => Stage::WRITEBACK(Some(WritebackStage {
                register: register,
                value: self
                    .memory
                    .read_single(
                        offset as u64 + self.memory.get_base_address(),
                        MemoryAccessWidth::WORD,
                    )
                    .unwrap(),
            })),
            MemoryAccess::READ64(offset, register) => Stage::WRITEBACK(Some(WritebackStage {
                register: register,
                value: self
                    .memory
                    .read_single(
                        offset as u64 + self.memory.get_base_address(),
                        MemoryAccessWidth::LONG,
                    )
                    .unwrap(),
            })),
            MemoryAccess::WRITE8(offset, value) => {
                self.memory.write_single(
                    offset as u64 + self.memory.get_base_address(),
                    value as u64,
                    MemoryAccessWidth::BYTE,
                );
                Stage::WRITEBACK(None)
            }
            MemoryAccess::WRITE16(offset, value) => {
                self.memory.write_single(
                    offset as u64 + self.memory.get_base_address(),
                    value as u64,
                    MemoryAccessWidth::HALFWORD,
                );
                Stage::WRITEBACK(None)
            }
            MemoryAccess::WRITE32(offset, value) => {
                self.memory.write_single(
                    offset as u64 + self.memory.get_base_address(),
                    value as u64,
                    MemoryAccessWidth::WORD,
                );
                Stage::WRITEBACK(None)
            }
            MemoryAccess::WRITE64(offset, value) => {
                self.memory.write_single(
                    offset as u64 + self.memory.get_base_address(),
                    value as u64,
                    MemoryAccessWidth::LONG,
                );
                Stage::WRITEBACK(None)
            }
        }
    }

    fn writeback(&mut self, writeback: Option<WritebackStage>) -> Stage {
        match writeback {
            Some(wb) => self.write_register(wb.register, wb.value),
            None => {}
        }

        // Update the instret CSR based on what PrivMode we are in

        let (instretcsr, instrethcsr) = match self.pmode {
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
