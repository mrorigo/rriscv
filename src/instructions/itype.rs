use std::fmt::Display;

use quark::Signs;

use crate::{
    cpu::{CSRRegister, Core, Register, TrapCause, Xlen},
    pipeline::Stage,
};

use super::{
    functions::{
        CSR_Funct3, Funct3, Funct7, Load_Funct3, MiscMem_Funct3, OpImm32_Funct3, OpImm_Funct3,
    },
    opcodes::MajorOpcode,
    FormatDecoder, Instruction, InstructionExcecutor, InstructionFormatType, InstructionSelector,
    UncompressedFormatType,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Itype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub rs1: Register,
    pub funct3: Funct3,
    // Shifts by a constant are encoded as a specialization of the I-type format. (@see SRLI/SLLI/SRAI)
    pub imm12: u16,
    pub funct7: Funct7,
}

impl InstructionFormatType for Itype {}
impl UncompressedFormatType for Itype {}

impl FormatDecoder<Itype> for Itype {
    fn decode(word: u32) -> Itype {
        Itype {
            opcode: num::FromPrimitive::from_u8((word & 0x7f) as u8).unwrap(),
            rd: ((word >> 7) & 31) as Register,
            rs1: ((word >> 15) & 31) as Register,
            imm12: (word >> 20) as u16,
            funct3: num::FromPrimitive::from_u8(((word >> 12) & 7) as u8).unwrap(),
            funct7: num::FromPrimitive::from_u8((word >> 25) as u8).unwrap_or(Funct7::B0000000),
            //imm5: ((word >> 20) & 0x3f) as u8,
        }
    }
}

#[allow(non_snake_case)]
impl Instruction<Itype> {
    pub fn CSRRW(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"CSRRW",
            args: Some(*args),
            funct: |core, args| {
                let csr_register = num::FromPrimitive::from_u16(args.imm12).unwrap();
                // "If rd=x0, then the instruction shall not read the CSR"
                let value = if args.rd != 0 {
                    Some(core.read_csr(csr_register))
                } else {
                    None
                };

                let rs1v = core.read_register(args.rs1);
                core.write_csr(csr_register, rs1v);

                if value.is_none() {
                    Stage::WRITEBACK(None)
                } else {
                    Stage::writeback(args.rd, value.unwrap())
                }
            },
        }
    }

    pub fn CSRWI(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"CSRWI",
            args: Some(*args),
            funct: |core, args| {
                let csr_register = num::FromPrimitive::from_u16(args.imm12).unwrap();
                core.write_csr(csr_register, args.imm12 as u64);

                Stage::WRITEBACK(None)
            },
        }
    }

    pub fn CSRRS(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"CSRRS",
            args: Some(*args),
            funct: |core, args| {
                let csr_register = num::FromPrimitive::from_u16(args.imm12).unwrap();
                let value = core.read_csr(csr_register);
                if args.rs1 != 0 {
                    let rs1v = core.read_register(args.rs1);
                    core.write_csr(csr_register, value | rs1v);
                }
                match args.rd {
                    0 => Stage::WRITEBACK(None),
                    _ => Stage::writeback(args.rd, value),
                }
            },
        }
    }

    pub fn CSRRC(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"CSRRC",
            args: Some(*args),
            funct: |core, args| {
                let csr_register = num::FromPrimitive::from_u16(args.imm12).unwrap();
                let value = core.read_csr(csr_register);
                println!("CSRRC read value {:#x?}", value);
                if args.rs1 != 0 {
                    let rs1v = core.read_register(args.rs1);
                    core.write_csr(csr_register, value & !rs1v);
                }
                match args.rd {
                    0 => Stage::WRITEBACK(None),
                    _ => Stage::writeback(args.rd, value),
                }
            },
        }
    }
}

