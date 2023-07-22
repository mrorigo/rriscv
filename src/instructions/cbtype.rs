use std::fmt::Display;

use quark::Signs;

use crate::{
    cpu::{Core, Register, Xlen},
    pipeline::Stage,
};

use super::{
    functions::{C1_Funct3, Funct3},
    opcodes::CompressedOpcode,
    CompressedFormatDecoder, CompressedFormatType, ImmediateDecoder, Instruction,
    InstructionExcecutor, InstructionFormatType, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CBtype {
    pub opcode: CompressedOpcode,
    pub rs1: Register,
    pub offset: u16,
    pub funct3: Funct3,
    pub funct2: u8,
    pub imm6: u8,
}

impl InstructionFormatType for CBtype {}
impl CompressedFormatType for CBtype {}

impl CompressedFormatDecoder<CBtype> for CBtype {
    fn decode(word: u16) -> CBtype {
        CBtype {
            opcode: num::FromPrimitive::from_u8((word & 3) as u8).unwrap(),
            rs1: 8 + ((word >> 7) & 3) as u8,
            offset: CBtype::decode_immediate(word as u16),
            imm6: (((word >> 2) & 0b1111) | ((word >> 12) & 1) << 5) as u8,
            funct2: ((word >> 10) & 0b11) as u8,
            funct3: num::FromPrimitive::from_u8(((word >> 13) & 0x7) as u8).unwrap(),
        }
    }
}

// CB format
// +-------+-------------+------+-----------------+-------+
// 15    13|12         10|9    7|6               2|1     0|
// +-------+-------------+------+-----------------+-------+
// |funct3 |    imm      | rs1' |      imm        |  op   |
// +-------+-------------+------+-----------------+-------+
// |   3   |     3       |  3   |        5        |   2   |
// |C.BEQZ | offs[8|4:3] | src  | offs[7:6|2:1|5] |   C1  |
// |C.BNEZ | offs[8|4:3] | src  | offs[7:6|2:1|5] |   C1  |
// +-------+-------------+------+-----------------+-------+

impl ImmediateDecoder<u16, u16> for CBtype {
    fn decode_immediate(halfword: u16) -> u16 {
        (match halfword & 0x1000 {
            0x1000 => 0xfe00,
            _ => 0,
        }) | ((halfword >> 4) & 0x100)
            | ((halfword >> 7) & 0x18)
            | ((halfword << 1) & 0xc0)
            | ((halfword >> 2) & 0x6)
            | ((halfword << 3) & 0x20)
    }
}

impl Display for Instruction<CBtype> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.args.is_some() {
            write!(f, "{}", self.mnemonic)
        } else {
            let args = self.args.unwrap();
            write!(f, "{} x{},{:#x?}", self.mnemonic, args.rs1, args.offset)
        }
    }
}

#[allow(non_snake_case)]
impl Instruction<CBtype> {
    /// C.BEQZ takes the branch if the value in register rs1 is zero.
    pub fn C_BEQZ(args: &CBtype) -> Instruction<CBtype> {
        Instruction {
            args: Some(*args),
            mnemonic: "C.BEQZ",
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1);
                if rs1v == 0 {
                    let target =
                        ((args.offset as i64).sign_extend(64 - 9) + core.prev_pc as i64) as u64;
                    core.set_pc(target)
                }
                Stage::WRITEBACK(None)
            },
        }
    }
    /// C.BNEZ takes the branch if the value in register rs1 is NOT zero.
    pub fn C_BNEZ(args: &CBtype) -> Instruction<CBtype> {
        Instruction {
            args: Some(*args),
            mnemonic: "C.BNEZ",
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1);
                if rs1v != 0 {
                    let target =
                        ((args.offset as i64).sign_extend(64 - 9) + core.prev_pc as i64) as u64;
                    core.set_pc(target)
                }
                Stage::WRITEBACK(None)
            },
        }
    }

    pub fn C_ANDI(args: &CBtype) -> Instruction<CBtype> {
        Instruction {
            args: Some(*args),
            mnemonic: "C.ANDI",
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1);
                let mask = (args.imm6 as i64).sign_extend(64 - 6);
                let value = rs1v & mask as u64;

                Stage::writeback(args.rs1, value)
            },
        }
    }

    pub fn C_SRLI(args: &CBtype) -> Instruction<CBtype> {
        Instruction {
            mnemonic: &"C.SRLI",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1);
                // the shift amount is encoded in the lower 6 bits of the I-immediate field for RV64I.
                let mask = match core.xlen {
                    Xlen::Bits32 => 0x1f,
                    Xlen::Bits64 => 0x3f,
                };

                let shamt = (args.imm6) & mask;
                let value = (rs1v as u64) >> shamt;
                Stage::writeback(args.rs1, value)
            },
        }
    }
    pub fn C_SRAI(args: &CBtype) -> Instruction<CBtype> {
        Instruction {
            mnemonic: &"C.SRAI",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1);
                // the shift amount is encoded in the lower 6 bits of the I-immediate field for RV64I.
                let mask = match core.xlen {
                    Xlen::Bits32 => 0x1f,
                    Xlen::Bits64 => 0x3f,
                };
                let shamt = (args.imm6) & mask;
                println!("C.SRAI shamt: {:?}", shamt);
                let value = ((rs1v as i64).wrapping_shr(shamt as u32)) as u64;
                Stage::writeback(args.rs1, value)
            },
        }
    }
}

impl InstructionSelector<CBtype> for CBtype {
    fn select(&self, _xlen: crate::cpu::Xlen) -> Instruction<CBtype> {
        match self.opcode {
            CompressedOpcode::C1 => match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
                C1_Funct3::C_BEQZ => Instruction::C_BEQZ(self),
                C1_Funct3::C_BNEZ => Instruction::C_BNEZ(self),
                C1_Funct3::C_ANDI => match self.funct2 {
                    0b00 => Instruction::C_SRLI(self),
                    0b01 => Instruction::C_SRAI(self),
                    0b10 => Instruction::C_ANDI(self),
                    _ => panic!(),
                },
                _ => todo!(),
            },
            _ => panic!(),
        }
    }
}

impl InstructionExcecutor<CBtype> for Instruction<CBtype> {
    fn run(&self, core: &mut Core) -> Stage {
        instruction_trace!(println!("{}", self.to_string()));
        (self.funct)(core, &self.args.unwrap())
    }
}
