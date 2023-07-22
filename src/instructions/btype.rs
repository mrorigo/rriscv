use std::fmt::Display;

use quark::Signs;

use crate::{
    cpu::{Core, Register},
    pipeline::Stage,
};

use super::{
    functions::BRANCH_Funct3, opcodes::MajorOpcode, FormatDecoder, ImmediateDecoder, Instruction,
    InstructionExcecutor, InstructionFormatType, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Btype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub rs1: Register,
    pub rs2: Register,
    pub funct3: u8,
    pub imm12: u16,
}

impl InstructionFormatType for Btype {}

impl FormatDecoder<Btype> for Btype {
    fn decode(word: u32) -> Btype {
        Btype {
            opcode: num::FromPrimitive::from_u8((word & 0x7f) as u8).unwrap(),
            rd: ((word >> 7) & 31) as Register,
            rs1: ((word >> 15) & 31) as Register,
            rs2: ((word >> 20) & 31) as Register,
            imm12: Btype::decode_immediate(word),
            funct3: ((word >> 12) & 7) as u8,
        }
    }
}

impl ImmediateDecoder<u32, u16> for Btype {
    fn decode_immediate(i: u32) -> u16 {
        let imm12 = ((i >> 31) & 1) as u16;
        let imm105 = ((i >> 25) & 0b111111) as u16;
        let imm41 = ((i >> 8) & 0xf) as u16;
        let imm11 = ((i >> 7) & 1) as u16;
        (imm12 << 12) | (imm105 << 5) | (imm41 << 1) | (imm11 << 11)
    }
}

#[allow(non_snake_case)]
impl Instruction<Btype> {
    pub fn BNE(args: &Btype) -> Instruction<Btype> {
        Instruction {
            args: Some(*args),
            mnemonic: "BNE",
            funct: |core, args| {
                let rs1v = core.bit_extend(core.read_register(args.rs1) as i64) as u64;
                let rs2v = core.bit_extend(core.read_register(args.rs2) as i64) as u64;
                match rs1v == rs2v {
                    false => {
                        let target = core
                            .prev_pc
                            .wrapping_add(args.imm12.sign_extend(64 - 13) as u64);
                        core.set_pc(target)
                    }
                    _ => {}
                }
                Stage::WRITEBACK(None)
            },
        }
    }
    pub fn BEQ(args: &Btype) -> Instruction<Btype> {
        Instruction {
            args: Some(*args),
            mnemonic: "BNE",
            funct: |core, args| {
                let rs1v = core.bit_extend(core.read_register(args.rs1) as i64) as u64;
                let rs2v = core.bit_extend(core.read_register(args.rs2) as i64) as u64;
                match rs1v == rs2v {
                    true => {
                        let target = core
                            .prev_pc
                            .wrapping_add(args.imm12.sign_extend(64 - 13) as u64);
                        core.set_pc(target)
                    }
                    _ => {}
                }
                Stage::WRITEBACK(None)
            },
        }
    }
}

impl Display for Instruction<Btype> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.args.is_some() {
            write!(f, "{}", self.mnemonic)
        } else {
            let args = self.args.unwrap();
            write!(
                f,
                "{} x{},x{},{}",
                self.mnemonic, args.rs1, args.rs1, args.imm12
            )
        }
    }
}

impl InstructionSelector<Btype> for Btype {
    fn select(&self, _xlen: crate::cpu::Xlen) -> Instruction<Btype> {
        match self.opcode {
            MajorOpcode::BRANCH => match num::FromPrimitive::from_u8(self.funct3).unwrap() {
                BRANCH_Funct3::BNE => Instruction::BNE(self),
                BRANCH_Funct3::BEQ => Instruction::BEQ(self),
                _ => todo!("Branch instruction"),
            },
            _ => panic!("No such opcode for Btype instruction: {:#?}", self.opcode),
        }
    }
}

impl InstructionExcecutor for Instruction<Btype> {
    fn run(&self, core: &mut Core) -> Stage {
        instruction_trace!(println!("EXEC: {}", self.to_string()));
        (self.funct)(core, &self.args.unwrap())
    }
}
