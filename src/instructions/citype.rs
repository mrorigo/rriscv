use std::fmt::Display;

use quark::Signs;

use crate::{
    cpu::{Register, Xlen},
    pipeline::Stage,
};

use super::{
    functions::{C1_Funct3, C2_Funct3, Funct3},
    opcodes::CompressedOpcode,
    CompressedFormatDecoder, CompressedFormatType, ImmediateDecoder, Instruction,
    InstructionFormatType, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CItype {
    pub opcode: CompressedOpcode,
    pub rs1_rd: Register,
    pub imm: u16,
    pub funct3: Funct3,
    pub word: u16, // @FIXME: We need the original word here for ADDI16SP, which has a different immediate format
}

impl InstructionFormatType for CItype {}
impl CompressedFormatType for CItype {}

impl CompressedFormatDecoder<CItype> for CItype {
    fn decode(word: u16) -> CItype {
        CItype {
            opcode: num::FromPrimitive::from_u8((word & 3) as u8).unwrap(),
            rs1_rd: ((word >> 7) & 31) as u8,
            imm: CItype::decode_immediate(word as u16),
            funct3: num::FromPrimitive::from_u8(((word >> 13) & 0x7) as u8).unwrap(),
            word: word,
        }
    }
}

impl ImmediateDecoder<u16, u16> for CItype {
    // @FIXME: This only decodes the C.LUI-type immediate, there is also the C.ADDI16SP format,
    //         that we decode in the instruction itself for now.
    fn decode_immediate(i: u16) -> u16 {
        let nzimm1612 = (i >> 2) & 31;
        let nzimm17 = (i >> 12) & 1;
        nzimm1612 | (nzimm17 << 5)
    }
}

#[allow(non_snake_case)]
impl Instruction<CItype> {
    pub fn C_ADDI16SP(args: &CItype) -> Instruction<CItype> {
        Instruction {
            mnemonic: "C.ADDI16SP",
            args: Some(*args),
            funct: |core, args| {
                let imm = (match args.word & 0x1000 {
                    0x1000 => 0xfc00,
                    _ => 0,
                }) | ((args.word >> 3) & 0x200)
                    | ((args.word >> 2) & 0x10)
                    | ((args.word << 1) & 0x40)
                    | ((args.word << 4) & 0x180)
                    | ((args.word << 3) & 0x20);

                let sp = core.read_register(2) as i64;
                let value = sp.wrapping_add(imm as i16 as i32 as i64) as u64;
                Stage::writeback(args.rs1_rd, value)
            },
        }
    }

    pub fn C_LUI(args: &CItype) -> Instruction<CItype> {
        Instruction {
            mnemonic: "C.LUI",
            args: Some(*args),
            funct: |_core, args| {
                let value = (args.imm as u64).sign_extend(64 - 6) << 12;
                instruction_trace!(println!(
                    "C.LUI x{}, {:#x?} ; x{} = {:#x?}",
                    args.rs1_rd, args.imm, args.rs1_rd, value as i64
                ));
                Stage::writeback(args.rs1_rd, value)
            },
        }
    }

    pub fn C_ADDI(args: &CItype) -> Instruction<CItype> {
        Instruction {
            mnemonic: "C.ADDI",
            args: Some(*args),
            funct: |core, args| {
                let value = (core.read_register(args.rs1_rd) as i64)
                    .wrapping_add((args.imm as i64).sign_extend(64 - 6))
                    as u64;

                Stage::writeback(args.rs1_rd, value)
            },
        }
    }

    pub fn C_SLLI(args: &CItype) -> Instruction<CItype> {
        Instruction {
            mnemonic: &"C.SLLI",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1_rd);
                let mask = match core.xlen {
                    Xlen::Bits32 => 0x1f,
                    Xlen::Bits64 => 0x3f,
                    Xlen::Bits128 => 0x7f,
                };
                let shamt = (args.imm) & mask;
                let value = ((rs1v as i64) << shamt) as u64;
                Stage::writeback(args.rs1_rd, value)
            },
        }
    }

    pub fn C_LDSP(args: &CItype) -> Instruction<CItype> {
        Instruction {
            mnemonic: &"C.LDSP",
            args: Some(*args),
            funct: |core, args| {
                let ze_imm = args.imm as u64;
                let sp = core.read_register(2);
                let addr = sp + (ze_imm);
                // instruction_trace!(println!(
                //     "C.LDSP: sp={:#x?} ze_imm={:#x?} addr={:#x?}",
                //     sp, ze_imm, addr
                // ));
                Stage::MEMORY(crate::pipeline::MemoryAccess::READ64(
                    addr,
                    args.rs1_rd,
                    false,
                ))
            },
        }
    }

    pub fn C_LWSP(args: &CItype) -> Instruction<CItype> {
        Instruction {
            mnemonic: &"C.LWSP",
            args: Some(*args),
            funct: |core, args| {
                let ze_imm = args.imm as u64;
                let sp = core.read_register(2);
                let addr = sp + (ze_imm);
                instruction_trace!(println!(
                    "C.LWSP: sp={:#x?} ze_imm={:#x?} addr={:#x?}",
                    sp, ze_imm, addr
                ));
                Stage::MEMORY(crate::pipeline::MemoryAccess::READ32(
                    addr,
                    args.rs1_rd,
                    true,
                ))
            },
        }
    }

    pub fn C_LI(args: &CItype) -> Instruction<CItype> {
        Instruction {
            mnemonic: "C.LI",
            args: Some(*args),
            funct: |_core, args| {
                let value = (args.imm as u64).sign_extend(64 - 6);
                Stage::writeback(args.rs1_rd, value)
            },
        }
    }

    pub fn C_ADDIW(args: &CItype) -> Instruction<CItype> {
        Instruction {
            mnemonic: &"C.ADDIW",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1_rd) as i64;
                let seimm = (args.imm as u64).sign_extend(64 - 6);
                let value = core.bit_extend(rs1v.wrapping_add(seimm as i64)) as i32 as u64;
                instruction_trace!(println!(
                    "C.ADDIW x{}, {}  ; x{} = {:#x?}",
                    args.rs1_rd, seimm as i64, args.rs1_rd, value
                ));
                Stage::writeback(args.rs1_rd, value)
            },
        }
    }
}

impl Display for Instruction<CItype> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.args.is_some() {
            write!(f, "{}", self.mnemonic)
        } else {
            let args = self.args.unwrap();
            write!(f, "{} x{},{}", self.mnemonic, args.rs1_rd, args.imm,)
        }
    }
}

impl InstructionSelector<CItype> for CItype {
    fn select(&self, _xlen: Xlen) -> Instruction<CItype> {
        match self.opcode {
            CompressedOpcode::C1 => match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
                C1_Funct3::C_LUI => match self.rs1_rd != 0 && self.rs1_rd != 2 {
                    true => Instruction::C_LUI(self),
                    false => Instruction::C_ADDI16SP(self),
                },
                C1_Funct3::C_ADDI => Instruction::C_ADDI(self),
                C1_Funct3::C_LI => Instruction::C_LI(self),
                C1_Funct3::C_ADDIW => Instruction::C_ADDIW(self),
                _ => panic!(),
            },
            CompressedOpcode::C2 => match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
                C2_Funct3::C_SLLI => Instruction::C_SLLI(self),
                C2_Funct3::C_LDSP => Instruction::C_LDSP(self),
                C2_Funct3::C_LWSP => Instruction::C_LWSP(self),
                _ => panic!(),
            },
            _ => panic!("opcode {:#x?} unknown for CIType", self.opcode),
        }
    }
}
