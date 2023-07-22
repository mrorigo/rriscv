use elfloader::VAddr;
use quark::Signs;

use crate::{
    cpu::{CSRRegister, Core, MipMask, PrivMode, Register, RegisterValue, TrapCause, Xlen},
    instructions::{
        decoder::{DecodedInstruction, InstructionDecoder},
        InstructionSelector,
    },
    memory::{MemoryOperations, RAMOperations},
    mmu::MMU,
};

macro_rules! pipeline_trace {
    ($instr:expr) => {
        // print!("P: ");
        // $instr;
    };
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum MemoryAccess {
    AMOSWAP_W(VAddr, RegisterValue, Register),
    AMOSWAP_D(VAddr, VAddr, Register),
    READ8(VAddr, Register, bool),
    READ16(VAddr, Register, bool),
    READ32(VAddr, Register, bool),
    READ64(VAddr, Register, bool),
    WRITE8(VAddr, u8),
    WRITE16(VAddr, u16),
    WRITE32(VAddr, u32),
    WRITE64(VAddr, u64),
}

#[derive(Debug, Copy, Clone)]
pub enum MemoryAccessWidth {
    BYTE,     // 8 bits
    HALFWORD, // 16 bits
    WORD,     // 32 bits
    LONG,
}

#[derive(Debug, Copy, Clone)]
pub struct RawInstruction {
    pub compressed: bool,
    pub word: u32,
    pub pc: u64,
}

impl RawInstruction {
    pub fn size_in_bytes(&self) -> u64 {
        match self.compressed {
            true => 2,
            false => 4,
        }
    }

    pub fn from_word(instruction: u32, pc: u64) -> Self {
        match (instruction & 0x3) == 0x03 {
            true => {
                pipeline_trace!(println!(
                    "f:     instruction @ {:#x}: {:#x}",
                    pc, instruction
                ));
                RawInstruction {
                    compressed: false,
                    word: instruction,
                    pc,
                }
            }
            false => {
                pipeline_trace!(println!(
                    "f:     instruction @ {:#x?}: {:#x?} (C)",
                    pc,
                    instruction & 0xffff,
                ));
                RawInstruction {
                    compressed: true,
                    word: instruction & 0xffff,
                    pc,
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct WritebackStage {
    pub register: Register,
    pub value: u64,
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Stage {
    TRAP(TrapCause),
    FETCH,
    DECODE(RawInstruction),
    EXECUTE(DecodedInstruction),       // May be skipped (by eg NOP)
    MEMORY(MemoryAccess),              // May be skipped
    WRITEBACK(Option<WritebackStage>), // This stage is ALWAYS executed
    IRQ,
}

impl Stage {
    pub fn writeback(register: Register, value: RegisterValue) -> Stage {
        Stage::WRITEBACK(Some(WritebackStage { register, value }))
    }
}

pub trait PipelineStages {
    fn fetch(&mut self, mmu: &mut MMU) -> Stage;
    fn decode(&mut self, instruction: &RawInstruction) -> Stage;
    fn execute(&mut self, decoded: &DecodedInstruction) -> Stage;
    fn memory(&mut self, mmu: &mut MMU, memory_access: &MemoryAccess) -> Stage;

    fn writeback(&mut self, writeback: Option<WritebackStage>) -> Stage;
    fn trap(&mut self, cause: TrapCause) -> Stage;
}

impl PipelineStages for Core {
    fn fetch(&mut self, mmu: &mut MMU) -> Stage {
        match mmu.fetch(self.pc()) {
            Ok(word) => {
                let ri = RawInstruction::from_word(word, self.pc());
                self.prev_pc = self.pc();
                self.add_pc(ri.size_in_bytes());
                Stage::DECODE(ri)
            }
            _ => Stage::TRAP(TrapCause::InstructionAccessFault),
        }
    }

    fn decode(&mut self, instruction: &RawInstruction) -> Stage {
        let decoded = InstructionDecoder::decode_instruction(*instruction);
        pipeline_trace!(println!("d:    {:?}", decoded));

        Stage::EXECUTE(decoded)
    }

    fn execute(&mut self, decoded: &DecodedInstruction) -> Stage {
        match *decoded {
            DecodedInstruction::I(typ) => ((typ.select(self.xlen)).funct)(self, &typ),
            DecodedInstruction::U(typ) => ((typ.select(self.xlen)).funct)(self, &typ),
            DecodedInstruction::CI(typ) => ((typ.select(self.xlen)).funct)(self, &typ),
            DecodedInstruction::J(typ) => ((typ.select(self.xlen)).funct)(self, &typ),
            DecodedInstruction::CR(typ) => ((typ.select(self.xlen)).funct)(self, &typ),
            DecodedInstruction::B(typ) => ((typ.select(self.xlen)).funct)(self, &typ),
            DecodedInstruction::S(typ) => ((typ.select(self.xlen)).funct)(self, &typ),
            DecodedInstruction::R(typ) => ((typ.select(self.xlen)).funct)(self, &typ),
            DecodedInstruction::CSS(typ) => ((typ.select(self.xlen)).funct)(self, &typ),
            DecodedInstruction::CIW(typ) => ((typ.select(self.xlen)).funct)(self, &typ),
            DecodedInstruction::CL(typ) => ((typ.select(self.xlen)).funct)(self, &typ),
            DecodedInstruction::CS(typ) => ((typ.select(self.xlen)).funct)(self, &typ),
            DecodedInstruction::CB(typ) => ((typ.select(self.xlen)).funct)(self, &typ),
            DecodedInstruction::CJ(typ) => ((typ.select(self.xlen)).funct)(self, &typ),
        }
    }

    fn memory(&mut self, mmu: &mut MMU, memory_access: &MemoryAccess) -> Stage {
        match *memory_access {
            MemoryAccess::READ8(offset, register, sign_extend) => {
                let value = mmu.read8(offset);
                if value.is_err() {
                    return Stage::TRAP(value.err().unwrap());
                }
                // pipeline_trace!(println!("m:    READ8 @ {:#x?}: {:#x?}", offset, value));
                let value = value.unwrap();
                Stage::WRITEBACK(Some(WritebackStage {
                    register: register,
                    value: match sign_extend {
                        false => value as u64,
                        true => value as i8 as i16 as i32 as u64,
                    },
                }))
            }
            MemoryAccess::READ16(offset, register, sign_extend) => {
                let h = mmu.read8(offset + 1).unwrap() as u16;
                let l = mmu.read8(offset).unwrap() as u16;
                let value = (h << 8 | l) as i16 as u64;
                // pipeline_trace!(println!("m:    READ16 @ {:#x?}: {:#x?}", offset, value));

                Stage::WRITEBACK(Some(WritebackStage {
                    register: register,
                    value: match sign_extend {
                        false => (value & 0xffff) as u64,
                        true => value as i16 as i32 as u64,
                    },
                }))
                // let v = self.mmu.read_16(offset).unwrap();
                // Stage::WRITEBACK(Some(WritebackStage {
                //     register: register,
                //     value: v as u64,
                // }))
            }
            MemoryAccess::READ32(offset, register, sign_extend) => match mmu.read_32(offset) {
                Ok(value) => Stage::WRITEBACK(Some(WritebackStage {
                    register: register,
                    value: match sign_extend {
                        true => value as i32 as i64 as u64,
                        false => (value & 0xffffffff) as u64,
                    },
                })),
                Err(cause) => Stage::TRAP(cause),
            },

            MemoryAccess::READ64(offset, register, sign_extend) => {
                let l = match mmu.read_32(offset) {
                    Err(cause) => return Stage::TRAP(cause),
                    Ok(val) => val,
                };
                let h = match mmu.read_32(offset + 4) {
                    Err(cause) => return Stage::TRAP(cause),
                    Ok(val) => val,
                };

                let comp = ((h as u64) << 32) | l as u64;
                let value = match sign_extend {
                    true => comp.sign_extend(64 - 32),
                    false => comp as u64,
                };
                // pipeline_trace!(println!(
                //     "m:    READ64 @ {:#x?}: {:#x?} ({:?})",
                //     offset, value, sign_extend
                // ));
                Stage::WRITEBACK(Some(WritebackStage {
                    register: register,
                    value,
                }))
            }
            MemoryAccess::WRITE8(offset, value) => {
                pipeline_trace!(println!("m:    WRITE8 @ {:#x?}: {:#x}", offset, value));
                mmu.write8(offset, value);
                Stage::WRITEBACK(None)
            }
            MemoryAccess::WRITE16(offset, value) => {
                mmu.write8(offset + 1, (value >> 8) as u8);
                mmu.write8(offset, (value & 0xff) as u8);
                Stage::WRITEBACK(None)
            }
            MemoryAccess::WRITE32(offset, value) => {
                pipeline_trace!(println!("m:    WRITE32 @ {:#x?}: {:#x?}", offset, value));
                mmu.write_32(offset, value);
                Stage::WRITEBACK(None)
            }
            MemoryAccess::WRITE64(offset, value) => {
                pipeline_trace!(println!("m:    WRITE64 @ {:#x?}: {:#x?}", offset, value));
                mmu.write_32(offset + 0, value as u32);
                mmu.write_32(offset + 4, (value >> 32) as u32);
                Stage::WRITEBACK(None)
            }
            MemoryAccess::AMOSWAP_W(from, rs2v, rd) => {
                let v1 = mmu.read_32(from).unwrap();
                //let v2 = mmu.read_32(to).unwrap();
                mmu.write_32(from, rs2v as u32);
                pipeline_trace!(println!(
                    "m:    AMOSWAP.W @ {:#x?} = {:#x?} now {:#x?}",
                    from, v1, rs2v
                ));

                // "AMOs can be used to implement parallel reduction operations,
                //   where typically the return value would be discarded by writing to x0."
                if rd == 0 {
                    Stage::WRITEBACK(None)
                } else {
                    //  AMOs can either operate on 64-bit (RV64 only) or 32-bit words in memory.
                    // For RV64, 32-bit AMOs always sign-extend the value placed in rd. T
                    let value = match self.xlen {
                        Xlen::Bits32 => v1 as u64,
                        Xlen::Bits64 => v1 as i32 as i64 as u64,
                    };
                    Stage::writeback(rd, value as u64)
                }
            }
            MemoryAccess::AMOSWAP_D(_from, _to, _rd) => todo!(),
        }
    }

    fn writeback(&mut self, writeback: Option<WritebackStage>) -> Stage {
        match writeback {
            Some(wb) if wb.register > 0 => {
                pipeline_trace!(println!("w: x{} = {:#x?}", wb.register, wb.value));

                self.write_register(wb.register, wb.value)
            }
            Some(wb) if wb.register == 0 => { /*warn*/ }
            _ => {}
        }

        self.update_instret();

        Stage::IRQ
    }

    fn trap(&mut self, cause: TrapCause) -> Stage {
        let csr_cause = self.get_mcause(self.xlen, cause);
        let mip_mask = match cause {
            TrapCause::SupervisorExternalIrq => Some(MipMask::SEIP),
            TrapCause::SupervisorTimerIrq => Some(MipMask::STIP),
            TrapCause::MachineExternalIrq => Some(MipMask::MEIP),
            TrapCause::MachineTimerIrq => Some(MipMask::MTIP),
            _ => None,
        };
        println!("TRAP: {:#x?} @ {:#x?}", cause, self.prev_pc);

        let is_interrupt = mip_mask.is_some();

        let mdeleg = match is_interrupt {
            true => self.read_csr(CSRRegister::mideleg),
            false => self.read_csr(CSRRegister::medeleg),
        };

        let sdeleg = match is_interrupt {
            true => self.read_csr(CSRRegister::sideleg),
            false => self.read_csr(CSRRegister::sedeleg),
        };

        // @TODO: We might need to change privmode!
        let bit = csr_cause & 0xffff;

        let old_privilege_mode = self.pmode();
        let mut epc_value = self.pc();

        let new_privilege_mode = match ((mdeleg >> bit) & 1) == 0 {
            true => PrivMode::Machine,
            false => match ((sdeleg >> bit) & 1) == 0 {
                true => PrivMode::Supervisor,
                false => PrivMode::User,
            },
        };

        let current_status = match old_privilege_mode {
            PrivMode::Machine => self.read_csr(CSRRegister::mstatus),
            PrivMode::Supervisor => self.read_csr(CSRRegister::sstatus),
            PrivMode::User => self.read_csr(CSRRegister::ustatus),
            PrivMode::Reserved => panic!(),
        };

        // Mask interrupts
        if is_interrupt {
            let ie = match new_privilege_mode {
                PrivMode::Machine => self.read_csr(CSRRegister::mie),
                PrivMode::Supervisor => self.read_csr(CSRRegister::sie),
                PrivMode::User => self.read_csr(CSRRegister::uie),
                PrivMode::Reserved => panic!(),
            };

            let current_mie = (current_status >> 3) & 1;
            let current_sie = (current_status >> 1) & 1;
            let current_uie = current_status & 1;

            println!(
                "IRQ!, status: {:#x}  sie: {:#x?}",
                current_status, current_sie
            );

            // Unmask IRQ from mip
            self.write_csr(
                CSRRegister::mip,
                self.read_csr(CSRRegister::mip) & !(mip_mask.unwrap() as u64),
            );

            if new_privilege_mode < old_privilege_mode {
                panic!("Ignore irq!?");
                return Stage::FETCH;
            } else if old_privilege_mode == new_privilege_mode {
                match old_privilege_mode {
                    PrivMode::Machine if current_mie == 0 => return Stage::FETCH,
                    PrivMode::Supervisor if current_sie == 0 => {
                        panic!("Ignore irq 2!?");
                        return Stage::FETCH;
                    }
                    PrivMode::User if current_uie == 0 => return Stage::FETCH,
                    _ => {} // Non-masked
                }
            }
            println!("IRQ NOT masked out!");

            let msie = (ie >> 3) & 1;
            let ssie = current_sie;
            let usie = ie & 1;

            let mtie = (ie >> 7) & 1;
            let stie = (ie >> 5) & 1;
            let utie = (ie >> 4) & 1;

            let meie = (ie >> 11) & 1;
            let seie = (ie >> 9) & 1;
            let ueie = (ie >> 8) & 1;

            match cause {
                TrapCause::UserSoftwareIrq if usie == 0 => return Stage::FETCH,
                TrapCause::SupervisorSoftIrq if ssie == 0 => return Stage::FETCH,
                TrapCause::MachineSoftIrq if msie == 0 => return Stage::FETCH,
                TrapCause::UserTimerIrq if utie == 0 => return Stage::FETCH,
                TrapCause::SupervisorTimerIrq if stie == 0 => return Stage::FETCH,
                TrapCause::MachineTimerIrq if mtie == 0 => return Stage::FETCH,
                TrapCause::UserExternalIrq if ueie == 0 => return Stage::FETCH,
                TrapCause::SupervisorExternalIrq if seie == 0 => return Stage::FETCH,
                TrapCause::MachineExternalIrq if meie == 0 => return Stage::FETCH,
                _ => {} // Not masked!
            }
        } else {
            // Not sure how to implement this, but epc should be instruction address for EBREAK/ECALL
            epc_value = match cause {
                TrapCause::Breakpoint
                | TrapCause::EnvCallFromUMode
                | TrapCause::EnvCallFromSMode
                | TrapCause::EnvCallFromMMode => epc_value.wrapping_sub(4),
                _ => epc_value,
            };
        }
        println!("IRQ NOT masked out 2!");

        // Masking passed, execute trap
        self.set_pmode(new_privilege_mode);

        let epc_address = match new_privilege_mode {
            PrivMode::Machine => CSRRegister::mepc,
            PrivMode::Supervisor => CSRRegister::sepc,
            PrivMode::User => CSRRegister::uepc,
            _ => panic!(),
        };

        let cause_reg = match new_privilege_mode {
            PrivMode::Machine => CSRRegister::mcause,
            PrivMode::Supervisor => CSRRegister::scause,
            PrivMode::User => CSRRegister::ucause,
            _ => panic!(),
        };

        let tval_reg = match new_privilege_mode {
            PrivMode::Machine => CSRRegister::mtval,
            PrivMode::Supervisor => CSRRegister::stval,
            PrivMode::User => CSRRegister::utval,
            _ => panic!(),
        };

        let tvec_reg = match new_privilege_mode {
            PrivMode::Machine => CSRRegister::mtvec,
            PrivMode::Supervisor => CSRRegister::stvec,
            PrivMode::User => CSRRegister::utvec,
            _ => panic!(),
        };

        self.write_csr(epc_address, epc_value);
        self.write_csr(cause_reg, csr_cause);
        self.write_csr(tval_reg, self.pc()); // @TODO: Not always correct (could be -4 for IllegalInstruction etc?)

        let tvec_val = self.read_csr(tvec_reg);
        self.set_pc(tvec_val);
        //println!("trap: tvec_reg={:?}", tvec_reg);
        // Add 4 * cause if tvec has vector type address
        if (tvec_val & 0x3) != 0 {
            self.set_pc((tvec_val & !0x3) + 4 * (csr_cause & 0xffff));
        }

        match self.pmode() {
            PrivMode::Machine => {
                let status = self.read_csr(CSRRegister::mstatus);
                let mie = (status >> 3) & 1;
                let new_status =
                    (status & !0x1888) | (mie << 7) | (u64::from(old_privilege_mode) << 11);
                self.write_csr(CSRRegister::mstatus, new_status);
            }
            PrivMode::Supervisor => {
                let status = self.read_csr(CSRRegister::sstatus);
                let sie = (status >> 1) & 1;
                let new_status = (status & !0x122) | (sie << 5) | ((1) << 8);
                self.write_csr(CSRRegister::sstatus, new_status);
            }
            _ => panic!(),
        }

        Stage::FETCH
    }
}
