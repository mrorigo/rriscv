use crate::{
    cpu::{Core, Xlen},
    pipeline::Stage,
};

macro_rules! instruction_trace {
    ($instr:expr) => {
        print!("P:x: ");
        $instr;
    };
}

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
pub mod functions;
pub mod itype;
pub mod jtype;
pub mod map;
pub mod opcodes;
pub mod rtype;
pub mod stype;
pub mod utype;

/// "2.2 Base instruction formats" and "2.3 Immediate Encoding Variants"
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum InstructionFormat {
    Unknown = 0,
    R,
    I,
    S,
    U,
    B,
    J,
}

/// Table 12.1: Compressed 16-bit RVC instruction formats.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum CompressedFormat {
    Unknown = 0,
    CR,
    CI,
    CSS,
    CIW,
    CL,
    CS,
    CB,
    CJ,
}

pub trait InstructionFormatType {}

pub trait UncompressedFormatType: InstructionFormatType {}
pub trait CompressedFormatType: InstructionFormatType {}

pub struct Instruction<T> {
    pub args: Option<T>,
    pub mnemonic: &'static str,
    pub funct: fn(&mut Core, &T) -> Stage,
}

pub trait FormatDecoder<T: UncompressedFormatType> {
    fn decode(word: u32) -> T;
}

pub trait CompressedFormatDecoder<T: CompressedFormatType> {
    fn decode(word: u16) -> T;
}

pub trait ImmediateDecoder<T, T2> {
    fn decode_immediate(i: T) -> T2;
}

// pub trait InstructionExcecutor<T: InstructionFormatType> {
//     fn run(&self, core: &mut Core) -> Stage;
// }

pub trait InstructionSelector<T> {
    fn select(&self, _xlen: Xlen) -> Instruction<T> {
        todo!();
    }
}
