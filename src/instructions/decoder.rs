//! Each instruction format type is represented by a separate struct, passing the parameters for a specifi
//! instruction inside a DecodedInstruction. Each instruction type implements its own ExecutionStage, where
//! actual execution of instructions take place.
//!
//! Compressed instructions are handled the same way as regular instructions, which might be a downside
//! of this implementation, but keeps this relatively simple, though the compressed instruction decoding
//! takes one or several more extra match arms.

use crate::{
    cpu::Core,
    instructions::{functions::C0_Funct3, map::FORMAT_MAP, CompressedFormat, InstructionFormat},
    pipeline::RawInstruction,
};

use super::{
    btype::Btype,
    cbtype::CBtype,
    citype::CItype,
    ciwtype::CIWtype,
    cjtype::CJtype,
    cltype::CLtype,
    crtype::CRtype,
    csstype::CSStype,
    cstype::CStype,
    functions::{C1_Funct3, C2_Funct3},
    itype::Itype,
    jtype::Jtype,
    rtype::Rtype,
    stype::Stype,
    utype::Utype,
    CompressedFormatDecoder, FormatDecoder,
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

/// Decodes a RawInstruction into a DecodedInstruction variant, which holds the type-
/// specific parameters for the decoded instruction.
pub trait InstructionDecoder {
    fn decode_instruction(&self, instruction: RawInstruction) -> DecodedInstruction;
}

impl InstructionDecoder for Core {
    fn decode_instruction(&self, instruction: RawInstruction) -> DecodedInstruction {
        let word = instruction.word;
        match instruction.compressed {
            false => {
                let opcode_idx = (word & 0x7f) as usize;
                //debug_trace!(println!("opcode_idx: {}", opcode_idx));
                match FORMAT_MAP[opcode_idx] {
                    InstructionFormat::R => DecodedInstruction::R(Rtype::decode(word)),
                    InstructionFormat::U => DecodedInstruction::U(Utype::decode(word)),
                    InstructionFormat::S => DecodedInstruction::S(Stype::decode(word)),
                    InstructionFormat::B => DecodedInstruction::B(Btype::decode(word)),
                    InstructionFormat::I => DecodedInstruction::I(Itype::decode(word)),
                    InstructionFormat::J => DecodedInstruction::J(Jtype::decode(word)),
                    _ => panic!(
                        "invalid format {:?} ({})",
                        FORMAT_MAP[opcode_idx], opcode_idx
                    ),
                }
            }

            true => {
                let funct3 = ((word >> 13) & (0x7)) as u8; // bits 2,3,4
                let format = match word & 3 {
                    0 => {
                        // Quadrant 0
                        match num::FromPrimitive::from_u8(funct3).unwrap() {
                            C0_Funct3::C_LD => CompressedFormat::CI,
                            C0_Funct3::C_ADDI4SPN => CompressedFormat::CIW,
                            C0_Funct3::C_LQ => CompressedFormat::CL,
                            C0_Funct3::C_LW => CompressedFormat::CL,
                            C0_Funct3::C_SQ => CompressedFormat::CS,
                            C0_Funct3::C_SW => CompressedFormat::CS,
                            C0_Funct3::C_SD => CompressedFormat::CS,
                        }
                    }
                    1 => match num::FromPrimitive::from_u8(funct3).unwrap() {
                        C1_Funct3::C_ANDI => CompressedFormat::CB,
                        C1_Funct3::C_BEQZ => CompressedFormat::CB,
                        C1_Funct3::C_BNEZ => CompressedFormat::CB,
                        C1_Funct3::C_LI => CompressedFormat::CI,
                        C1_Funct3::C_LUI => CompressedFormat::CI,
                        C1_Funct3::C_ADDI => CompressedFormat::CI,
                        C1_Funct3::C_ADDIW => CompressedFormat::CI,
                        C1_Funct3::C_J => CompressedFormat::CJ,
                    },
                    2 => match num::FromPrimitive::from_u8(funct3).unwrap() {
                        C2_Funct3::C_LDSP => CompressedFormat::CI,
                        C2_Funct3::C_SLLI => CompressedFormat::CI,
                        C2_Funct3::C_SDSP => CompressedFormat::CSS,
                        _ => {
                            let rs2 = ((word >> 2) & 31) as u8;
                            let funct1 = ((word >> 12) & 1) as u8;
                            match funct1 {
                                0 => CompressedFormat::CR, // C.JR
                                _ => match rs2 {
                                    0 => todo!("C.EBREAK/C.JALR"),
                                    _ => CompressedFormat::CR, // C.ADD
                                },
                            }
                        }
                    },
                    _ => panic!("no more quadrants, sir"),
                };
                match format {
                    CompressedFormat::CSS => DecodedInstruction::CSS(CSStype::decode(word as u16)),
                    CompressedFormat::CB => DecodedInstruction::CB(CBtype::decode(word as u16)),
                    CompressedFormat::CS => DecodedInstruction::CS(CStype::decode(word as u16)),
                    CompressedFormat::CI => DecodedInstruction::CI(CItype::decode(word as u16)),
                    CompressedFormat::CR => DecodedInstruction::CR(CRtype::decode(word as u16)),
                    CompressedFormat::CIW => DecodedInstruction::CIW(CIWtype::decode(word as u16)),
                    CompressedFormat::CL => DecodedInstruction::CL(CLtype::decode(word as u16)),
                    CompressedFormat::CJ => DecodedInstruction::CJ(CJtype::decode(word as u16)),
                    _ => {
                        panic!("unknown compressed format {:#x?}", format)
                    }
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
