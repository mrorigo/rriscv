use crate::{
    cpu::{Core, Register},
    pipeline::{Stage, WritebackStage},
};

use super::{
    decoder::RV32M_Funct3, opcodes::MajorOpcode, Instruction, InstructionExcecutor,
    InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Rtype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub rs1: Register,
    pub rs2: Register,
    pub funct3: u8,
    pub funct7: u8,
}

impl Instruction<Rtype> {
    #[allow(non_snake_case)]
    fn MUL(itype: Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"MUL",
            args: Some(itype),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1);
                let r2v = core.read_register(args.rs2);
                Stage::WRITEBACK(Some(WritebackStage {
                    register: args.rd,
                    value: core.bit_extend(r1v.wrapping_mul(r2v) as i64) as u64,
                }))
            },
        }
    }
}

impl InstructionSelector<Rtype> for Rtype {
    fn select(&self) -> Instruction<Rtype> {
        match self.opcode {
            MajorOpcode::OP => match self.funct7 {
                // RV32M
                1 => match num::FromPrimitive::from_u8(self.funct3).unwrap() {
                    RV32M_Funct3::MUL => Instruction::MUL(*self),
                    _ => panic!(),
                },
                _ => todo!("Support non-RV32M OP opcode"),
            },
            _ => panic!(),
        }
    }
}
impl InstructionExcecutor for Instruction<Rtype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}
