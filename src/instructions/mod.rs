use crate::{cpu::Core, pipeline::Stage};

pub mod btype;
pub mod cbtype;
pub mod citype;
pub mod ciwtype;
pub mod cjtype;
pub mod cltype;
pub mod crtype;
pub mod csstype;
pub mod cstype;
pub mod decoder;
pub mod itype;
pub mod jtype;
pub mod map;
pub mod opcodes;
pub mod rtype;
pub mod stype;
pub mod utype;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum InstructionFormat {
    Unknown = 0,
    // 2.2 Base instruction formats
    R,
    I,
    S,
    U,
    // 2.3 Immediate Encoding Variants
    B,
    J,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum CompressedInstructionFormat {
    Unknown = 0,
    // Table 12.1: Compressed 16-bit RVC instruction formats.
    CR,
    CI,
    CSS,
    CIW,
    CL,
    CS,
    CB,
    CJ,
}
pub struct Instruction<T> {
    args: Option<T>,
    mnemonic: &'static str,
    funct: fn(&mut Core, &T) -> Stage,
}

pub trait ImmediateDecoder<T, T2> {
    fn decode_immediate(i: T) -> T2;
}

pub trait InstructionExcecutor {
    fn run(&self, core: &mut Core) -> Stage;
}

pub trait InstructionSelector<T> {
    fn select(&self) -> Instruction<T> {
        Instruction {
            mnemonic: &"ILLEGAL",
            args: None,
            funct: |_core, _args| panic!("ILLEGAL INSTRUCTION"),
        }
    }
}
