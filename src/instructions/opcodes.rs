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
    AMO = 0b101111,
    // OP_32,
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

/// Each opcode encoded together with the funct3 and func7 bits as a 32 bit number:
/// Bits 0-7:  opcode
/// Bits 8-10: funct3
/// Bit 11-18: funct7
#[repr(u32)]
#[derive(Debug, Copy, Clone, FromPrimitive, PartialEq, Eq)]
pub enum OpCodes {
    NONE = 0,
    ILLEGAL = 1 << 31,
    // RV32I Base Instruction Set
    LUI = MajorOpcode::LUI as u32 | (0b000 << FUNCT3_SHIFT),

    AUIPC = MajorOpcode::AUIPC as u32 | (0b000 << FUNCT3_SHIFT),
    JAL = MajorOpcode::JAL as u32 | (0b000 << FUNCT3_SHIFT),
    JALR = MajorOpcode::JALR as u32 | (0b000 << FUNCT3_SHIFT),

    BEQ = MajorOpcode::BRANCH as u32 | (0b000 << FUNCT3_SHIFT),
    BNE = MajorOpcode::BRANCH as u32 | (0b001 << FUNCT3_SHIFT),
    BLT = MajorOpcode::BRANCH as u32 | (0b100 << FUNCT3_SHIFT),
    BGE = MajorOpcode::BRANCH as u32 | (0b101 << FUNCT3_SHIFT),
    BLTU = MajorOpcode::BRANCH as u32 | (0b110 << FUNCT3_SHIFT),
    BGEU = MajorOpcode::BRANCH as u32 | (0b111 << FUNCT3_SHIFT),

    LB = (MajorOpcode::LOAD as u32) | (0b000 << FUNCT3_SHIFT),
    LH = (MajorOpcode::LOAD as u32) | (0b001 << FUNCT3_SHIFT),
    LW = (MajorOpcode::LOAD as u32) | (0b010 << FUNCT3_SHIFT),
    LBU = (MajorOpcode::LOAD as u32) | (0b100 << FUNCT3_SHIFT),
    LHU = (MajorOpcode::LOAD as u32) | (0b101 << FUNCT3_SHIFT),

    SB = MajorOpcode::STORE as u32 | (0b000 << FUNCT3_SHIFT),
    SH = MajorOpcode::STORE as u32 | (0b001 << FUNCT3_SHIFT),
    SW = MajorOpcode::STORE as u32 | (0b010 << FUNCT3_SHIFT),

    ADDI = MajorOpcode::OP_IMM as u32 | (0b000 << FUNCT3_SHIFT),

    SLTI = MajorOpcode::OP_IMM as u32 | (0b010 << FUNCT3_SHIFT),
    SLTIU = MajorOpcode::OP_IMM as u32 | (0b011 << FUNCT3_SHIFT),
    XORI = MajorOpcode::OP_IMM as u32 | (0b100 << FUNCT3_SHIFT),
    ORI = MajorOpcode::OP_IMM as u32 | (0b110 << FUNCT3_SHIFT),
    ANDI = MajorOpcode::OP_IMM as u32 | (0b111 << FUNCT3_SHIFT),

    SLLI = MajorOpcode::OP_IMM as u32 | (0b001 << FUNCT3_SHIFT),
    SRLI = MajorOpcode::OP_IMM as u32 | (0b101 << FUNCT3_SHIFT),
    SRAI = MajorOpcode::OP_IMM as u32 | (0b101 << FUNCT3_SHIFT) | (0b0100000 << FUNCT7_SHIFT),

    ADD = MajorOpcode::OP as u32 | (0b000 << FUNCT3_SHIFT),
    SUB = MajorOpcode::OP as u32 | (0b000 << FUNCT3_SHIFT) | (0b0100000 << FUNCT7_SHIFT),
    SLL = MajorOpcode::OP as u32 | (0b001 << FUNCT3_SHIFT) | (0b0000000 << FUNCT7_SHIFT),
    SLT = MajorOpcode::OP as u32 | (0b010 << FUNCT3_SHIFT),
    SLTU = MajorOpcode::OP as u32 | (0b011 << FUNCT3_SHIFT),
    XOR = MajorOpcode::OP as u32 | (0b100 << FUNCT3_SHIFT),
    SRL = MajorOpcode::OP as u32 | (0b101 << FUNCT3_SHIFT) | (0b0000000 << FUNCT7_SHIFT),
    SRA = MajorOpcode::OP as u32 | (0b101 << FUNCT3_SHIFT) | (0b0100000 << FUNCT7_SHIFT),
    OR = MajorOpcode::OP as u32 | (0b110 << FUNCT3_SHIFT),
    AND = MajorOpcode::OP as u32 | (0b111 << FUNCT3_SHIFT),

