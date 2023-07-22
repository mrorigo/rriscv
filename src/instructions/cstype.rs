use std::fmt::Display;

use crate::{
    cpu::{Core, Register},
    pipeline::{Stage, WritebackStage},
};

use super::{
    functions::Funct3, opcodes::CompressedOpcode, CompressedFormatDecoder, CompressedFormatType,
    Instruction, InstructionExcecutor, InstructionSelector,
};

/// This instruction format is shared between C0 and C1 ops, hence it
/// has both funct, funct3, and funct6 decoded.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CStype {
    pub opcode: CompressedOpcode,
    pub rs1_rd: Register,
    pub rs2: Register,
    pub funct2: u8,
    pub funct6: u8,
    pub funct3: Funct3,
}

impl CompressedFormatType for CStype {}

impl CompressedFormatDecoder<CStype> for CStype {
    fn decode(word: u16) -> CStype {
        CStype {
            opcode: num::FromPrimitive::from_u8((word & 3) as u8).unwrap(),
            rs1_rd: ((word >> 7) & 3) as u8 + 8,
            rs2: ((word >> 2) & 7) as u8 + 8,
            funct2: (word as u8 >> 5) & 3,
            funct6: (word >> 10) as u8,
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
    pub fn C_AND(cstype: CStype) -> Instruction<CStype> {
        Instruction {
            args: Some(cstype),
            mnemonic: "C.AND",
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1_rd);
                let rs2v = core.read_register(args.rs2);
                let value = rs1v & rs2v;
                //                debug_trace!(println!("C.AND x{},x{}", args.rs1_rd, args.rs2));
                Stage::WRITEBACK(Some(WritebackStage {
                    register: args.rs1_rd,
                    value,
                }))
            },
        }
    }

    pub fn C_OR(cstype: CStype) -> Instruction<CStype> {
        Instruction {
            args: Some(cstype),
            mnemonic: "C.OR",
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1_rd);
                let rs2v = core.read_register(args.rs2);
                let value = rs1v | rs2v;
                //                debug_trace!(println!("C.OR x{},x{}", args.rs1_rd, args.rs2));
                Stage::WRITEBACK(Some(WritebackStage {
                    register: args.rs1_rd,
                    value,
                }))
            },
        }
    }
}

impl InstructionSelector<CStype> for CStype {
    fn select(&self, _xlen: crate::cpu::Xlen) -> Instruction<CStype> {
        match self.opcode {
            CompressedOpcode::C0 => todo!(),
            CompressedOpcode::C1 => match self.funct2 {
                0b00 => todo!(),
                0b01 => todo!(),
                0b10 => Instruction::C_OR(*self),
                // C.AND is the only instruction matching funct2=0b11, other matches are Reserved
                0b11 => Instruction::C_AND(*self),
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
