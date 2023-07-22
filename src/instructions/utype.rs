use crate::{
    cpu::{Core, Register},
    pipeline::{Stage, WritebackStage},
};

use super::{opcodes::MajorOpcode, Instruction, InstructionExcecutor, InstructionSelector};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Utype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub imm20: u32,
}

impl Instruction<Utype> {
    #[allow(non_snake_case)]
    fn AUIPC(utype: Utype) -> Instruction<Utype> {
        Instruction {
            mnemonic: "AUIPC",
            args: Some(utype),
            funct: |core, args| {
                const M: u32 = 1 << (20 - 1);
                let se_imm20 = (args.imm20 ^ M) - M;
                Stage::WRITEBACK(Some(WritebackStage {
                    register: args.rd,
                    value: (se_imm20 << 12) as u64 + core.prev_pc,
                }))
            },
        }
    }

    #[allow(non_snake_case)]
    fn LUI(utype: Utype) -> Instruction<Utype> {
        Instruction {
            mnemonic: "LUI",
            args: Some(utype),
            funct: |core, args| {
                Stage::WRITEBACK(Some(WritebackStage {
                    register: args.rd,
                    value: ((args.imm20 as u64) << (core.xlen as u64 - 20)),
                }))
            },
        }
    }
}

impl InstructionExcecutor for Instruction<Utype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}

impl InstructionSelector<Utype> for Utype {
    fn select(&self) -> Instruction<Utype> {
        match self.opcode {
            MajorOpcode::AUIPC => Instruction::AUIPC(*self),
            MajorOpcode::LUI => Instruction::LUI(*self),
            _ => panic!(),
        }
    }
}
