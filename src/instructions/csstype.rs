use crate::{
    cpu::{Core, Register, Xlen},
    pipeline::{MemoryAccess, Stage},
};

use super::{
    opcodes::CompressedOpcode, ImmediateDecoder, Instruction, InstructionExcecutor,
    InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CSStype {
    pub opcode: CompressedOpcode,
    pub uimm: u16,
    pub rs2: Register,
    pub funct3: u8,
}

impl ImmediateDecoder<u16, u16> for CSStype {
    fn decode_immediate(i: u16) -> u16 {
        let offset = ((i >> 7) & 0x38) | // offset[5:3] <= [12:10]
                        ((i >> 1) & 0x1c0); // offset[8:6] <= [9:7]
        let imm11_5 = (offset >> 5) & 0x3f;
        let imm4_0 = offset & 0x1f;
        (imm11_5 << 5) | (imm4_0)
    }
}

#[allow(non_snake_case)]
impl Instruction<CSStype> {
    // C.SDSP is an RV64C/RV128C-only instruction that stores a 64-bit value in register rs2 to memory.
    // It computes an effective address by adding the zero-extended offset, scaled by 8, to the stack pointer,
    // x2. It expands to sd rs2, offset[8:3](x2).
    pub fn C_SDSP(csstype: CSStype) -> Instruction<CSStype> {
        Instruction {
            args: Some(csstype),
            mnemonic: "C.SDSP",
            funct: |core, args| {
                let sp = core.read_register(2);
                let addr = sp + (args.uimm as u64);
                let rs2v = core.read_register(args.rs2);
                debug_trace!(println!(
                    "C.SDSP: args.uimm={:#x?}  rs2v: {:#x?}  sp={:#x?}  addr={:#x?}",
                    args.uimm, rs2v, sp, addr
                ));
                Stage::MEMORY(MemoryAccess::WRITE64(addr, rs2v))
            },
        }
    }

    pub fn C_FSWSP(csstype: CSStype) -> Instruction<CSStype> {
        Instruction {
            args: Some(csstype),
            mnemonic: "C.FSWSP",
            funct: |core, args| todo!(),
        }
    }
}

impl InstructionSelector<CSStype> for CSStype {
    fn select(&self, xlen: Xlen) -> Instruction<CSStype> {
        match self.funct3 {
            // C.FSWSP or C.SDSP
            0b111 => match xlen {
                Xlen::Bits32 => Instruction::C_FSWSP(*self),
                _ => Instruction::C_SDSP(*self),
            },
            _ => panic!(),
        }
    }
}

impl InstructionExcecutor for Instruction<CSStype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}
