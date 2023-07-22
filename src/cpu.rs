use std::cell::Ref;

use crate::decoder::{DecodedInstruction, InstructionDecoder};

use crate::memory::{Memory, MemoryAccessWidth, MemoryOperations};
use crate::opcodes::{self, OpCodes};
use crate::optypes::InstructionFormat;
use elfloader::ElfLoader;
use quark::Signs;

#[derive(Copy, Clone, Debug)]
pub enum PrivMode {
    User,
    Supervisor,
    Machine,
}

pub type Register = u8;
pub type RegisterValue = u64;

type Registers = [RegisterValue; 32];
type CSRRegisters = [RegisterValue; 4096];

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, FromPrimitive)]
#[repr(u16)]
pub enum CSRRegister {
    mvendorid = 0xf11,
    marchid = 0xf11 + 1,
    mimpid = 0xf11 + 2,
    mhartid = 0xf11 + 3,
    mconfigptr = 0xf11 + 4,
    mstatus = 0x300,
    misa = 0x300 + 1,
    medeleg = 0x300 + 2,
    mideleg = 0x300 + 3,
    mie = 0x300 + 4,
    mtvec = 0x300 + 5,
    mcounteren = 0x300 + 6,
    mstatush = 0x300 + 7,

    mcycle = 0xb00,
    minstret = 0xb02,

    minstreth = 0xb82, // RV32 only
}

#[derive(Debug)]
pub enum Stage {
    TRAP,
    FETCH,
    DECODE,
    EXECUTE,
    MEMORY,
    WRITEBACK,
}

#[derive(Debug)]
pub struct Core<'a> {
    pub id: u64,
    pub registers: Registers,
    pub csrs: CSRRegisters,
    pub pmode: PrivMode,
    pub memory: &'a mut dyn MemoryOperations,
    pub pc: u64,
    pub prev_pc: u64,
    pub cycles: u64,
    pub pipe: Pipeline,
}

#[derive(Debug)]
pub struct Pipeline {
    pub stage: Stage,
    pub instruction: (bool, u32),
    pub decoded: Option<DecodedInstruction>,
    pub memory_access_width: MemoryAccessWidth, // FIXME: Default?
    pub reg_out: Option<(Register, u64)>,
    pub rs2v: u64,
}

