use std::fmt::Display;

use elfloader::VAddr;

use crate::{
    cpu::{Core, Register, RegisterValue, Xlen},
    pipeline::{MemoryAccess, Stage},
};

use super::{
    functions::{C0_Funct3, Funct3},
    opcodes::CompressedOpcode,
    CompressedFormatDecoder, CompressedFormatType, ImmediateDecoder, Instruction,
    InstructionExcecutor, InstructionFormatType, InstructionSelector,
};

/// This instruction format is shared between C0 and C1 ops, hence it
/// has both funct, funct3, and funct6 decoded.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CStype {
    pub opcode: CompressedOpcode,
    pub rs1_rd: Register,
    pub rs2: Register,
    pub offset: u8,
    pub shamt: u8,
    pub funct2: u8,
    pub funct6: u8,
    pub funct3: Funct3,
}

impl InstructionFormatType for CStype {}
impl CompressedFormatType for CStype {}

impl ImmediateDecoder<u16, u8> for CStype {
    fn decode_immediate(i: u16) -> u8 {
        (((i >> 7) & 0x38) | ((i << 1) & 0x40) | ((i >> 4) & 0x4)) as u8

        //        (((i >> 7) & 0x38) | ((i << 1) & 0xc0)) as u8
    }
}

impl CompressedFormatDecoder<CStype> for CStype {
    fn decode(word: u16) -> CStype {
        CStype {
            opcode: num::FromPrimitive::from_u8((word & 3) as u8).unwrap(),
            rs1_rd: ((word >> 7) & 7) as u8 + 8,
            rs2: ((word >> 2) & 7) as u8 + 8,
            shamt: (((word >> 7) & 0b100000) | ((word >> 2) & 0x1f)) as u8,
            offset: CStype::decode_immediate(word),
            funct2: (word >> 5) as u8 & 0x3,
            funct6: (word >> 10) as u8 & 0b111111,
            funct3: num::FromPrimitive::from_u8(((word >> 13) & 0x7) as u8).unwrap(),
        }
    }
}

impl Display for Instruction<CStype> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.args.is_some() {
            write!(f, "{}", self.mnemonic)
        } else {
            let args = self.args.unwrap();
            write!(f, "{} x{},x{}", self.mnemonic, args.rs1_rd, args.rs2)
        }
    }
}

macro_rules! cs_operation {
    ($args:expr) => {
        |core, args| {
            let rs1v = core.read_register(args.rs1_rd);
            let rs2v = core.read_register(args.rs2);
            let value = $args(core, rs1v, rs2v);
            Stage::writeback(args.rs1_rd, value)
        }
    };
}

macro_rules! instruction {
    ($type:tt, $name:tt, $mnemonic:expr, $op:expr) => {
        pub fn $name(args: &$type) -> Instruction<$type> {
            Instruction {
                args: Some(*args),
                mnemonic: $mnemonic,
                funct: $op,
            }
        }
    };
}

macro_rules! cs_instruction {
    ($name:tt, $mnemonic:expr, $op:expr) => {
        instruction!(CStype, $name, $mnemonic, $op);
    };
}

#[allow(non_snake_case)]
impl Instruction<CStype> {
    cs_instruction!(
        C_AND,
        "C.AND",
        cs_operation!(|core, rs1v, rs2v| rs1v & rs2v)
    );

    cs_instruction!(C_OR, "C.OR", cs_operation!(|core, rs1v, rs2v| rs1v | rs2v));

    cs_instruction!(
        C_XOR,
        "C.XOR",
        cs_operation!(|core, rs1v, rs2v| rs1v ^ rs2v)
    );

    cs_instruction!(
        C_SUBW,
        "C.SUBW",
        cs_operation!(
            |core: &mut Core, rs1v: RegisterValue, rs2v: RegisterValue| core
                .bit_extend(((rs1v as i32).wrapping_sub(rs2v as i32)) as i64)
                as u64
        )
    );

    cs_instruction!(
        C_ADDW,
        "C.ADDW",
        cs_operation!(
            |core: &mut Core, rs1v: RegisterValue, rs2v: RegisterValue| core
                .bit_extend(((rs1v as i32).wrapping_add(rs2v as i32)) as i64)
                as u64
        )
    );

    cs_instruction!(
        C_SUB,
        "C.SUB",
        cs_operation!(
            |core: &mut Core, rs1v: RegisterValue, rs2v: RegisterValue| core
                .bit_extend((rs1v.wrapping_sub(rs2v)) as i64)
                as u64
        )
    );

    cs_instruction!(C_SD, "C.SD", |core, args| {
        let rs1v = core.read_register(args.rs1_rd);
        let rs2v = core.read_register(args.rs2);
        let addr = rs1v + args.offset as VAddr;
        Stage::MEMORY(MemoryAccess::WRITE64(addr, rs2v))
    });

    cs_instruction!(C_SW, "C.SW", |core, args| {
        let rs1v = core.read_register(args.rs1_rd);
        let rs2v = core.read_register(args.rs2);
        let addr = (rs1v + args.offset as u64) as VAddr;
        Stage::MEMORY(MemoryAccess::WRITE32(addr, rs2v as u32))
    });
}

impl InstructionSelector<CStype> for CStype {
    fn select(&self, _xlen: Xlen) -> Instruction<CStype> {
        match self.opcode {
            CompressedOpcode::C0 => match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
                C0_Funct3::C_SD => Instruction::C_SD(self),
                C0_Funct3::C_SW => Instruction::C_SW(self),
                _ => panic!(),
            },
            CompressedOpcode::C1 => match self.funct6 {
                0b100011 => match self.funct2 {
                    0b00 => Instruction::C_SUB(self),
                    0b01 => Instruction::C_XOR(self),
                    0b10 => Instruction::C_OR(self),
                    0b11 => Instruction::C_AND(self),
                    _ => panic!(),
                },
                0b100111 => match self.funct2 {
                    0b00 => Instruction::C_SUBW(self),
                    0b01 => Instruction::C_ADDW(self),
                    _ => panic!("reserved instruction?"),
                },
                _ => panic!("{:#x?}", self.funct6),
            },
            _ => panic!(),
        }
    }
}

impl InstructionExcecutor<CStype> for Instruction<CStype> {
    fn run(&self, core: &mut Core) -> Stage {
        instruction_trace!(println!("{}", self.to_string()));
        (self.funct)(core, &self.args.unwrap())
    }
}
