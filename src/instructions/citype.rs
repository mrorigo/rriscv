use quark::Signs;

use crate::{
    cpu::{Core, Register},
    pipeline::{Stage, WritebackStage},
};

use super::{
    decoder::C1_Funct3, opcodes::CompressedOpcode, ImmediateDecoder, Instruction,
    InstructionExcecutor, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CItype {
    pub opcode: CompressedOpcode,
    pub rd: Register,
    pub imm: u16,
    pub funct3: u8,
}

impl ImmediateDecoder<u16, u16> for CItype {
    fn decode_immediate(i: u16) -> u16 {
        let nzimm1612 = (i >> 2) & 31;
        let nzimm17 = (i >> 12) & 1;
        nzimm1612 | (nzimm17 << 5)
    }
}

impl Instruction<CItype> {
    #[allow(non_snake_case)]
    fn C_LUI(citype: CItype) -> Instruction<CItype> {
        Instruction {
            mnemonic: "C.LUI",
            args: Some(citype),
            funct: |core, args| {
                Stage::WRITEBACK(Some(WritebackStage {
                    register: args.rd,
                    value: ((args.imm as u64) << (core.xlen as u64 - 20)),
                }))
            },
        }
    }

    #[allow(non_snake_case)]
    fn C_ADDI(citype: CItype) -> Instruction<CItype> {
        Instruction {
            mnemonic: "C.ADDI",
            args: Some(citype),
            funct: |core, args| {
                Stage::WRITEBACK(Some(WritebackStage {
                    register: args.rd,
                    value: core
                        .read_register(args.rd)
                        .wrapping_add((args.imm as u64).sign_extend(64 - 12)),
                }))
            },
        }
    }
}

impl InstructionSelector<CItype> for CItype {
    fn select(&self) -> Instruction<CItype> {
        match self.opcode {
            CompressedOpcode::C1 => match num::FromPrimitive::from_u8(self.funct3).unwrap() {
                C1_Funct3::C_LUI => Instruction::C_LUI(*self),
                C1_Funct3::C_ADDI => Instruction::C_ADDI(*self),
                _ => panic!(),
            },
            _ => panic!(),
        }
    }
}

impl InstructionExcecutor for Instruction<CItype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}
