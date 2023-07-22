extern crate num;

type OpCode = u16;

pub const FUNCT7_SHIFT: u8 = 11;
pub const FUNCT7_MASK: u16 = (1 << FUNCT7_SHIFT) - 1;
//const FUNCT7: u16 = 1 << FUNCT7_SHIFT;

pub const FUNCT3_SHIFT: u8 = 7;
pub const FUNCT3_MASK: u8 = (1 << FUNCT3_SHIFT) - 1;
//const FUNCT7: u16 = 1 << FUNCT7_SHIFT;

/// Each opcode encoded together with the funct3 and func7 bits
/// as a 16 bit number:
/// Bits 0-7:  opcode
/// Bits 8-10: funct3
/// Bit 11:    funct7
#[repr(u32)]
#[derive(Debug, Copy, Clone, FromPrimitive, PartialEq)]
pub enum OpCodes {
    NONE = 0,
    ILLEGAL = 1 << 31,
    // RV32I Base Instruction Set
    LUI = (0b0110111) | (0b000 << FUNCT3_SHIFT),

    AUIPC = (0b0010111) | (0b000 << FUNCT3_SHIFT),
    JAL = (0b1101111) | (0b000 << FUNCT3_SHIFT),
    JALR = (0b1100111) | (0b000 << FUNCT3_SHIFT),

    BEQ = (0b1100011) | (0b000 << FUNCT3_SHIFT),
    BNE = (0b1100011) | (0b001 << FUNCT3_SHIFT),
    BLT = (0b1100011) | (0b100 << FUNCT3_SHIFT),
    BGE = (0b1100011) | (0b101 << FUNCT3_SHIFT),
    BLTU = (0b1100011) | (0b110 << FUNCT3_SHIFT),
    BGEU = (0b1100011) | (0b111 << FUNCT3_SHIFT),

    LB = (0b0000011) | (0b000 << FUNCT3_SHIFT),
    LH = (0b0000011) | (0b001 << FUNCT3_SHIFT),
    LW = (0b0000011) | (0b010 << FUNCT3_SHIFT),
    LBU = (0b0000011) | (0b100 << FUNCT3_SHIFT),
    LHU = (0b0000011) | (0b101 << FUNCT3_SHIFT),

    SB = (0b0100011) | (0b000 << FUNCT3_SHIFT),
    SH = (0b0100011) | (0b001 << FUNCT3_SHIFT),
    SW = (0b0100011) | (0b010 << FUNCT3_SHIFT),

    ADDI = (0b0010011) | (0b000 << FUNCT3_SHIFT),
    SLTI = (0b0010011) | (0b010 << FUNCT3_SHIFT),
    SLTIU = (0b0010011) | (0b011 << FUNCT3_SHIFT),
    XORI = (0b0010011) | (0b100 << FUNCT3_SHIFT),
    ORI = (0b0010011) | (0b110 << FUNCT3_SHIFT),
    ANDI = (0b0010011) | (0b111 << FUNCT3_SHIFT),

    SLLI = (0b0010011) | (0b001 << FUNCT3_SHIFT),
    SRLI = (0b0010011) | (0b101 << FUNCT3_SHIFT),
    SRAI = (0b0010011) | (0b101 << FUNCT3_SHIFT) | (0b0100000 << FUNCT7_SHIFT),

    ADD = (0b0110011) | (0b000 << FUNCT3_SHIFT),
    SUB = (0b0110011) | (0b000 << FUNCT3_SHIFT) | (0b0100000 << FUNCT7_SHIFT),
    SLL = (0b0110011) | (0b001 << FUNCT3_SHIFT) | (0b0000000 << FUNCT7_SHIFT),
    SLT = (0b0110011) | (0b010 << FUNCT3_SHIFT),
    SLTU = (0b0110011) | (0b011 << FUNCT3_SHIFT),

