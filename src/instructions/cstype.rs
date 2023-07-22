use std::fmt::Display;

use crate::{
    cpu::{Core, Register},
    pipeline::{MemoryAccess, Stage, WritebackStage},
};

use super::{
    functions::{C0_Funct3, Funct3},
    opcodes::CompressedOpcode,
    CompressedFormatDecoder, CompressedFormatType, ImmediateDecoder, Instruction,
    InstructionExcecutor, InstructionSelector,
};

/// This instruction format is shared between C0 and C1 ops, hence it
/// has both funct, funct3, and funct6 decoded.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CStype {
    pub opcode: CompressedOpcode,
    pub rs1_rd: Register,
    pub rs2: Register,
    pub offset: u8,
    pub shamt: u8,
    pub funct2: u8,
    pub funct6: u8,
    pub funct3: Funct3,
}

impl ImmediateDecoder<u16, u8> for CStype {
    fn decode_immediate(i: u16) -> u8 {
        (((i >> 7) & 0x38) | ((i << 1) & 0xc0)) as u8
    }
}

impl CompressedFormatType for CStype {}
impl CompressedFormatDecoder<CStype> for CStype {
    fn decode(word: u16) -> CStype {
        CStype {
            opcode: num::FromPrimitive::from_u8((word & 3) as u8).unwrap(),
            rs1_rd: ((word >> 7) & 7) as u8 + 8,
            rs2: ((word >> 2) & 7) as u8 + 8,
            shamt: (((word >> 7) & 0b100000) | ((word >> 2) & 0x1f)) as u8,
            offset: CStype::decode_immediate(word),
            funct2: (word >> 5) as u8 & 0x3,
            funct6: (word >> 10) as u8 & 0b111111,
            funct3: num::FromPrimitive::from_u8(((word >> 13) & 0x7) as u8).unwrap(),
        }
    }
}

impl Display for Instruction<CStype> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.args.is_some() {
            write!(f, "{}", self.mnemonic)
        } else {
            let args = self.args.unwrap();
            write!(f, "{} x{},x{}", self.mnemonic, args.rs1_rd, args.rs2)
        }
    }
}

#[allow(non_snake_case)]
impl Instruction<CStype> {
    pub fn C_AND(args: &CStype) -> Instruction<CStype> {
        Instruction {
            args: Some(*args),
            mnemonic: "C.AND",
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1_rd);
                let rs2v = core.read_register(args.rs2);
                let value = rs1v & rs2v;
                //                debug_trace!(println!("C.AND x{},x{}", args.rs1_rd, args.rs2));
                Stage::writeback(args.rs1_rd, value)
            },
        }
    }

    pub fn C_OR(args: &CStype) -> Instruction<CStype> {
        Instruction {
            args: Some(*args),
            mnemonic: "C.OR",
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1_rd);
                let rs2v = core.read_register(args.rs2);
                let value = rs1v | rs2v;
                //                debug_trace!(println!("C.OR x{},x{}", args.rs1_rd, args.rs2));
                Stage::writeback(args.rs1_rd, value)
            },
        }
    }

    pub fn C_SRLI(args: &CStype) -> Instruction<CStype> {
        Instruction {
            args: Some(*args),
            mnemonic: &"C.SRLI",
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1_rd);
                let value = rs1v >> args.shamt;
                Stage::writeback(args.rs1_rd, value)
            },
        }
    }

    pub fn C_SRAI(args: &CStype) -> Instruction<CStype> {
        Instruction {
            args: Some(*args),
            mnemonic: &"C.SRAI",
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1_rd);
                let rs2v = core.read_register(args.rs2);
                let value = todo!();
                Stage::writeback(args.rs1_rd, value)
            },
        }
    }

    pub fn C_SD(args: &CStype) -> Instruction<CStype> {
        Instruction {
            args: Some(*args),
            mnemonic: &"C.SD",
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1_rd);
                let rs2v = core.read_register(args.rs2);
                let addr = rs1v + (args.offset as u64);

                Stage::MEMORY(MemoryAccess::WRITE64(addr, rs2v))
            },
        }
    }
}

impl InstructionSelector<CStype> for CStype {
    fn select(&self, _xlen: crate::cpu::Xlen) -> Instruction<CStype> {
        match self.opcode {
            CompressedOpcode::C0 => match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
                C0_Funct3::C_SD => Instruction::C_SD(self),
                _ => panic!(),
            },
            CompressedOpcode::C1 => match self.funct6 {
                0b100011 => match self.funct2 {
                    0b00 => todo!(),
                    0b01 => todo!(),
                    0b10 => Instruction::C_OR(self),
                    0b11 => Instruction::C_AND(self),
                    _ => panic!(),
                },
                0b100000 => Instruction::C_SRLI(self),
                0b100001 => Instruction::C_SRAI(self),

                _ => panic!(),
            },
            _ => panic!(),
        }
    }
}

impl InstructionExcecutor for Instruction<CStype> {
    fn run(&self, core: &mut Core) -> Stage {
        debug_trace!(println!("{}", self.to_string()));
        (self.funct)(core, &self.args.unwrap())
    }
}
