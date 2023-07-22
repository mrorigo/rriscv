use std::fmt::Display;

use quark::Signs;

use crate::{
    cpu::{Core, Register},
    pipeline::{Stage, WritebackStage},
};

use super::{
    decoder::{CSR_Funct3, OpImmFunct3},
    opcodes::MajorOpcode,
    Instruction, InstructionExcecutor, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Itype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub rs1: Register,
    pub funct3: u8,
    pub imm12: u16,
}

impl Instruction<Itype> {
    #[allow(non_snake_case)]
    fn CSRRS(itype: Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"CSRRS",
            args: Some(itype),
            funct: |core, args| {
                let csr_register = num::FromPrimitive::from_u16(args.imm12).unwrap();
                let csrv = core.read_csr(csr_register);
                if args.rs1 != 0 {
                    let rs1v = core.read_register(args.rs1);
                    core.write_csr(csr_register, csrv | rs1v);
                }
                Stage::WRITEBACK(Some(WritebackStage {
                    register: args.rd,
                    value: csrv,
                }))
            },
        }
    }

    #[allow(non_snake_case)]
    fn ADDI(itype: Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"ADDI",
            args: Some(itype),
            funct: |core, args| {
                Stage::WRITEBACK(Some(WritebackStage {
                    register: args.rd,
                    value: core
                        .read_register(args.rs1)
                        .wrapping_add((args.imm12 as u64).sign_extend(64 - 12)),
                }))
            },
        }
    }
}

impl Display for Instruction<Itype> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.args.is_some() {
            write!(f, "{}", self.mnemonic)
        } else {
            let args = self.args.unwrap();
            write!(
                f,
                "{} x{}, {}(x{})",
                self.mnemonic, args.rd, args.imm12, args.rs1
            )
        }
    }
}

impl InstructionSelector<Itype> for Itype {
    fn select(&self) -> Instruction<Itype> {
        match self.opcode {
            MajorOpcode::OP_IMM => match num::FromPrimitive::from_u8(self.funct3).unwrap() {
                OpImmFunct3::ADDI => Instruction::ADDI(*self),
                _ => panic!(),
            },
            MajorOpcode::SYSTEM => match num::FromPrimitive::from_u8(self.funct3).unwrap() {
                CSR_Funct3::CSRRS => Instruction::CSRRS(*self),
                _ => panic!(),
            },
            _ => panic!(),
        }
    }
}

impl InstructionExcecutor for Instruction<Itype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}
