use crate::{
    cpu::{Core, Register, Xlen},
    pipeline::{Stage, WritebackStage},
};

use super::{
    opcodes::MajorOpcode, FormatDecoder, ImmediateDecoder, Instruction, InstructionExcecutor,
    InstructionFormatType, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Jtype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub imm20: u32,
}

impl InstructionFormatType for Jtype {}

impl FormatDecoder<Jtype> for Jtype {
    fn decode(word: u32) -> Jtype {
        Jtype {
            opcode: num::FromPrimitive::from_u8((word & 0x7f) as u8).unwrap(),
            rd: ((word >> 7) & 31) as Register,
            imm20: Jtype::decode_immediate(word),
        }
    }
}

#[allow(non_snake_case)]
impl Instruction<Jtype> {
    pub fn JAL(args: &Jtype) -> Instruction<Jtype> {
        Instruction {
            mnemonic: "JAL",
            args: Some(*args),
            funct: |core, args| {
                const M: u32 = 1 << (20 - 1);
                let se_imm20 = ((args.imm20 << 1) ^ M) - M;
                let target = core.prev_pc + se_imm20 as u64;
                //println!("JAL: core.pc: {:#x?}  se_imm20: {:#x?}", core.pc, se_imm20);
                core.set_pc(target);

                debug_trace!(println!("JAL {:#x?}", target));

                Stage::WRITEBACK(Some(WritebackStage {
                    register: args.rd,
                    value: core.pc,
                }))
            },
        }
    }
}

impl ImmediateDecoder<u32, u32> for Jtype {
    fn decode_immediate(i: u32) -> u32 {
        let imm20 = ((i >> 31) & 0b1) as u32;
        let imm101 = ((i >> 21) & 0b1111111111) as u32;
        let imm11 = ((i >> 20) & 0b1) as u32;
        let imm1912 = ((i >> 12) & 0b11111111) as u32;

        let imm = (imm20 << 20) | (imm101 << 1) | (imm11 << 11) | (imm1912 << 12);
        ((imm) << 11) >> 12
    }
}

impl InstructionSelector<Jtype> for Jtype {
    fn select(&self, xlen: Xlen) -> Instruction<Jtype> {
        match self.opcode {
            MajorOpcode::JAL => Instruction::JAL(self),
            _ => panic!(),
        }
    }
}

impl InstructionExcecutor for Instruction<Jtype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}
