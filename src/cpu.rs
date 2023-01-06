use std::cell::Ref;

use crate::memory::{Memory, MemoryAccessWidth, MemoryOperations};
use crate::opcodes::{self, OpCodes};
use crate::optypes::OpType;

#[derive(Copy, Clone, Debug)]
pub enum PrivMode {
    User,
    Supervisor,
    Machine,
}

pub type Register = u64;

type Registers = [Register; 32];
type CSRRegisters = [Register; 4096];

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
}

#[derive(Debug)]
pub struct DecodedInstruction {
    optype: OpType,
    opcode: OpCodes,
    rs1: u8,
    rs2: u8,
    rd: u8,
    funct7: u8,
    funct3: u8,
    shamt: u8,
    imm5: u8,
    imm12: u16,
    imm20: u32,
    memory_access_width: MemoryAccessWidth, // FIXME: Default?
    mem_read: bool,
    mem_write: bool,
    mem_offset: u64,
    //    jump_offset: Option<u64>,
}

#[derive(Debug)]
enum Stage {
    TRAP,
    FETCH,
    DECODE,
    EXECUTE,
    MEMORY,
    WRITEBACK,
}

#[derive(Debug)]
pub struct Core<'a> {
    id: u64,
    stage: Stage,
    instruction: (bool, u32),
    decoded: Option<DecodedInstruction>,
    reg_out: Option<(Register, u64)>,
    registers: Registers,
    csrs: CSRRegisters,
    pmode: PrivMode,
    memory: &'a dyn MemoryOperations,
    pc: u64,
    cycles: u64,
}

