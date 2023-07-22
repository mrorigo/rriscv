use crate::{
    cpu::{CSRRegister, Core, Execution, PrivMode, Register},
    decoder::{DecodedInstruction, InstructionDecoder},
    memory::MemoryAccessWidth,
};

#[derive(Debug, Copy, Clone)]
pub enum MemoryAccess {
    READ8(usize),
    READ16(usize),
    READ32(usize),
    READ64(usize),
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
    fn enter_trap(&self) -> Stage;
    fn exit_trap(&self) -> Stage;
}

impl PipelineStages for Core<'_> {
    fn enter_trap(&self) -> Stage {
        todo!("pipeline: ENTER TRAP");
        Stage::EXIT_TRAP
    }

    fn exit_trap(&self) -> Stage {
        todo!("pipeline: EXIT TRAP");
        Stage::FETCH
    }

    fn memory(&mut self, memory_access: &MemoryAccess) -> Stage {
        match memory_access {
            MemoryAccess::READ8(offset) => todo!(),
            MemoryAccess::READ16(offset) => todo!(),
            MemoryAccess::READ32(offset) => todo!(),
            MemoryAccess::READ64(offset) => todo!(),
            _ => {}
        }
        match memory_access {
            MemoryAccess::WRITE8(offset, value) => todo!(),
            MemoryAccess::WRITE16(offset, value) => todo!(),
            MemoryAccess::WRITE32(offset, value) => todo!(),
            MemoryAccess::WRITE64(offset, value) => todo!(),
            _ => {}
        }
        Stage::WRITEBACK(None)
        //let decoded = self.pipe.decoded.as_ref().unwrap();
        //let memory = self.pipe.decoded.as_ref().unwrap();
        // match decoded.mem_offset {
        //     Some(offset) => {
        //         match decoded.instruction_format {
        //             // STORE
        //             InstructionFormat::S => {
        //                 self.memory.write_single(
        //                     (offset + self.memory.get_base_address()),
        //                     self.pipe.rs2v,
        //                     self.pipe.memory_access_width,
        //                 );
        //             }
        //             InstructionFormat::CSS => {
        //                 self.memory.write_single(
        //                     (/*self.read_register(decoded.rs1 as u64)
        //                     + */offset + self.memory.get_base_address()),
        //                     self.pipe.rs2v,
        //                     self.pipe.memory_access_width,
        //                 );
        //             }
        //             // LOAD
        //             InstructionFormat::I => {
        //                 todo!()
        //             }
        //             _ => panic!(),
        //         }
        //     }
        //     None => {}
        // }
        //Stage::WRITEBACK
    }

    fn writeback(&mut self, writeback: Option<WritebackStage>) -> Stage {
        match writeback {
            Some(wb) => self.write_register(wb.register, wb.value),
            None => {}
        }

        match self.pmode {
            PrivMode::Machine => {
                self.csrs[CSRRegister::minstret as usize] += 1;
            }
            _ => {
                todo!("Only machine mode so far")
            }
        }

        Stage::FETCH
    }

    fn execute(&mut self, decoded: &DecodedInstruction) -> Stage {
        let param: &dyn Execution = match decoded {
            DecodedInstruction::I(param) => param,
            DecodedInstruction::J(param) => param,
            DecodedInstruction::U(param) => param,
            DecodedInstruction::CR(param) => param,
            DecodedInstruction::B(param) => param,
            DecodedInstruction::S(param) => param,
            DecodedInstruction::R(param) => param,
            DecodedInstruction::CI(param) => param,
            DecodedInstruction::CSS(param) => param,
            DecodedInstruction::CIW(param) => param,
            DecodedInstruction::CL(param) => param,
            DecodedInstruction::CS(param) => param,
            DecodedInstruction::CB(param) => param,
            DecodedInstruction::CJ(param) => param,
        };
        param.execute(&self)
    }

    fn decode(&mut self, instruction: &RawInstruction) -> Stage {
        self.prev_pc = instruction.pc;
        let decoded = (self as &dyn InstructionDecoder).decode_instruction(*instruction);
        println!("decode: decoded: {:?}", decoded);
        Stage::EXECUTE(decoded)
    }

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
}
