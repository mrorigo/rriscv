use std::fmt::Display;

use crate::{
    cpu::{Core, Register, Xlen},
    pipeline::{MemoryAccess, Stage},
};

use super::{
    functions::{Funct3, Funct5, Funct7, Op32_Funct3, Op_Funct3, RV32M_Funct3, RV64M_Funct3},
    opcodes::MajorOpcode,
    FormatDecoder, Instruction, InstructionFormatType, InstructionSelector, UncompressedFormatType,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Rtype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub rs1: Register,
    pub rs2: Register,
    pub funct3: Funct3,
    pub funct5: Funct5,
    pub funct7: Funct7,
}

impl InstructionFormatType for Rtype {}
impl UncompressedFormatType for Rtype {}

impl FormatDecoder<Rtype> for Rtype {
    fn decode(word: u32) -> Rtype {
        Rtype {
            opcode: num::FromPrimitive::from_u8((word & 0x7f) as u8).unwrap(),
            rd: ((word >> 7) & 31) as Register,
            rs1: ((word >> 15) & 31) as Register,
            rs2: ((word >> 20) & 31) as Register,
            funct3: num::FromPrimitive::from_u8(((word >> 12) & 7) as u8).unwrap(),
            funct7: num::FromPrimitive::from_u8(((word >> 25) & 0x7f) as u8)
                .unwrap_or(Funct7::B0000000),
            funct5: num::FromPrimitive::from_u8(((word >> 27) & 0x1f) as u8).unwrap(), // @FIXME: Default?
        }
    }
}

impl Display for Instruction<Rtype> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.args.is_some() {
            write!(f, "{}", self.mnemonic)
        } else {
            let args = self.args.unwrap();
            write!(
                f,
                "{} x{},x{},x{}",
                self.mnemonic, args.rd, args.rs1, args.rs2
            )
        }
    }
}

#[allow(non_snake_case)]
impl Instruction<Rtype> {
    pub fn ADD(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"ADD",
            args: Some(*args),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1) as i64;
                let r2v = core.read_register(args.rs2) as i64;
                let value = core.bit_extend(r1v.wrapping_add(r2v) as i64) as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn AND(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"AND",
            args: Some(*args),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1);
                let r2v = core.read_register(args.rs2);
                let value = r1v & r2v;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn SRL(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"SRL",
            args: Some(*args),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1);
                let r2v = core.read_register(args.rs2);
                let value = r1v.wrapping_shr(r2v as u32) as i64 as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn OR(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"OR",
            args: Some(*args),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1);
                let r2v = core.read_register(args.rs2);
                let value = core.bit_extend((r1v | r2v) as i64) as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn XOR(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"XOR",
            args: Some(*args),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1);
                let r2v = core.read_register(args.rs2);
                let value = core.bit_extend((r1v ^ r2v) as i64) as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn REMUW(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"REMUW",
            args: Some(*args),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1);
                let r2v = core.read_register(args.rs2);
                let dividend = r1v as u32;
                let divisor = r2v as u32;
                let value = match divisor {
                    0 => dividend as i32 as i64,
                    _ => dividend.wrapping_rem(divisor) as i32 as i64,
                };

