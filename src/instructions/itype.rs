use std::fmt::Display;

use quark::Signs;

use crate::{
    cpu::{Core, Register, Xlen},
    pipeline::{Stage, WritebackStage},
};

use super::{
    functions::{CSR_Funct3, OpImmFunct3},
    opcodes::MajorOpcode,
    FormatDecoder, Instruction, InstructionExcecutor, InstructionFormatType, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Itype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub rs1: Register,
    pub funct3: u8,
    pub imm12: u16,
}

impl InstructionFormatType for Itype {}

impl FormatDecoder<Itype> for Itype {
    fn decode(word: u32) -> Itype {
        Itype {
            opcode: num::FromPrimitive::from_u8((word & 0x7f) as u8).unwrap(),
            rd: ((word >> 7) & 31) as Register,
            rs1: ((word >> 15) & 31) as Register,
            imm12: (word >> 20) as u16,
            funct3: ((word >> 12) & 7) as u8,
        }
    }
}

#[allow(non_snake_case)]
impl Instruction<Itype> {
    pub fn CSRRW(itype: Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"CSRRW",
            args: Some(itype),
            funct: |core, args| {
                let csr_register = num::FromPrimitive::from_u16(args.imm12).unwrap();
                // "If rd=x0, then the instruction shall not read the CSR"
                let csrv = if args.rd != 0 {
                    Some(core.read_csr(csr_register))
                } else {
                    None
                };

                let rs1v = core.read_register(args.rs1);
                core.write_csr(csr_register, rs1v);

                if csrv.is_none() {
                    Stage::WRITEBACK(None)
                } else {
                    Stage::WRITEBACK(Some(WritebackStage {
                        register: args.rd,
                        value: csrv.unwrap(),
                    }))
                }
            },
        }
    }

    pub fn CSRRS(itype: Itype) -> Instruction<Itype> {
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

    pub fn ADDI(itype: Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"ADDI",
            args: Some(itype),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as i64;
                let seimm = (args.imm12 as u64).sign_extend(64 - 12);
                let value = core.bit_extend(rs1v.wrapping_add(seimm as i64)) as u64;
                // debug_trace!(println!(
                //     "ADDI x{}, x{}, {}  ; x{} = {:#x?}",
                //     args.rd, args.rs1, seimm as i64, args.rd, value
                // ));
                Stage::WRITEBACK(Some(WritebackStage {
                    register: args.rd,
                    value,
                }))
            },
        }
    }

    pub fn ORI(itype: Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"ORI",
            args: Some(itype),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as i64;
                let seimm = (args.imm12 as u64).sign_extend(64 - 12);
                let value = core.bit_extend(rs1v | seimm as i64) as u64;
                debug_trace!(println!(
                    "ORI x{}, x{}, {:#x?}  ; x{} = {:#x?}",
                    args.rd, args.rs1, seimm as u64, args.rd, value
                ));
                Stage::WRITEBACK(Some(WritebackStage {
                    register: args.rd,
                    value,
                }))
            },
        }
    }

    pub fn JALR(itype: Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"JALR",
            args: Some(itype),
            funct: |core, args| {
                let value = core.pc;

                let se_imm12 = (args.imm12 as u64).sign_extend(64 - 12) as i64;

                let rs1v = core.read_register(args.rs1);
                let target = (rs1v as i64 + se_imm12) as u64;
                core.set_pc(target);

                if args.rd != 0 {
                    Stage::WRITEBACK(Some(WritebackStage {
                        register: args.rd,
                        value,
                    }))
                } else {
                    Stage::WRITEBACK(None)
                }
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
                "{} x{},x{},{}",
                self.mnemonic, args.rd, args.rs1, args.imm12,
            )
        }
    }
}

impl InstructionSelector<Itype> for Itype {
    fn select(&self, _xlen: Xlen) -> Instruction<Itype> {
        match self.opcode {
            MajorOpcode::OP_IMM => match num::FromPrimitive::from_u8(self.funct3).unwrap() {
                OpImmFunct3::ADDI => Instruction::ADDI(*self),
                OpImmFunct3::ORI => Instruction::ORI(*self),
                _ => panic!(),
            },
            MajorOpcode::SYSTEM => match num::FromPrimitive::from_u8(self.funct3).unwrap() {
                CSR_Funct3::CSRRS => Instruction::CSRRS(*self),
                CSR_Funct3::CSRRW => Instruction::CSRRW(*self),
                _ => panic!(),
            },
            MajorOpcode::JALR => Instruction::JALR(*self),
            _ => panic!(),
        }
    }
}

impl InstructionExcecutor for Instruction<Itype> {
    fn run(&self, core: &mut Core) -> Stage {
        debug_trace!(println!("{}", self.to_string()));
        (self.funct)(core, &self.args.unwrap())
    }
}
