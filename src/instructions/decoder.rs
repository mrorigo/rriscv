//! Each instruction format type is represented by a separate struct, passing the parameters for a specifi
//! instruction inside a DecodedInstruction. Each instruction type implements its own ExecutionStage, where
//! actual execution of instructions take place.

use crate::{
    cpu::{Core, Register},
    instructions::{
        map::{COMPRESSED_FORMAT_MAP, FORMAT_MAP},
        CompressedInstructionFormat, InstructionFormat,
    },
    pipeline::RawInstruction,
};

use super::{
    btype::Btype, cbtype::CBtype, citype::CItype, ciwtype::CIWtype, cjtype::CJtype, cltype::CLtype,
    crtype::CRtype, csstype::CSStype, cstype::CStype, itype::Itype, jtype::Jtype, rtype::Rtype,
    stype::Stype, utype::Utype, ImmediateDecoder,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DecodedInstruction {
    I(Itype),
    J(Jtype),
    B(Btype),
    S(Stype),
    U(Utype),
    R(Rtype),
    CR(CRtype),
    CI(CItype),
    CSS(CSStype),
    CIW(CIWtype),
    CL(CLtype),
    CS(CStype),
    CB(CBtype),
    CJ(CJtype),
}

/// Decodes a RawInstruction into a DecodedInstruction, which holds type-
/// specific parameters for the decoded instruction.
pub trait InstructionDecoder {
    fn decode_instruction(&self, instruction: RawInstruction) -> DecodedInstruction;
}

impl InstructionDecoder for Core<'_> {
    fn decode_instruction(&self, instruction: RawInstruction) -> DecodedInstruction {
        let word = instruction.word;
        match instruction.compressed {
            false => {
                let opcode = num::FromPrimitive::from_u8((word & 0x7f) as u8).unwrap();
                let funct3 = ((word >> 12) & 7) as u8;
                let funct7 = ((word >> 25) & 0x7f) as u8;

                match FORMAT_MAP[opcode as usize] {
                    InstructionFormat::R => DecodedInstruction::R(Rtype {
                        opcode: opcode,
                        rd: ((word >> 7) & 31) as Register,
                        rs1: ((word >> 15) & 31) as Register,
                        rs2: ((word >> 20) & 31) as Register,
                        funct3,
                        funct7,
                    }),
                    InstructionFormat::U => DecodedInstruction::U(Utype {
                        opcode: opcode,
                        rd: ((word >> 7) & 31) as Register,
                        imm20: (word >> 12),
                    }),
                    InstructionFormat::S => DecodedInstruction::S(Stype {
                        opcode: opcode,
                        rs1: ((word >> 15) & 31) as Register,
                        rs2: ((word >> 20) & 31) as Register,
                        imm12: Stype::decode_immediate(word),
                        funct3,
                    }),
                    InstructionFormat::B => DecodedInstruction::B(Btype {
                        opcode: opcode,
                        rd: ((word >> 7) & 31) as Register,
                        rs1: ((word >> 15) & 31) as Register,
                        rs2: ((word >> 20) & 31) as Register,
                        imm12: Btype::decode_immediate(word),
                        funct3,
                    }),
                    InstructionFormat::I => DecodedInstruction::I(Itype {
                        opcode: opcode,
                        rd: ((word >> 7) & 31) as Register,
                        rs1: ((word >> 15) & 31) as Register,
                        imm12: (word >> 20) as u16,
                        funct3,
                    }),
                    InstructionFormat::J => DecodedInstruction::J(Jtype {
                        opcode: opcode,
                        rd: ((word >> 7) & 31) as Register,
                        imm20: Jtype::decode_immediate(word),
                    }),
                    _ => panic!("invalid format {:?}", FORMAT_MAP[opcode as usize]),
                }
            }
            true => {
                let opcode = num::FromPrimitive::from_u8((word & 0x3) as u8).unwrap();
                let funct3 = ((word >> 13) & 0x7) as u8;
                let cop = ((opcode as usize) & 3) as usize | (funct3 << 2) as usize;
                match COMPRESSED_FORMAT_MAP[cop as usize] {
                    CompressedInstructionFormat::CSS => DecodedInstruction::CSS(CSStype {
                        opcode,
                        uimm: CSStype::decode_immediate(word as u16),
                        funct3: funct3,
                        rs2: (word >> 2) as u8 & 31,
                    }),
                    CompressedInstructionFormat::CB => DecodedInstruction::CB(CBtype {
                        opcode,
                        rs1: 8 + ((word >> 7) & 3) as u8,
                        offset: CBtype::decode_immediate(word as u16),
                        funct3,
                    }),
                    CompressedInstructionFormat::CS => DecodedInstruction::CS(CStype {
                        opcode,
                        rs1: ((word >> 7) & 3) as u8 + 8,
                        rs2: ((word >> 2) & 7) as u8 + 8,
                        funct: (word as u8 >> 5) & 3,
                        funct6: (word >> 10) as u8,
                    }),
                    CompressedInstructionFormat::CI => DecodedInstruction::CI(CItype {
                        opcode,
                        rd: ((word >> 7) & 31) as u8,
                        imm: CItype::decode_immediate(word as u16),
                        funct3,
                    }),
                    CompressedInstructionFormat::CR => DecodedInstruction::CR(CRtype {
                        opcode,
                        rs2: ((word >> 2) & 31) as Register,
                        rs1: (word >> 7 & 31) as Register,
                        funct1: (word >> 12) as u8 & 1,
                        funct3,
                    }),
                    CompressedInstructionFormat::CIW => DecodedInstruction::CIW(CIWtype {
                        opcode,
                        imm: (word >> 5) as u16,
                        rd: ((word >> 2) & 7) as Register + 8,
                        funct3,
                    }),
                    CompressedInstructionFormat::CL => DecodedInstruction::CL(CLtype {
                        opcode,
                        rd: ((word >> 2) & 7) as Register + 8,
                        rs1: ((word >> 7) & 31) as Register + 8,
                        imm: CLtype::decode_immediate(word as u16),
                    }),
                    CompressedInstructionFormat::CJ => todo!(),
                    CompressedInstructionFormat::Unknown => panic!(),
                    // _ => panic!(
                    //     "invalid format for {:?} {:?}",
                    //     opcode,
                    //     crate::instruction_format::COMPRESSED_FORMAT_MAP[opcode as usize]
                    // ),
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum OpImmFunct3 {
    ADDI = 0b000,
    SLTI = 0b010,
    SLTIU = 0b011,
    XORI = 0b100,
    ORI = 0b110,
    ANDI = 0b111,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, FromPrimitive)]
pub enum CSR_Funct3 {
    CSRRW = 0b001,
    CSRRS = 0b010,
    CSRRC = 0b011,
    CSRRWI = 0b101,
    CSRRSI = 0b110,
    CSRRCI = 0b111,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, FromPrimitive)]
pub enum RV32M_Funct3 {
    MUL = 0b000,
    MULH = 0b001,
    MULHSU = 0b010,
    MULSHU = 0b011,
    DIV = 0b100,
    DIVU = 0b101,
    REM = 0b110,
    REMU = 0b111,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, FromPrimitive)]
pub enum C1_Funct3 {
    C_LUI = 0b011,
    C_LI = 0b010,
    C_ADDI = 0b000,
}