#[allow(non_snake_case)]
impl Instruction<Itype> {
    pub fn ADDI(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"ADDI",
            args: Some(*args),
            funct: |core, args| {
                if args.rs1 == 0 && args.rd == 0 {
                    // NOP
                    Stage::WRITEBACK(None)
                } else {
                    let rs1v = core.read_register(args.rs1) as i64;
                    let seimm = (args.imm12 as u64).sign_extend(64 - 12);
                    let value = core.bit_extend(rs1v.wrapping_add(seimm as i64)) as u64;
                    // instruction_trace!(println!(
                    //     "ADDI x{}, x{}, {}  ; x{} = {:#x?}",
                    //     args.rd, args.rs1, seimm as i64, args.rd, value
                    // ));
                    Stage::writeback(args.rd, value)
                }
            },
        }
    }

    pub fn ANDI(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"ANDI",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as u64;
                let seimm = (args.imm12 as u64).sign_extend(64 - 12);
                let value = core.bit_extend((rs1v & seimm) as i64) as u64;
                // instruction_trace!(println!(
                //     "ADDI x{}, x{}, {}  ; x{} = {:#x?}",
                //     args.rd, args.rs1, seimm as i64, args.rd, value
                // ));
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn ADDIW(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"ADDIW",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as i64;
                let seimm = (args.imm12 as u64).sign_extend(64 - 12);
                let value = core.bit_extend(rs1v.wrapping_add(seimm as i64)) as i32 as u64;
                // instruction_trace!(println!(
                //     "ADDIW x{}, x{}, {}  ; x{} = {:#x?} + {} => x{} = {:#x?}",
                //     args.rd, args.rs1, seimm as i64, args.rs1, rs1v, seimm as i64, args.rd, value
                // ));
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn ORI(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"ORI",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as i64;
                let seimm = (args.imm12 as u64).sign_extend(64 - 12);
                let value = core.bit_extend(rs1v | seimm as i64) as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn XORI(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"XORI",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as i64;
                let seimm = (args.imm12 as u64).sign_extend(64 - 12);
                let value = core.bit_extend(rs1v ^ seimm as i64) as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }
}

#[allow(non_snake_case)]
impl Instruction<Itype> {
    pub fn JALR(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"JALR",
            args: Some(*args),
            funct: |core, args| {
                let value = core.pc();
                let se_imm12 = (args.imm12 as u64).sign_extend(64 - 12) as i32;

                let rs1v = core.read_register(args.rs1);
                let target = (((rs1v as i32).wrapping_add(se_imm12)) as u32 & 0xffffffff) as u64;
                // instruction_trace!(println!(
                //     "JALR x{}={:#x?} target={:#x?}",
                //     args.rs1, rs1v, target
                // ));
                core.set_pc(target);

                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn SLLI(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"SLLI",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1);
                // the shift amount is encoded in the lower 6 bits of the I-immediate field for RV64I.
                let mask = match core.xlen {
                    Xlen::Bits32 => 0x1f,
                    Xlen::Bits64 => 0x3f,
                };
                let shamt = (args.imm12) & mask;
                let value = ((rs1v as i64) << shamt) as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn SLTI(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"SLTI",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as i64;
                let value = match rs1v < (args.imm12 as i64).sign_extend(64 - 12) {
                    true => 1,
                    false => 0,
                };
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn SLTIU(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"SLTIU",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as u64;
                let imm = (args.imm12 as i64).sign_extend(64 - 12) as u64;
                //println!("SLTIU imm: {:#x?} {:#x?}", args.imm12, imm);
                let value = match rs1v < imm as u64 {
                    true => 1,
                    false => 0,
                };
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn SRLI(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"SRLI",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1);
                // the shift amount is encoded in the lower 6 bits of the I-immediate field for RV64I.
                let mask = match core.xlen {
                    Xlen::Bits32 => 0x1f,
                    Xlen::Bits64 => 0x3f,
                };

                let shamt = (args.imm12) & mask;
                let value = (rs1v as u64) >> shamt;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn SRAI(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"SRAI",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1);
                // the shift amount is encoded in the lower 6 bits of the I-immediate field for RV64I.
                let mask = match core.xlen {
                    Xlen::Bits32 => 0x1f,
                    Xlen::Bits64 => 0x3f,
                };

                let shamt = (args.imm12) & mask;
                let value = ((rs1v as i64).wrapping_shr(shamt as u32)) as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn SLLIW(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"SLLIW",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as i32;
                // the shift amount is encoded in the lower 6 bits of the I-immediate field for RV64I.
                let shamt = args.imm12 & 0b111111;
                let value = ((rs1v as i64) << shamt) as i32 as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn SRAIW(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"SRAIW",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as i32;
                // the shift amount is encoded in the lower 6 bits of the I-immediate field for RV64I.
                let mask = match core.xlen {
                    Xlen::Bits32 => 0x1f,
                    Xlen::Bits64 => 0x3f,
                };

                let shamt = (args.imm12) & mask;
                println!("SRAIW: shamt: {:?}", shamt);
                //                let shamt = args.imm12 & 0b111111;
                let value = ((rs1v as i32).wrapping_shr(shamt as u32)) as i32 as i64 as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn SRLIW(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"SRLIW",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as i32;
                // the shift amount is encoded in the lower 6 bits of the I-immediate field for RV64I.
                let shamt = args.imm12 & 0b111111;
                let value = ((rs1v as u32) >> shamt) as i32 as i64 as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn LB(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"LB",
            args: Some(*args),
            funct: |core, args| {
                let se_imm12 = (args.imm12 as u64).sign_extend(64 - 12) as i64;
                let rs1v = core.read_register(args.rs1);
                let addr = (rs1v as i64 + se_imm12) as u64;
                // instruction_trace!(println!(
                //     "LD: rs1v={:#x?} se_imm12={:#x?} addr={:#x?}",
                //     rs1v, se_imm12, addr
                // ));
                Stage::MEMORY(crate::pipeline::MemoryAccess::READ8(addr, args.rd, true))
            },
        }
    }

    pub fn LH(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"LH",
            args: Some(*args),
            funct: |core, args| {
                let se_imm12 = (args.imm12 as u64).sign_extend(64 - 12) as i64;
                let rs1v = core.read_register(args.rs1);
                let addr = (rs1v as i64 + se_imm12) as u64;
                // instruction_trace!(println!(
                //     "LD: rs1v={:#x?} se_imm12={:#x?} addr={:#x?}",
                //     rs1v, se_imm12, addr
                // ));
                Stage::MEMORY(crate::pipeline::MemoryAccess::READ16(addr, args.rd, true))
            },
        }
    }

    pub fn LHU(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"LHU",
            args: Some(*args),
            funct: |core, args| {
                let se_imm12 = (args.imm12 as u64).sign_extend(64 - 12) as i64;
                let rs1v = core.read_register(args.rs1);
                let addr = (rs1v as i64 + se_imm12) as u64;
                // instruction_trace!(println!(
                //     "LD: rs1v={:#x?} se_imm12={:#x?} addr={:#x?}",
                //     rs1v, se_imm12, addr
                // ));
                Stage::MEMORY(crate::pipeline::MemoryAccess::READ16(addr, args.rd, false))
            },
        }
    }

    pub fn LD(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"LD",
            args: Some(*args),
            funct: |core, args| {
                let se_imm12 = (args.imm12 as u64).sign_extend(64 - 12) as i64;
                let rs1v = core.read_register(args.rs1);
                let addr = (rs1v as i64 + se_imm12) as u64;
                instruction_trace!(println!(
                    "LD: rs1v={:#x?} se_imm12={:#x?} addr={:#x?}",
                    rs1v, se_imm12, addr
                ));
                Stage::MEMORY(crate::pipeline::MemoryAccess::READ64(addr, args.rd, false))
            },
        }
    }

    pub fn LWU(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"LW",
            args: Some(*args),
            funct: |core, args| {
                let se_imm12 = (args.imm12 as u64).sign_extend(64 - 12) as i64;
                let rs1v = core.read_register(args.rs1);
                let addr = (rs1v as i64 + se_imm12) as u64;
                // instruction_trace!(println!(
                //     "LW: rs1v={:#x?} se_imm12={:#x?} addr={:#x?}",
                //     rs1v, se_imm12, addr
                // ));
                Stage::MEMORY(crate::pipeline::MemoryAccess::READ32(addr, args.rd, false))
            },
        }
    }

    pub fn LW(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"LW",
            args: Some(*args),
            funct: |core, args| {
                let se_imm12 = (args.imm12 as u64).sign_extend(64 - 12) as i64;
                let rs1v = core.read_register(args.rs1);
                let addr = (rs1v as i64 + se_imm12) as u64;
                // instruction_trace!(println!(
                //     "LW: rs1v={:#x?} se_imm12={:#x?} addr={:#x?}",
                //     rs1v, se_imm12, addr
                // ));
                Stage::MEMORY(crate::pipeline::MemoryAccess::READ32(addr, args.rd, true))
            },
        }
    }

    pub fn LBU(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"LBU",
            args: Some(*args),
            funct: |core, args| {
                let se_imm12 = (args.imm12 as u64).sign_extend(64 - 12) as i64;
                let rs1v = core.read_register(args.rs1);
                let addr = (rs1v as i64 + se_imm12) as u64;
                // instruction_trace!(println!(
                //     "LBU: rs1v={:#x?} se_imm12={:#x?} addr={:#x?}",
                //     rs1v, se_imm12, addr
                // ));
                Stage::MEMORY(crate::pipeline::MemoryAccess::READ8(addr, args.rd, false))
            },
        }
    }

    pub fn EBREAK(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: "EBREAK",
            args: Some(*args),
            funct: |_core, _args| Stage::TRAP(crate::cpu::TrapCause::Breakpoint),
        }
    }

    pub fn SFENCE_WMA(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: "SFENCE.VMA",
            args: Some(*args),
            funct: |_core, _args| {
                // NOP for now
                Stage::WRITEBACK(None)
            },
        }
    }

    pub fn ECALL(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: "ECALL",
            args: Some(*args),
            funct: |core, _args| {
                let cause = match core.pmode() {
                    crate::cpu::PrivMode::User => TrapCause::EnvCallFromUMode,
                    crate::cpu::PrivMode::Supervisor => TrapCause::EnvCallFromSMode,
                    crate::cpu::PrivMode::Reserved => panic!(),
                    crate::cpu::PrivMode::Machine => TrapCause::EnvCallFromMMode,
                };
                // NOP for now
                Stage::TRAP(cause)
            },
        }
    }
    pub fn MRET(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: "MRET",
            args: Some(*args),
            funct: |core, _args| {
                core.set_pc(core.read_csr(CSRRegister::mepc));

                let status = core.read_csr(CSRRegister::mstatus);
                let mpie = (status >> 7) & 1;
                let mpp = (status >> 11) & 3;
                let mprv = match core.pmode() {
                    crate::cpu::PrivMode::Machine => (status >> 17) & 1,
                    _ => 0,
                };
                // Write MPIE[7] to MIE[3], set MPIE[7] to 1, set MPP[12:11] to 0 and write 1 to MPRV[17]
                let new_status = (status & !0x21888) | (mprv << 17) | (mpie << 3) | (1 << 7);
                core.write_csr(CSRRegister::mstatus, new_status);

                // mpp is the privilege level the CPU was in prior to trapping to machine privilege mode
                core.set_pmode(num::FromPrimitive::from_u8(mpp as u8).unwrap());

                Stage::WRITEBACK(None)
            },
        }
    }

    pub fn SRET(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: "SRET",
            args: Some(*args),
            funct: |core, _args| {
                core.set_pc(core.read_csr(CSRRegister::sepc));

                let status = core.read_csr(CSRRegister::sstatus);
                let spie = (status >> 5) & 1;
                let spp = (status >> 8) & 1;
                let mprv = match core.pmode() {
                    crate::cpu::PrivMode::Machine => (status >> 17) & 1,
                    _ => 0,
                };
                // Write MPIE[7] to MIE[3], set MPIE[7] to 1, set MPP[12:11] to 0 and write 1 to MPRV[17]
                let new_status = (status & !0x21888) | (mprv << 17) | (spie << 3) | (1 << 7);
                core.write_csr(CSRRegister::sstatus, new_status);

                // mpp is the privilege level the CPU was in prior to trapping to machine privilege mode
                core.set_pmode(num::FromPrimitive::from_u8(spp as u8).unwrap());

                Stage::WRITEBACK(None)
            },
        }
    }

    pub fn FENCE(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: "FENCE",
            args: Some(*args),
            funct: |_core, _args| Stage::WRITEBACK(None),
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
                "{} x{},x{},{:#x}",
                self.mnemonic, args.rd, args.rs1, args.imm12,
            )
        }
    }
}