    XOR = (0b0110011) | (0b100 << FUNCT3_SHIFT),
    SRL = (0b0110011) | (0b101 << FUNCT3_SHIFT) | (0b0000000 << FUNCT7_SHIFT),
    SRA = (0b0110011) | (0b101 << FUNCT3_SHIFT) | (0b0100000 << FUNCT7_SHIFT),
    OR = (0b0110011) | (0b110 << FUNCT3_SHIFT),
    AND = (0b0110011) | (0b111 << FUNCT3_SHIFT),

    FENCE = (0b0001111) | (0b000 << FUNCT3_SHIFT),
    FENCEI = (0b0001111) | (0b001 << FUNCT3_SHIFT),

    ECALL = (0b1110011) | (0b000 << FUNCT3_SHIFT) | (0b000000000000 << FUNCT7_SHIFT),
    EBREAK = (0b1110011) | (0b000 << FUNCT3_SHIFT) | (0b000000000001 << FUNCT7_SHIFT),

    CSRRW = (0b1110011) | (0b001 << FUNCT3_SHIFT),
    CSRRS = (0b1110011) | (0b010 << FUNCT3_SHIFT),
    CSRRC = (0b1110011) | (0b011 << FUNCT3_SHIFT),

    CSRRWI = (0b1110011) | (0b101 << FUNCT3_SHIFT),
    CSRRSI = (0b1110011) | (0b110 << FUNCT3_SHIFT),
    CSRRCI = (0b1110011) | (0b111 << FUNCT3_SHIFT),

    // RV64I Base Instruction Set (in addition to RV32I)
    LWU = (0b0000011) | (0b110 << FUNCT3_SHIFT),
    LD = (0b0000011) | (0b011 << FUNCT3_SHIFT),
    SD = (0b0100011) | (0b011 << FUNCT3_SHIFT),
    //SLLI64 = (0b0010011) | (0b001 << FUNCT3_SHIFT),
    //SRLI64 = (0b0010011) | (0b101 << FUNCT3_SHIFT),
    SRAI64 = (0b0010011) | (0b101 << FUNCT3_SHIFT) | (0b010000 << FUNCT7_SHIFT),

    // RV32M Standard Extension
    MUL = (0b0110011) | (0b000 << FUNCT3_SHIFT) | (0b0000001 << FUNCT7_SHIFT),
    MULH = (0b0110011) | (0b001 << FUNCT3_SHIFT) | (0b0000001 << FUNCT7_SHIFT),
    MULHSU = (0b0110011) | (0b010 << FUNCT3_SHIFT) | (0b0000001 << FUNCT7_SHIFT),
    MULHU = (0b0110011) | (0b011 << FUNCT3_SHIFT) | (0b0000001 << FUNCT7_SHIFT),

    DIV = (0b0110011) | (0b100 << FUNCT3_SHIFT) | (0b0000001 << FUNCT7_SHIFT),
    DIVU = (0b0110011) | (0b101 << FUNCT3_SHIFT) | (0b0000001 << FUNCT7_SHIFT),

    REM = (0b0110011) | (0b110 << FUNCT3_SHIFT) | (0b0000001 << FUNCT7_SHIFT),
    REMU = (0b0110011) | (0b111 << FUNCT3_SHIFT) | (0b0000001 << FUNCT7_SHIFT),

    // RV64M Standard Extension (in addition to RV32M)
    MULW = (0b0111011) | (0b000 << FUNCT3_SHIFT) | (0b0000001 << FUNCT7_SHIFT),
    DIVW = (0b0111011) | (0b100 << FUNCT3_SHIFT) | (0b0000001 << FUNCT7_SHIFT),
    DIVUW = (0b0111011) | (0b101 << FUNCT3_SHIFT) | (0b0000001 << FUNCT7_SHIFT),
    REMW = (0b0111011) | (0b110 << FUNCT3_SHIFT) | (0b0000001 << FUNCT7_SHIFT),
    REMUW = (0b0111011) | (0b111 << FUNCT3_SHIFT) | (0b0000001 << FUNCT7_SHIFT),
}
