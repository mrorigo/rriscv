#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum OpType {
    Unknown = 0,
    R,
    I,
    S,
    B,
    U,
    J,
    C, // System (CSR*)

    // Compressed op types
    CR,
    CI,
    CSS,
    CIW,
    CL,
    CS,
    CB,
    CJ,
}

pub const COMPRESSED_OPTYPES: [OpType; 32] = [
    OpType::Unknown, // 0
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown, // 10
    OpType::Unknown,
    OpType::Unknown,
    OpType::CI, // C.LUI/ADDI16SP
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown, // 20
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown,
    OpType::Unknown, // 31
];
// pub enum CsrInstructionType {
//     CSRRW = 0b001,
//     CSRRS = 0b010,
//     CSRRC = 0b011,
//     CSRRWI = 0b101,
//     CSRRSI = 0b110,
//     CSRRCI = 0b111,
// }
pub const OPTYPES: [OpType; 128] = [
    OpType::Unknown, /*0000000 */
    OpType::Unknown, /*0000001 */
    OpType::Unknown, /*0000010 */
    OpType::I,       /*0000011 = LB/LH/LW */
    OpType::Unknown, /*0000100 */
    OpType::Unknown, /*0000101 */
    OpType::Unknown, /*0000110 */
    OpType::Unknown, /*0000111 */
    OpType::Unknown, /*0001000 */
    OpType::Unknown, /*0001001 */
    OpType::Unknown, /*0001010 */
    OpType::Unknown, /*0001011 */
    OpType::Unknown, /*0001100 */
    OpType::Unknown, /*0001101 */
    OpType::Unknown, /*0001110 */
    OpType::Unknown, /*0001111 */
    OpType::Unknown, /*0010000 */
    OpType::Unknown, /*0010001 */
    OpType::Unknown, /*0010010 */
    OpType::I,       /*0010011 = ADDI/SLTI/SLTIU/XORI/ORI/ANDI/SLLI/SRLI/SRAI */
    OpType::Unknown, /*0010100 */
    OpType::Unknown, /*0010101 */
    OpType::Unknown, /*0010110 */
    OpType::U,       /*0010111 = AUIPC */
    OpType::Unknown, /*0011000 */
    OpType::Unknown, /*0011001 */
    OpType::Unknown, /*0011010 */
    OpType::Unknown, /*0011011 */
    OpType::Unknown, /*0011100 */
    OpType::Unknown, /*0011101 */
    OpType::Unknown, /*0011110 */
    OpType::Unknown, // 32 /*0011111 */
    OpType::Unknown, /*0100000 */
    OpType::Unknown, /*0100001 */
    OpType::Unknown, /*0100010 */
    OpType::S,       /*0100011 = SB/SH/SW */
    OpType::Unknown, /*0100100 */
    OpType::Unknown, /*0100101 */
    OpType::Unknown, /*0100110 */
    OpType::Unknown, /*0100111 */
    OpType::Unknown, /*0101000 */
    OpType::Unknown, /*0101001 */
    OpType::Unknown, /*0101010 */
    OpType::Unknown, /*0101011 */
    OpType::Unknown, /*0101100 */
    OpType::Unknown, /*0101101 */
    OpType::Unknown, /*0101110 */
    OpType::Unknown, /*0101111 */
    OpType::Unknown, /*0110000 */
    OpType::Unknown, /*0110001 */
    OpType::Unknown, /*0110010 */
    OpType::R,       /*0110011 XOR/XRL/SRA/OR/AND */
    OpType::Unknown, /*0110100 */
    OpType::Unknown, /*0110101 */
    OpType::Unknown, /*0110110 */
    OpType::U,       /*0110111 = LUI */
    OpType::Unknown, /*0111000 */
    OpType::Unknown, /*0111001 */
    OpType::Unknown, /*0111010 */
    OpType::Unknown, /*0111011 */
    OpType::Unknown, /*0111100 */
    OpType::Unknown, /*0111101 */
    OpType::Unknown, /*0111110 */
    OpType::Unknown, /*0111111 */
    OpType::Unknown, /*1000000 */
    OpType::Unknown, /*1000001 */
    OpType::Unknown, /*1000010 */
    OpType::Unknown, /*1000011 */
    OpType::Unknown, /*1000100 */
    OpType::Unknown, /*1000101 */
    OpType::Unknown, /*1000110 */
    OpType::Unknown, /*1000111 */
    OpType::Unknown, /*1001000 */
    OpType::Unknown, /*1001001 */
    OpType::Unknown, /*1001010 */
    OpType::Unknown, /*1001011 */
    OpType::Unknown, /*1001100 */
    OpType::Unknown, /*1001101 */
    OpType::Unknown, /*1001110 */
    OpType::Unknown, /*1001111 */
    OpType::Unknown, /*1010000 */
    OpType::Unknown, /*1010001 */
    OpType::Unknown, /*1010010 */
    OpType::Unknown, /*1010011 */
    OpType::Unknown, /*1010100 */
    OpType::Unknown, /*1010101 */
    OpType::Unknown, /*1010110 */
    OpType::Unknown, /*1010111 */
    OpType::Unknown, /*1011000 */
    OpType::Unknown, /*1011001 */
    OpType::Unknown, /*1011010 */
    OpType::Unknown, /*1011011 */
    OpType::Unknown, /*1011100 */
    OpType::Unknown, /*1011101 */
    OpType::Unknown, /*1011110 */
    OpType::Unknown, /*1011111 */
    OpType::Unknown, /*1100000 */
    OpType::Unknown, /*1100001 */
    OpType::Unknown, /*1100010 */
    OpType::B,       /*1100011 = BEQ/BNE/BLT/BGE/BLTU/BGEU */
    OpType::Unknown, /*1100100 */
    OpType::Unknown, /*1100101 */
    OpType::Unknown, /*1100110 */
    OpType::I,       /*1100111 = JALR */
    OpType::Unknown, /*1101000 */
    OpType::Unknown, /*1101001 */
    OpType::Unknown, /*1101010 */
    OpType::Unknown, /*1101011 */
    OpType::Unknown, /*1101100 */
    OpType::Unknown, /*1101101 */
    OpType::Unknown, /*1101110 */
    OpType::J,       /*1101111 = JAL */
    OpType::Unknown, /*1110000 */
    OpType::Unknown, /*1110001 */
    OpType::Unknown, /*1110010 */
    OpType::C,       /*1110011 = CSR* */
    OpType::Unknown, /*1110100 */
    OpType::Unknown, /*1110101 */
    OpType::Unknown, /*1110110 */
    OpType::Unknown, /*1110111 */
    OpType::Unknown, /*1111000 */
    OpType::Unknown, /*1111001 */
    OpType::Unknown, /*1111010 */
    OpType::Unknown, /*1111011 */
    OpType::Unknown, /*1111100 */
    OpType::Unknown, /*1111101 */
    OpType::Unknown, /*1111110 */
    OpType::Unknown, /*1111111 */
];
