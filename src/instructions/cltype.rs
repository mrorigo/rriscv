use std::fmt::Display;

use crate::{cpu::Register, pipeline::Stage};

use super::{
    functions::{C0_Funct3, Funct3},
    opcodes::CompressedOpcode,
    CompressedFormatDecoder, CompressedFormatType, ImmediateDecoder, Instruction,
    InstructionFormatType, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CLtype {
    pub opcode: CompressedOpcode,
    pub rd: Register,
    pub rs1: Register,
    pub imm: u16,
    pub funct3: Funct3,
}

impl InstructionFormatType for CLtype {}
impl CompressedFormatType for CLtype {}

impl CompressedFormatDecoder<CLtype> for CLtype {
    fn decode(word: u16) -> CLtype {
        CLtype {
            opcode: num::FromPrimitive::from_u8((word & 3) as u8).unwrap(),
            rd: ((word >> 2) & 7) as Register + 8,
            rs1: ((word >> 7) & 7) as Register + 8,
            imm: CLtype::decode_immediate(word as u16),
            funct3: num::FromPrimitive::from_u8(((word >> 13) & 0x7) as u8).unwrap(),
        }
    }
}

impl ImmediateDecoder<u16, u16> for CLtype {
    fn decode_immediate(i: u16) -> u16 {
        ((i >> 7) & 0x38) | // offset[5:3] <= [12:10]
        ((i >> 4) & 0x4) | // offset[2] <= [6]
        ((i << 1) & 0x40) // offset[6] <= [5]
    }
}

#[allow(non_snake_case)]
impl Instruction<CLtype> {
    pub fn C_LW(args: &CLtype) -> Instruction<CLtype> {
        Instruction {
            mnemonic: "C.LW",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as i64;
                let addr = rs1v.wrapping_add(args.imm as i64) as u64;
                // instruction_trace!(println!(
                //     "C.LW: rs1v={:#x?} imm={:#x?} addr={:#x?}",
                //     rs1v, args.imm, addr
                // ));
                Stage::MEMORY(crate::pipeline::MemoryAccess::READ32(addr, args.rd, true))
            },
        }
    }
    pub fn C_LD(args: &CLtype) -> Instruction<CLtype> {
        Instruction {
            mnemonic: "C.LD",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as i64;
                let addr = rs1v.wrapping_add(args.imm as i64) as u64;
                // instruction_trace!(println!(
                //     "C.LW: rs1v={:#x?} imm={:#x?} addr={:#x?}",
                //     rs1v, args.imm, addr
                // ));
                Stage::MEMORY(crate::pipeline::MemoryAccess::READ64(addr, args.rd, false))
            },
        }
    }
}

impl Display for Instruction<CLtype> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.args.is_some() {
            write!(f, "{}", self.mnemonic)
        } else {
            let args = self.args.unwrap();
            write!(
                f,
                "{} x{},x{},{}",
                self.mnemonic, args.rd, args.rs1, args.imm
            )
        }
    }
}

impl InstructionSelector<CLtype> for CLtype {
    fn select(&self, _xlen: crate::cpu::Xlen) -> Instruction<CLtype> {
        match self.opcode {
            CompressedOpcode::C0 => match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
                C0_Funct3::C_LW => Instruction::C_LW(self),
                C0_Funct3::C_LD => Instruction::C_LD(self),
                _ => todo!(),
            },
            _ => panic!(),
        }
    }
}
