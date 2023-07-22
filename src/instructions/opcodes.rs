pub const FUNCT7_SHIFT: u8 = 11;
pub const FUNCT7_MASK: u16 = (1 << FUNCT7_SHIFT) - 1;
//const FUNCT7: u16 = 1 << FUNCT7_SHIFT;

pub const FUNCT3_SHIFT: u8 = 7;
pub const FUNCT3_MASK: u8 = (1 << FUNCT3_SHIFT) - 1;
//const FUNCT7: u16 = 1 << FUNCT7_SHIFT;

/// Table 19.1: RISC-V base opcode map
#[derive(Debug, Copy, Clone, FromPrimitive, PartialEq)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum MajorOpcode {
    AUIPC = 0b0010111,
    BRANCH = 0b1100011,
    LOAD = 0b0000011,
    JALR = 0b1100111,
    JAL = 0b1101111,
    OP = 0b0110011,
    AMO = 0b0101111,
    OP_32 = 0b0111011,
    // OP_FP,
    OP_IMM = 0b0010011,
    OP_IMM_32 = 0b0011011,
    LUI = 0b0110111,
    MISC_MEM = 0b0001111,
    //MADD,
    //MSUB,
    STORE = 0b0100011,
    SYSTEM = 0b1110011,
}

#[derive(Debug, Copy, Clone, FromPrimitive, PartialEq)]
#[repr(u8)]
pub enum CompressedOpcode {
    C0 = 0b00,
    C1 = 0b01,
    C2 = 0b10,
    C3 = 0b11,
}

impl MajorOpcode {}