impl<'a> Core<'a> {
    pub fn create(id: u64, memory: &'a impl MemoryOperations) -> Core<'a> {
        let registers: [Register; 32] = [0; 32];
        let mut csrs: [Register; 4096] = [0; 4096];
        csrs[CSRRegister::mhartid as usize] = id;

        Core {
            id,
            decoded: None,
            instruction: (false, 0),
            registers,
            csrs,
            pmode: PrivMode::Machine,
            stage: Stage::FETCH,
            memory,
            pc: 0,
            cycles: 0,
            reg_out: None,
        }
    }

    pub fn reset(&mut self, pc: u64) {
        self.pc = pc;
        self.stage = Stage::FETCH;
        self.decoded = None;
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

    pub fn read_register(&self, reg: usize) -> u64 {
        match reg {
            0 => 0,
            _ => self.registers[reg],
        }
    }

    pub fn write_register(&mut self, reg: usize, value: u64) {
        if reg != 0 {
            self.registers[reg] = value;
        } else {
            panic!("Should never write to register x0")
        }
    }

    // pub fn registers(&self) -> &Registers {
    //     &self.registers
    // }

    // pub fn registers_mut(&mut self) -> &mut Registers {
    //     &mut self.registers
    // }

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

        println!("Read instruction @ {:#x}: {:#x}", self.pc, instruction);

        self.instruction = match b01 == 0x03 {
            true => {
                self.pc += 4;
                (false, instruction)
            }
            false => {
                self.pc += 2;
                println!("compressed instruction {:#x?}", instruction & 0xffff);
                (true, instruction & 0xffff)
            }
        };
        Stage::DECODE
    }

    fn decode(&mut self) -> Stage {
        let i = self.instruction.1;

        let rs1 = ((i >> 15) & 31) as u8;
        let rs2 = ((i >> 20) & 31) as u8;
        let funct7 = ((i >> 25) & 7) as u8;
        let mut funct3 = ((i >> 12) & 7) as u8;
        let shamt = ((i >> 20) & 31) as u8;
        let imm5 = ((i >> 7) & 31) as u8; // STORE only
        let mem_offset: u64 = 0;

        let mut rd = ((i >> 7) & 31) as u8;
        let mut imm12 = 0;
        let mut imm20: u32 = 0;

        let memory_access_width = MemoryAccessWidth::WORD;

        println!("decoding instruction: {:#x?}", (i & 0x7f));
        // Determine opcode type, decode immediates, and set up `op` for lookup
        // in OpCodes by combining correct bits from raw opcode.
        //        println!("ci: {:?} compressed: {:?}", ci, self.instruction.0);
        let optype = match self.instruction.0 {
            false => crate::optypes::OPTYPES[(i & 0x7f) as usize],
            true => {
                funct3 = ((i >> 13) & 0x7) as u8;
                let ci = (i & 3) | (funct3 << 2) as u32;
                crate::optypes::COMPRESSED_OPTYPES[ci as usize]
            }
        };

        let op = match optype {
            OpType::CI => {
                let nzimm1612 = (i >> 2) & 31;
                let nzimm17 = (i >> 12) & 1;
                imm20 = nzimm1612 | (nzimm17 << 5); // @TODO: Sign extend imm20
                rd = (i >> 7 & 31) as u8;
                OpCodes::LUI as u32
            }
            OpType::Unknown => panic!(),
            OpType::R => {
                (i & 0x7f) as u32
                    | (funct3 << 7) as u32
                    | ((funct7 as u32) << opcodes::FUNCT7_SHIFT) as u32
            }
            OpType::I => {
                imm12 = ((i >> 20) & ((1 << 12) - 1)) as u16;
                (i & 0x7f) as u32 | (funct3 << 7) as u32
            }
            OpType::S => {
                imm12 = (((i >> 7) & 0b11111) | ((i >> 20) & 0xffffe0)) as u16;
                imm20 = imm12 as u32 | imm5 as u32;
                (i & 0x7f) as u32 | (funct3 << 7) as u32
            }
            OpType::B => {
                imm12 = ((i >> 31) & 1) as u16;
                let imm105 = ((i >> 25) & 0b111111) as u16;
                let imm41 = ((i >> 8) & 0xf) as u16;
                let imm11 = ((i >> 7) & 1) as u16;
                imm12 = (imm12 << 12) | (imm105 << 5) | (imm41 << 1) | (imm11 << 11);
                (i & 0x7f) as u32 | (funct3 << 7) as u32

                //imm12 is only 12 bits in struct, but we have 13 for branches, so to preserve
                //top bit, we need to shift down once, and then account for that when
                // calculating jumpTarget
                //                imm12 = bimm >> 1;
            }
            OpType::U => {
                imm20 = ((i >> 12) & 0xfffff) as u32;
                (i & 0x7f) as u32
            }
            OpType::J => {
                imm20 = ((i >> 31) & 0b1) as u32;
                let imm101 = ((i >> 21) & 0b1111111111) as u32;
                let imm11 = ((i >> 20) & 0b1) as u32;
                let imm1912 = ((i >> 12) & 0b11111111) as u32;

                let imm = (imm20 << 20) | (imm101 << 1) | (imm11 << 11) | (imm1912 << 12);
                imm20 = ((imm) << 11) >> 12;
                (i & 0x7f) as u32
            }
            OpType::C => {
                imm12 = (i >> 20) as u16;
                println!(
                    "OpType::C {:#x} funct3={} -> {:#x?}",
                    i & 0x7f,
                    funct3,
                    (i & 0x7f) as u32 | ((funct3 as u32) << 7)
                );
                (i & 0x7f) as u32 | ((funct3 as u32) << 7)
            }
            _ => {
                todo!()
            }
        };
        assert!(op != 0);
        println!("op: {:#x?}", op);
        let opcode: OpCodes = num::FromPrimitive::from_u32(op).unwrap();
        println!("op: {:#x?}  opcode: {:?}", op, opcode);

        self.decoded = Some(DecodedInstruction {
            optype,
            opcode,
            rs1,
            rs2,
            rd,
            funct7,
            funct3,
            shamt,
            imm5,
            imm12,
            imm20,
            mem_read: false,
            mem_write: false,
            memory_access_width,
            mem_offset,
            //jump_offset,
        });
        println!("decoded: {:?}", self.decoded);
        Stage::EXECUTE
    }

    fn execute_r(&mut self) {}
    fn execute_i(&mut self) {
        let decoded = self.decoded.as_ref().unwrap();
        match decoded.opcode {
            OpCodes::ADDI => {
                // @TODO: Sign extend imm12?
                self.reg_out = Some((
                    decoded.rd as Register,
                    self.read_register(decoded.rs1 as usize)
                        .wrapping_add(decoded.imm12 as u64),
                ));
            }
            _ => todo!(),
        }
    }
    fn execute_s(&mut self) {}
    fn execute_b(&mut self) {}
    fn execute_u(&mut self) {
        let decoded = self.decoded.as_ref().unwrap();
        match decoded.opcode {
            OpCodes::LUI => {
                self.reg_out = Some((decoded.rd as Register, (decoded.imm20 << 12) as u64));
            }
            OpCodes::AUIPC => {
                const M: u32 = 1 << (20 - 1);
                let se_imm20 = (decoded.imm20 ^ M) - M;
                self.reg_out = Some((decoded.rd as Register, (se_imm20 << 12) as u64 + self.pc));
            }
            _ => panic!(),
        }
    }
    fn execute_j(&mut self) {}
    fn execute_c(&mut self) {
        let decoded = self.decoded.as_ref().unwrap();
        match decoded.opcode {
            OpCodes::CSRRS => {
                self.reg_out = Some((decoded.rd as Register, self.csrs[decoded.imm12 as usize]));
                self.csrs[decoded.imm12 as usize] = self.read_register(decoded.rs1 as usize);
            }
            _ => todo!(),
        }
    }

    fn execute(&mut self) -> Stage {
        let decoded = self.decoded.as_ref().unwrap();
        match decoded.optype {
            OpType::R => self.execute_r(),
            OpType::I => self.execute_i(),
            OpType::CI => self.execute_u(),
            OpType::S => self.execute_s(),
            OpType::B => self.execute_b(),
            OpType::U => self.execute_u(),
            OpType::J => self.execute_j(),
            OpType::C => self.execute_c(),
            _ => panic!("Unknown optype {:?}", decoded.optype),
        }
        Stage::MEMORY
    }

    fn memory(&self) -> Stage {
        Stage::WRITEBACK
    }

    fn writeback(&mut self) -> Stage {
        match self.reg_out {
            Some((register, value)) => self.write_register(register as usize, value),
            None => {
                panic!()
            }
        }
        Stage::FETCH
    }

    fn trap(&self) -> Stage {
        Stage::FETCH
    }

    pub fn cycle(&mut self) {
        self.stage = match self.stage {
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
