use crate::{
    cpu::{Core, Register, Xlen},
    pipeline::{Stage, WritebackStage},
};

use super::{
    opcodes::CompressedOpcode, CompressedFormatDecoder, CompressedFormatType, Instruction,
    InstructionExcecutor, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CRtype {
    pub opcode: CompressedOpcode,
    pub rs1: Register,
    pub rs2: Register,
    pub funct1: u8,
    pub funct3: u8,
}

impl CompressedFormatType for CRtype {}

impl CompressedFormatDecoder<CRtype> for CRtype {
    fn decode(word: u16) -> CRtype {
        CRtype {
            opcode: num::FromPrimitive::from_u8((word & 3) as u8).unwrap(),
            rs2: ((word >> 2) & 31) as Register,
            rs1: (word >> 7 & 31) as Register,
            funct1: (word >> 12) as u8 & 1,
            funct3: num::FromPrimitive::from_u8(((word >> 13) & 0x7) as u8).unwrap(),
        }
    }
}

#[allow(non_snake_case)]
impl Instruction<CRtype> {
    pub fn C_ADD(crtype: CRtype) -> Instruction<CRtype> {
        Instruction {
            args: Some(crtype),
            mnemonic: "C.ADD",
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1);
                let rs2v = core.read_register(args.rs2);
                Stage::WRITEBACK(Some(WritebackStage {
                    register: args.rs1,
                    value: core.bit_extend(rs1v.wrapping_add(rs2v) as i64) as u64,
                }))
            },
        }
    }
}

impl InstructionSelector<CRtype> for CRtype {
    fn select(&self, _xlen: Xlen) -> Instruction<CRtype> {
        match self.opcode {
            CompressedOpcode::C2 => {
                match self.funct1 {
                    // C.JR / C.MV
                    0 => match self.rs2 {
                        0 => todo!("C.JR"),
                        _ => todo!("C.MV"),
                    },
                    // C.EBREAK / C.JALR / C.ADD
                    1 => match self.rs2 {
                        0 => match self.rs1 {
                            0 => todo!("C.EBREAK"),
                            _ => todo!("C.JALR"),
                        },
                        _ => Instruction::C_ADD(*self),
                    },
                    _ => panic!(),
                }
            }
            _ => panic!(),
        }
    }
}

impl InstructionExcecutor for Instruction<CRtype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}
