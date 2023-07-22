use std::cell::Ref;

use crate::decoder::{DecodedInstruction, InstructionDecoder};

use crate::memory::{Memory, MemoryAccessWidth, MemoryOperations};
use crate::opcodes::{self, OpCodes};
use crate::optypes::OpType;
use elfloader::ElfLoader;

#[derive(Copy, Clone, Debug)]
pub enum PrivMode {
    User,
    Supervisor,
    Machine,
}

pub type Register = u64;

type Registers = [Register; 32];
type CSRRegisters = [Register; 4096];

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
    pub memory: &'a dyn MemoryOperations,
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
    pub reg_out: Option<(Register, u64)>,
}

impl<'a> Core<'a> {
    pub fn create(id: u64, memory: &'a impl MemoryOperations) -> Core<'a> {
        let registers: [Register; 32] = [0; 32];
        let mut csrs: [Register; 4096] = [0; 4096];
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

    pub fn read_register(&self, reg: Register) -> u64 {
        match reg {
            0 => 0,
            _ => self.registers[reg as usize],
        }
    }

    pub fn write_register(&mut self, reg: usize, value: u64) {
        if reg != 0 {
            self.registers[reg] = value;
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
            .read_single(self.pc as usize, MemoryAccessWidth::WORD)
            .unwrap() as u32;

        //let b1513 = instruction >> 13 & 0x3;
        let b01 = instruction & 0x3;

        println!("fetch: instruction @ {:#x}: {:#x}", self.pc, instruction);

        self.prev_pc = self.pc;
        self.pipe.instruction = match b01 == 0x03 {
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
        match decoded.opcode.unwrap_or(OpCodes::ILLEGAL) {
            OpCodes::CSRRS => {
                self.pipe.reg_out =
                    Some((decoded.rd as Register, self.csrs[decoded.imm12 as usize]));
                if decoded.rs1 != 0 {
                    self.csrs[decoded.imm12 as usize] |=
                        self.read_register(decoded.rs1 as Register);
                }
            }
            OpCodes::LUI => {
                self.pipe.reg_out = Some((decoded.rd as Register, (decoded.imm20 << 12) as u64));
            }
            OpCodes::AUIPC => {
                const M: u32 = 1 << (20 - 1);
                let se_imm20 = (decoded.imm20 ^ M) - M;
                self.pipe.reg_out =
                    Some((decoded.rd as Register, (se_imm20 << 12) as u64 + self.pc));
            }
            OpCodes::ADDI => {
                // @TODO: Sign extend imm12?
                self.pipe.reg_out = Some((
                    decoded.rd as Register,
                    self.read_register(decoded.rs1 as Register)
                        .wrapping_add(decoded.imm12 as u64),
                ));
            }
            OpCodes::MUL => {
                let rs1v = self.read_register(decoded.rs1 as Register);
                let rs2v = self.read_register(decoded.rs2 as Register);
                let rd = rs1v.checked_mul(rs2v);
                self.pipe.reg_out = Some((decoded.rd as Register, rd.unwrap())) // Will panic if overflow
            }
            OpCodes::ADD => {
                self.pipe.reg_out = Some((
                    decoded.rd as Register,
                    self.read_register(decoded.rs1 as Register)
                        .wrapping_add(self.read_register(decoded.rs2 as Register)),
                ))
            }
            OpCodes::JAL => {
                self.pipe.reg_out = Some((decoded.rd as Register, self.pc));
                self.pc = self.prev_pc + ((decoded.imm20 as u64) << 1 as u64);
                //panic!("JAL {:#x} => {:#x}", decoded.imm20, self.pc);
            }
            OpCodes::SD => todo!(),
            OpCodes::ILLEGAL => panic!("ILLEGAL INSTRUCTION"),
            _ => panic!(),
        }
        Stage::MEMORY
    }

    fn memory(&self) -> Stage {
        Stage::WRITEBACK
    }

    fn writeback(&mut self) -> Stage {
        match self.pipe.reg_out {
            Some((register, value)) => self.write_register(register as usize, value),
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