impl InstructionSelector<Itype> for Itype {
    fn select(&self, _xlen: Xlen) -> Instruction<Itype> {
        match self.opcode {
            MajorOpcode::OP_IMM => match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
                OpImm_Funct3::ADDI => Instruction::ADDI(self),
                OpImm_Funct3::ANDI => Instruction::ANDI(self),
                OpImm_Funct3::ORI => Instruction::ORI(self),
                OpImm_Funct3::XORI => Instruction::XORI(self),
                OpImm_Funct3::SLLI => Instruction::SLLI(self),
                OpImm_Funct3::SRLI_SRAI => {
                    //"a specialization of the I-type format"
                    // "The right shift type is encoded in bit 30"
                    match num::FromPrimitive::from_u8(self.funct7 as u8 & 0b1111110).unwrap() {
                        Funct7::B0000000 => Instruction::SRLI(self),
                        Funct7::B0100000 => Instruction::SRAI(self),
                        _ => panic!("{:?} is unknown Funct7 for SRLI_SRAI", self.funct7),
                    }
                }
                OpImm_Funct3::SLTI => Instruction::SLTI(self),
                OpImm_Funct3::SLTIU => Instruction::SLTIU(self),
            },
            MajorOpcode::SYSTEM => match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
                CSR_Funct3::CSRRS => Instruction::CSRRS(self),
                CSR_Funct3::CSRRW => Instruction::CSRRW(self),
                CSR_Funct3::CSRWI => Instruction::CSRWI(self),
                CSR_Funct3::CSRRC => Instruction::CSRRC(self),
                CSR_Funct3::ECALL_EBREAK_MRET => match self.funct7 {
                    Funct7::B0001000 => Instruction::SRET(self),
                    Funct7::B0011000 => Instruction::MRET(self),
                    Funct7::B0001001 => Instruction::SFENCE_WMA(self),
                    //Funct7::B0000000 => Instruction::ECALL(self),
                    _ => {
                        //println!("self.funct7 : {:#?}", self.funct7);
                        Instruction::EBREAK(self)
                    } //_ => panic!(),
                },
                _ => panic!("Unknown SYSTEM instruction"),
            },
            MajorOpcode::JALR => Instruction::JALR(self),
            MajorOpcode::OP_IMM_32 => match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap()
            {
                OpImm32_Funct3::ADDIW => Instruction::ADDIW(self),
                OpImm32_Funct3::SLLIW => Instruction::SLLIW(self),
                OpImm32_Funct3::SRLIW_SRAIW => {
                    match num::FromPrimitive::from_u8(self.funct7 as u8).unwrap() {
                        Funct7::B0000000 => Instruction::SRLIW(self),
                        Funct7::B0100000 => Instruction::SRAIW(self),
                        _ => panic!(),
                    }
                }
            },
            MajorOpcode::LOAD => match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
                Load_Funct3::LB => Instruction::LB(self),
                Load_Funct3::LH => Instruction::LH(self),
                Load_Funct3::LD => Instruction::LD(self),
                Load_Funct3::LW => Instruction::LW(self),
                Load_Funct3::LWU => Instruction::LWU(self),
                Load_Funct3::LBU => Instruction::LBU(self),
                Load_Funct3::LHU => Instruction::LHU(self),
            },
            MajorOpcode::MISC_MEM => {
                match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
                    MiscMem_Funct3::FENCE => Instruction::FENCE(self),
                    MiscMem_Funct3::FENCE_I => todo!(),
                }
            }
            _ => panic!(),
        }
    }
}

impl InstructionExcecutor<Itype> for Instruction<Itype> {
    fn run(&self, core: &mut Core) -> Stage {
        instruction_trace!(println!("{}", self.to_string()));
        (self.funct)(core, &self.args.unwrap())
    }
}