impl<'a> Core<'a> {
    pub fn create(id: u64, memory: &'a mut impl MemoryOperations) -> Core<'a> {
        let registers: [RegisterValue; 32] = [0; 32];
        let mut csrs: [RegisterValue; 4096] = [0; 4096];
        csrs[CSRRegister::mhartid as usize] = id;

        Core {
            id,
            registers,
            csrs,
            pmode: PrivMode::Machine,
            memory,
            pc: 0,
            prev_pc: 0,
            cycles: 0,
            pipe: Pipeline {
                decoded: None,
                instruction: (false, 0),
                stage: Stage::FETCH,
                reg_out: None,
                rs2v: 0,
                memory_access_width: MemoryAccessWidth::WORD,
            },
        }
    }

    pub fn reset(&mut self, pc: u64) {
        self.pc = pc;
        self.pipe.stage = Stage::FETCH;
        self.pipe.decoded = None;
    }

    pub fn pc(&self) -> u64 {
        self.pc
    }

    pub fn set_pc(&mut self, pc: u64) {
        self.pc = pc;
    }

    pub fn pmode(&self) -> PrivMode {
        self.pmode
    }

    pub fn read_register(&self, reg: Register) -> RegisterValue {
        match reg {
            0 => 0,
            _ => self.registers[reg as usize],
        }
    }

    pub fn write_register(&mut self, reg: Register, value: RegisterValue) {
        if reg != 0 {
            self.registers[reg as usize] = value;
        } else {
            panic!("Should never write to register x0")
        }
    }

    fn uncompress_instruction(&self, ci: u16) -> u32 {
        ci as u32
    }

    fn fetch(&mut self) -> Stage {
        let instruction = self
            .memory
            .read_single(self.pc, MemoryAccessWidth::WORD)
            .unwrap() as u32;

        println!("fetch: instruction @ {:#x}: {:#x}", self.pc, instruction);

        // Store prev_pc, as we might step a HALFWORD if instruction is compressed
        self.prev_pc = self.pc;

        // Determine if instruction is compressed
        self.pipe.instruction = match (instruction & 0x3) == 0x03 {
            true => {
                self.pc += 4;
                (false, instruction)
            }
            false => {
                self.pc += 2;
                println!("fetch: compressed instruction {:#x?}", instruction & 0xffff);
                (true, instruction & 0xffff)
            }
        };
        Stage::DECODE
    }

    pub fn decode(&mut self) -> Stage {
        self.pipe.decoded = Some(self.decode_instruction(self.pipe.instruction));
        println!("decoded: {:?}", self.pipe.decoded);
        Stage::EXECUTE
    }

    fn execute(&mut self) -> Stage {
        let decoded = self.pipe.decoded.as_ref().unwrap();
        let opcode = decoded.opcode.unwrap_or(OpCodes::ILLEGAL);
        match opcode {
            OpCodes::CSRRS => {
                self.pipe.reg_out = Some((decoded.rd, self.csrs[decoded.imm12 as usize]));
                if decoded.rs1 != 0 {
                    self.csrs[decoded.imm12 as usize] |= self.read_register(decoded.rs1);
                }
            }
            OpCodes::CSRRW => {
                if (decoded.rd != 0) {
                    self.pipe.reg_out = Some((decoded.rd, self.csrs[decoded.imm12 as usize]));
                }
                self.csrs[decoded.imm12 as usize] = self.read_register(decoded.rs1);
            }
            OpCodes::LUI => {
                self.pipe.reg_out = Some((decoded.rd, (decoded.imm20 << 12) as u64));
            }
            OpCodes::AUIPC => {
                const M: u32 = 1 << (20 - 1);
                let se_imm20 = (decoded.imm20 ^ M) - M;
                self.pipe.reg_out = Some((decoded.rd, (se_imm20 << 12) as u64 + self.prev_pc));
            }

            OpCodes::ADDI => {
                // @TODO: Sign extend imm12?
                let seimm12 = (decoded.imm12 as i64).sign_extend(64 - 12);
                self.pipe.reg_out = Some((
                    decoded.rd,
                    (self.read_register(decoded.rs1) as i64).wrapping_add(seimm12) as u64,
                ));
            }
            OpCodes::MUL => {
                let rs1v = self.read_register(decoded.rs1);
                let rs2v = self.read_register(decoded.rs2);
                let rd = rs1v.checked_mul(rs2v);
                self.pipe.reg_out = Some((decoded.rd, rd.unwrap()))
                // Will panic if overflow
            }
            OpCodes::ADD => {
                self.pipe.reg_out = Some((
                    decoded.rd,
                    self.read_register(decoded.rs1)
                        .wrapping_add(self.read_register(decoded.rs2)),
                ))
            }
            OpCodes::JAL => {
                self.pipe.reg_out = Some((decoded.rd, self.pc));
                self.pc = self.prev_pc + ((decoded.imm20 as u64) << 1 as u64);
                //panic!("JAL {:#x} => {:#x}", decoded.imm20, self.pc);
            }
            OpCodes::SD => {
                self.pipe.rs2v = self.read_register(decoded.rs2);
                self.pipe.memory_access_width = MemoryAccessWidth::WORD;
                //todo!();
            }
            OpCodes::AND => {
                self.pipe.reg_out = Some((
                    decoded.rd,
                    self.read_register(decoded.rs1) & self.read_register(decoded.rs2),
                ));
            }
            OpCodes::JALR => {
                self.pipe.reg_out = Some((decoded.rd, self.pc));
                let rs1v = self.read_register(decoded.rs1);
                let rel = (decoded.imm12 as i64).sign_extend(64 - 12);
                self.pc = (rs1v as i64 + rel) as u64;
            }
            OpCodes::ADDIW => {
                let seimm12 = (decoded.imm12 as i64).sign_extend(64 - 5);
                self.pipe.reg_out = Some((
                    decoded.rd,
                    (self.read_register(decoded.rs1) as i64).wrapping_add(seimm12) as u64,
                ));
                //                todo!()
            }
            OpCodes::ILLEGAL => panic!("ILLEGAL INSTRUCTION {:?}", opcode),
            _ => panic!("UNKNOWN INSTRUCTION {:?}", opcode),
        }
        Stage::MEMORY
    }

    fn memory(&mut self) -> Stage {
        let decoded = self.pipe.decoded.as_ref().unwrap();
        //let memory = self.pipe.decoded.as_ref().unwrap();
        match decoded.mem_offset {
            Some(offset) => {
                match decoded.optype {
                    // STORE
                    InstructionFormat::S => {
                        self.memory.write_single(
                            (offset + self.memory.get_base_address()),
                            self.pipe.rs2v,
                            self.pipe.memory_access_width,
                        );
                    }
                    InstructionFormat::CSS => {
                        self.memory.write_single(
                            (/*self.read_register(decoded.rs1 as u64)
                            + */offset + self.memory.get_base_address()),
                            self.pipe.rs2v,
                            self.pipe.memory_access_width,
                        );
                    }
                    // LOAD
                    InstructionFormat::I => {
                        todo!()
                    }
                    _ => panic!(),
                }
            }
            None => {}
        }
        Stage::WRITEBACK
    }

    fn writeback(&mut self) -> Stage {
        match self.pipe.reg_out {
            Some((register, value)) => self.write_register(register, value),
            None => {
                panic!()
            }
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

    fn trap(&self) -> Stage {
        Stage::FETCH
    }

    pub fn cycle(&mut self) {
        self.pipe.stage = match self.pipe.stage {
            Stage::TRAP => self.trap(),
            Stage::FETCH => self.fetch(),
            Stage::DECODE => self.decode(),
            Stage::EXECUTE => self.execute(),
            Stage::MEMORY => self.memory(),
            Stage::WRITEBACK => self.writeback(),
        };
        self.cycles = self.cycles.wrapping_add(1);
    }
}
