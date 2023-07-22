//! Each instruction format type is represented by a separate struct, passing the parameters for a specifi
//! instruction inside a DecodedInstruction. Each instruction type implements its own ExecutionStage, where
//! actual execution of instructions take place.

use crate::{
    cpu::Core,
    instructions::{
        map::{COMPRESSED_FORMAT_MAP, FORMAT_MAP},
        CompressedFormat, InstructionFormat,
    },
    pipeline::RawInstruction,
};

use super::{
    btype::Btype, cbtype::CBtype, citype::CItype, ciwtype::CIWtype, cjtype::CJtype, cltype::CLtype,
    crtype::CRtype, csstype::CSStype, cstype::CStype, itype::Itype, jtype::Jtype, rtype::Rtype,
    stype::Stype, utype::Utype, CompressedFormatDecoder, FormatDecoder,
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

impl InstructionDecoder for Core<'_> {
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
                let funct3 = ((word >> 11) & (0x7 << 2)) as usize;
                let c_opcode_idx = ((word) & 3) as usize | funct3;
                match COMPRESSED_FORMAT_MAP[c_opcode_idx] {
                    CompressedFormat::CSS => DecodedInstruction::CSS(CSStype::decode(word as u16)),
                    CompressedFormat::CB => DecodedInstruction::CB(CBtype::decode(word as u16)),
                    CompressedFormat::CS => DecodedInstruction::CS(CStype::decode(word as u16)),
                    CompressedFormat::CI => DecodedInstruction::CI(CItype::decode(word as u16)),
                    CompressedFormat::CR => DecodedInstruction::CR(CRtype::decode(word as u16)),
                    CompressedFormat::CIW => DecodedInstruction::CIW(CIWtype::decode(word as u16)),
                    CompressedFormat::CL => DecodedInstruction::CL(CLtype::decode(word as u16)),
                    CompressedFormat::CJ => todo!(),
                    CompressedFormat::Unknown => {
                        panic!("{} is an unknown compressed opcode index", c_opcode_idx)
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
