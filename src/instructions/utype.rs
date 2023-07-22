use crate::{
    cpu::{Core, Register, Xlen},
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
    pub fn AUIPC(utype: Utype) -> Instruction<Utype> {
        Instruction {
            mnemonic: "AUIPC",
            args: Some(utype),
            funct: |core, args| {
                const M: u32 = 1 << (20 - 1);
                let se_imm20 = (args.imm20 ^ M) - M;
                let value = ((se_imm20 << 12) as u64).wrapping_add(core.prev_pc);
                debug_trace!(println!(
                    "AUIPC x{}, {:#x?}\t; x{}={:#x?}",
                    args.rd, args.imm20, args.rd, value
                ));
                Stage::WRITEBACK(Some(WritebackStage {
                    register: args.rd,
                    value,
                }))
            },
        }
    }

    #[allow(non_snake_case)]
    pub fn LUI(utype: Utype) -> Instruction<Utype> {
        Instruction {
            mnemonic: "LUI",
            args: Some(utype),
            funct: |core, args| {
                Stage::WRITEBACK(Some(WritebackStage {
                    register: args.rd,
                    value: (args.imm20 as u64) << 12,
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
    fn select(&self, _xlen: Xlen) -> Instruction<Utype> {
        match self.opcode {
            MajorOpcode::AUIPC => Instruction::AUIPC(*self),
            MajorOpcode::LUI => Instruction::LUI(*self),
            _ => panic!(),
        }
    }
}
