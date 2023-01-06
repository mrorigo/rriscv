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
#[derive(Debug, Copy, Clone, FromPrimitive)]
pub enum OpCodes {
    NONE = 0,
    LUI = (0b0110111) | (0b000 << 7),

    AUIPC = (0b0010111) | (0b000 << 7),
    JAL = (0b1101111) | (0b000 << 7),
    JALR = (0b1100111) | (0b000 << 7),

    BEQ = (0b1100011) | (0b000 << 7),
    BNE = (0b1100011) | (0b001 << 7),
    BLT = (0b1100011) | (0b100 << 7),
    BGE = (0b1100011) | (0b101 << 7),
    BLTU = (0b1100011) | (0b110 << 7),
    BGEU = (0b1100011) | (0b111 << 7),

    LB = (0b0000011) | (0b000 << 7),
    LH = (0b0000011) | (0b001 << 7),
    LW = (0b0000011) | (0b010 << 7),
    LBU = (0b0000011) | (0b100 << 7),
    LHU = (0b0000011) | (0b101 << 7),

    SB = (0b0100011) | (0b000 << 7),
    SH = (0b0100011) | (0b001 << 7),
    SW = (0b0100011) | (0b010 << 7),

    ADDI = (0b0010011) | (0b000 << 7),
    SLTI = (0b0010011) | (0b010 << 7),
    SLTIU = (0b0010011) | (0b011 << 7),
    XORI = (0b0010011) | (0b100 << 7),
    ORI = (0b0010011) | (0b110 << 7),
    ANDI = (0b0010011) | (0b111 << 7),

    SLLI = (0b0010011) | (0b001 << 7),
    SRLI = (0b0010011) | (0b101 << 7),
    SRAI = (0b0010011) | (0b101 << 7) | (0b0100000 << FUNCT7_SHIFT),

    ADD = (0b0110011) | (0b000 << 7),
    SUB = (0b0110011) | (0b000 << 7) | (0b0100000 << FUNCT7_SHIFT),
    SLL = (0b0110011) | (0b001 << 7) | (0b0000000 << FUNCT7_SHIFT),
    SLT = (0b0110011) | (0b010 << 7),
    SLTU = (0b0110011) | (0b011 << 7),

    XOR = (0b0110011) | (0b100 << 7),
    SRL = (0b0110011) | (0b101 << 7) | (0b0000000 << FUNCT7_SHIFT),
    SRA = (0b0110011) | (0b101 << 7) | (0b0100000 << FUNCT7_SHIFT),
    OR = (0b0110011) | (0b110 << 7),
    AND = (0b0110011) | (0b111 << 7),

    FENCE = (0b0001111) | (0b000 << 7),

    ECALL = (0b1110011) | (0b000 << 7) | (0b000000000000 << FUNCT7_SHIFT),
    EBREAK = (0b1110011) | (0b000 << 7) | (0b000000000001 << FUNCT7_SHIFT),

    CSRRW = (0b1110011) | (0b001 << 7),
    CSRRS = (0b1110011) | (0b010 << 7),
    CSRRC = (0b1110011) | (0b011 << 7),

    CSRRWI = (0b1110011) | (0b101 << 7),
    CSRRSI = (0b1110011) | (0b110 << 7),
    CSRRCI = (0b1110011) | (0b111 << 7),
}