                Stage::writeback(args.rd, value as u64)
            },
        }
    }

    pub fn DIVUW(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"DIVUW",
            args: Some(*args),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1);
                let r2v = core.read_register(args.rs2);
                let dividend = r1v as u32;
                let divisor = r2v as u32;
                let value = match divisor {
                    0 => -1 as i64,
                    _ => dividend.wrapping_div(divisor) as i32 as i64,
                };
                //println!("DIVUW: r1v: {:#x?} r2v: {:#x?} dividend: {:#x?} divisor: {:#x?}  result: {:#x?}", r1v, r2v, dividend, divisor, value);

                Stage::writeback(args.rd, value as u64)
            },
        }
    }

    pub fn MULW(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"MULW",
            args: Some(*args),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1) as i32;
                let r2v = core.read_register(args.rs2) as i32;
                let value = core.bit_extend(r1v.wrapping_mul(r2v) as i64) as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn MUL(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"MUL",
            args: Some(*args),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1);
                let r2v = core.read_register(args.rs2);
                let value = core.bit_extend(r1v.wrapping_mul(r2v) as i64) as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn MULH(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"MULH",
            args: Some(*args),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1) as i64;
                let r2v = core.read_register(args.rs2) as i64;
                let value = match core.xlen {
                    Xlen::Bits32 => core.bit_extend((r1v as i64 * r2v as i64) >> 32) as u64,
                    Xlen::Bits64 => ((r1v as i128) * (r2v as i128) >> 64) as u64,
                };
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn SUB(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"SUB",
            args: Some(*args),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1) as i64;
                let r2v = core.read_register(args.rs2) as i64;
                let value = core.bit_extend((r1v.wrapping_sub(r2v)) as i64) as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn SUBW(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"SUBW",
            args: Some(*args),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1) as i32;
                let r2v = core.read_register(args.rs2) as i32;
                let value = core.bit_extend((r1v.wrapping_sub(r2v)) as i64) as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn ADDW(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"ADDW",
            args: Some(*args),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1) as i32;
                let r2v = core.read_register(args.rs2) as i32;
                let value = core.bit_extend((r1v.wrapping_add(r2v)) as i64) as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn SLLW(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"SLLW",
            args: Some(*args),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1) as u32;
                let r2v = core.read_register(args.rs2) as u32;
                let value = r1v.wrapping_shl(r2v) as u32 as i32 as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn SRLW(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"SRLW",
            args: Some(*args),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1) as u32;
                let r2v = core.read_register(args.rs2) as u32;
                let value = r1v.wrapping_shr(r2v) as u32 as i32 as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn SLT(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"SLT",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as i32;
                let rs2v = core.read_register(args.rs2) as i32;
                let value = match rs1v < rs2v {
                    true => 1,
                    false => 0,
                };
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn SLTU(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"SLTU",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as u32;
                let rs2v = core.read_register(args.rs2) as u32;
                let value = match rs1v < rs2v {
                    true => 1,
                    false => 0,
                };
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn SLL(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"SLL",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as u64;
                let rs2v = core.read_register(args.rs2) as u32;
                let value = core.bit_extend(rs1v.wrapping_shl(rs2v) as i64) as u64;
                match args.rd {
                    0 => Stage::WRITEBACK(None),
                    _ => Stage::writeback(args.rd, value),
                }
            },
        }
    }

    pub fn SRA(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"SRA",
            args: Some(*args),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1) as i64;
                let r2v = core.read_register(args.rs2) as u32;
                let value = core.bit_extend((r1v.wrapping_shr(r2v & 0x1f)) as i64) as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }
}

#[allow(non_snake_case)]
impl Instruction<Rtype> {
    pub fn AMOSWAP_W(args: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: "AMOSWAP.W",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1);
                let rs2v = core.read_register(args.rs2);
                Stage::MEMORY(MemoryAccess::AMOSWAP_W(rs1v, rs2v, args.rd))
            },
        }
    }
}

impl InstructionSelector<Rtype> for Rtype {
    fn select(&self, _xlen: Xlen) -> Instruction<Rtype> {
        match self.opcode {
            MajorOpcode::AMO => match self.funct5 {
                Funct5::AMOSWAP_W => Instruction::AMOSWAP_W(self),
                _ => panic!(),
            },
            MajorOpcode::OP_32 => match self.funct7 {
                Funct7::B0000001 => match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
                    RV64M_Funct3::REMUW => Instruction::REMUW(self),
                    RV64M_Funct3::DIVUW => Instruction::DIVUW(self),
                    RV64M_Funct3::MULW => Instruction::MULW(self),
                    _ => panic!(),
                },
                _ => match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
                    Op32_Funct3::SRLW => Instruction::SRLW(self),
                    Op32_Funct3::SLLW => Instruction::SLLW(self),
                    Op32_Funct3::ADDW_SUBW => match self.funct7 {
                        Funct7::B0000000 => Instruction::ADDW(self),
                        Funct7::B0100000 => Instruction::SUBW(self),
                        _ => panic!(),
                    },
                },
            },
            MajorOpcode::OP => match self.funct7 {
                // RV32M
                Funct7::B0000001 => match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
                    RV32M_Funct3::MUL => Instruction::MUL(self),
                    RV32M_Funct3::MULH => Instruction::MULH(self),
                    _ => panic!(),
                },
                Funct7::B0100000 => match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
                    Op_Funct3::ADD_SUB => Instruction::SUB(self),
                    Op_Funct3::SRL_SRA => Instruction::SRA(self), //todo!("SRA, since Funct7!=0"),
                    _ => todo!("keine anung"),
                },
                Funct7::B0000000 => match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
                    Op_Funct3::ADD_SUB => Instruction::ADD(self),
                    Op_Funct3::SRL_SRA => Instruction::SRL(self),
                    Op_Funct3::AND => Instruction::AND(self),
                    Op_Funct3::OR => Instruction::OR(self),
                    Op_Funct3::XOR => Instruction::XOR(self),
                    Op_Funct3::SLTU => Instruction::SLTU(self),
                    Op_Funct3::SLL => Instruction::SLL(self),
                    Op_Funct3::SLT => Instruction::SLT(self),
                },
                _ => todo!("R-type Funct7 not supported"),
            },
            _ => panic!(),
        }
    }
}
