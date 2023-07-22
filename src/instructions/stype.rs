use crate::{
    cpu::{Core, Register, Xlen},
    pipeline::{MemoryAccess, Stage},
};

use super::{
    functions::Store_Funct3, opcodes::MajorOpcode, FormatDecoder, ImmediateDecoder, Instruction,
    InstructionExcecutor, InstructionFormatType, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Stype {
    pub opcode: MajorOpcode,
    pub rs1: Register,
    pub rs2: Register,
    pub imm12: u16,
    pub funct3: u8,
}

impl InstructionFormatType for Stype {}

impl FormatDecoder<Stype> for Stype {
    fn decode(word: u32) -> Stype {
        Stype {
            opcode: num::FromPrimitive::from_u8((word & 0x7f) as u8).unwrap(),
            rs1: ((word >> 15) & 31) as Register,
            rs2: ((word >> 20) & 31) as Register,
            imm12: Stype::decode_immediate(word),
            funct3: ((word >> 12) & 7) as u8,
        }
    }
}

impl ImmediateDecoder<u32, u16> for Stype {
    fn decode_immediate(i: u32) -> u16 {
        let imm12 = (((i >> 7) & 0b11111) | ((i >> 20) & 0xffffe0)) as u16;
        let imm5 = ((i >> 7) & 31) as u16;
        imm12 | imm5 as u16
    }
}
#[allow(non_snake_case)]

impl Instruction<Stype> {
    pub fn SD(args: &Stype) -> Instruction<Stype> {
        Instruction {
            mnemonic: "SD",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1);
                let rs2v = core.read_register(args.rs2);

                // The effective byte address is obtained by adding register rs1 to the sign-extended 12-bit offset
                let addr = rs1v + (args.imm12 as u64);

                Stage::MEMORY(MemoryAccess::WRITE64(addr, rs2v))
            },
        }
    }
}

impl InstructionSelector<Stype> for Stype {
    fn select(&self, _xlen: Xlen) -> Instruction<Stype> {
        match self.opcode {
            MajorOpcode::STORE => match num::FromPrimitive::from_u8(self.funct3).unwrap() {
                Store_Funct3::SD => Instruction::SD(self),
                Store_Funct3::SB => todo!(),
                Store_Funct3::SH => todo!(),
                Store_Funct3::SW => todo!(),
            },
            _ => panic!(),
        }
    }
}

impl InstructionExcecutor for Instruction<Stype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}
