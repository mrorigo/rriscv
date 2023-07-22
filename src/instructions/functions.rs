#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum Funct3 {
    B000 = 0b000,
    B001 = 0b001,
    B010 = 0b010,
    B011 = 0b011,
    B100 = 0b100,
    B101 = 0b101,
    B110 = 0b110,
    B111 = 0b111,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum C0_Funct3 {
    C_ADDI4SPN = 0b000,
    C_LQ = 0b001, // also C_FLD on RV32
    C_LW = 0b010,
    C_LD = 0b011, // also C_FLW on RV32
    C_SQ = 0b101, // Also C_FSD on RV32
    C_SW = 0b110,
    C_SD = 0b111,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum OpImm_Funct3 {
    ADDI = 0b000,
    SLLI = 0b001,
    SLTI = 0b010,
    SLTIU = 0b011,
    XORI = 0b100,
    ORI = 0b110,
    ANDI = 0b111,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum OpImm32_Funct3 {
    ADDIW = 0b000,
    SLLIW = 0b001,
    SRLIW = 0b101,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum Load_Funct3 {
    LB = 0x000,
    LH = 0b001,
    LW = 0b010,
    LBU = 0b100,
    LHU = 0b101,
    LD = 0b011,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, FromPrimitive)]
pub enum CSR_Funct3 {
    CSRRW = 0b001,
    CSRRS = 0b010,
    CSRRC = 0b011,
    CSRRWI = 0b101,
    CSRRSI = 0b110,
    CSRRCI = 0b111,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, FromPrimitive)]
pub enum RV32M_Funct3 {
    MUL = 0b000,
    MULH = 0b001,
    MULHSU = 0b010,
    MULSHU = 0b011,
    DIV = 0b100,
    DIVU = 0b101,
    REM = 0b110,
    REMU = 0b111,
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, FromPrimitive)]
pub enum C1_Funct3 {
    C_ADDI = 0b000,
    C_ADDIW = 0b001,
    C_LI = 0b010,
    C_LUI = 0b011, // ADDI16SP shares the opcode
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, FromPrimitive)]
pub enum C2_Funct3 {
    C_SLLI = 0b000, // C.SLLI64 shares funct3
    C_LDSP = 0b011, // C.FLWSP
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, FromPrimitive)]
pub enum C1_Funct6 {
    C_AND = 0b100_011,
    // C_OR = 0b100,
    // C_XOR = 0b100,
    // C_SUB = 0b100,
    // C_ADDW = 0b100,
    C_SUBW = 0b100_111,
}