    FENCE = MajorOpcode::MISC_MEM as u32 | (0b000 << FUNCT3_SHIFT),
    FENCEI = MajorOpcode::MISC_MEM as u32 | (0b001 << FUNCT3_SHIFT),

    ECALL = MajorOpcode::SYSTEM as u32 | (0b000 << FUNCT3_SHIFT),
    EBREAK = MajorOpcode::SYSTEM as u32 | (0b000 << FUNCT3_SHIFT) | (1 << FUNCT7_SHIFT),

    CSRRW = MajorOpcode::SYSTEM as u32 | (0b001 << FUNCT3_SHIFT),
    CSRRS = MajorOpcode::SYSTEM as u32 | (0b010 << FUNCT3_SHIFT),
    CSRRC = MajorOpcode::SYSTEM as u32 | (0b011 << FUNCT3_SHIFT),
    CSRRWI = MajorOpcode::SYSTEM as u32 | (0b101 << FUNCT3_SHIFT),
    CSRRSI = MajorOpcode::SYSTEM as u32 | (0b110 << FUNCT3_SHIFT),
    CSRRCI = MajorOpcode::SYSTEM as u32 | (0b111 << FUNCT3_SHIFT),

    // RV64I Base Instruction Set (in addition to RV32I)
    LWU = (MajorOpcode::LOAD as u32) | (0b110 << FUNCT3_SHIFT),
    LD = (MajorOpcode::LOAD as u32) | (0b011 << FUNCT3_SHIFT),
    SD = (MajorOpcode::STORE as u32) | (0b011 << FUNCT3_SHIFT),

    ADDIW = MajorOpcode::OP_IMM_32 as u32 | (0b000 << FUNCT3_SHIFT),

    // RV32M Standard Extension
    MUL = MajorOpcode::OP as u32 | (0b000 << FUNCT3_SHIFT) | (1 << FUNCT7_SHIFT),
    MULH = MajorOpcode::OP as u32 | (0b001 << FUNCT3_SHIFT) | (1 << FUNCT7_SHIFT),
    MULHSU = MajorOpcode::OP as u32 | (0b010 << FUNCT3_SHIFT) | (1 << FUNCT7_SHIFT),
    MULHU = MajorOpcode::OP as u32 | (0b011 << FUNCT3_SHIFT) | (1 << FUNCT7_SHIFT),
    DIV = MajorOpcode::OP as u32 | (0b100 << FUNCT3_SHIFT) | (1 << FUNCT7_SHIFT),
    DIVU = MajorOpcode::OP as u32 | (0b101 << FUNCT3_SHIFT) | (1 << FUNCT7_SHIFT),
    REM = MajorOpcode::OP as u32 | (0b110 << FUNCT3_SHIFT) | (1 << FUNCT7_SHIFT),
    REMU = MajorOpcode::OP as u32 | (0b111 << FUNCT3_SHIFT) | (1 << FUNCT7_SHIFT),

    // RV64M Standard Extension (in addition to RV32M)
    MULW = MajorOpcode::OP_IMM_32 as u32 | (0b000 << FUNCT3_SHIFT) | (1 << FUNCT7_SHIFT),
    DIVW = MajorOpcode::OP_IMM_32 as u32 | (0b100 << FUNCT3_SHIFT) | (1 << FUNCT7_SHIFT),
    DIVUW = MajorOpcode::OP_IMM_32 as u32 | (0b101 << FUNCT3_SHIFT) | (1 << FUNCT7_SHIFT),
    REMW = MajorOpcode::OP_IMM_32 as u32 | (0b110 << FUNCT3_SHIFT) | (1 << FUNCT7_SHIFT),
    REMUW = MajorOpcode::OP_IMM_32 as u32 | (0b111 << FUNCT3_SHIFT) | (1 << FUNCT7_SHIFT),
}

impl MajorOpcode {
    #[allow(dead_code)]
    fn encode(&self, op: u8, funct3: u8, funct7: u8) -> OpCodes {
        let word =
            op as u32 | ((funct3 as u32) << FUNCT3_SHIFT) | ((funct7 as u32) << FUNCT7_SHIFT);
        let ret: Option<OpCodes> = num::FromPrimitive::from_u32(word);
        ret.unwrap_or(OpCodes::ILLEGAL)
    }
}
