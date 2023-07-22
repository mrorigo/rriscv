use crate::instructions::{CompressedFormat, InstructionFormat};

pub const COMPRESSED_FORMAT_MAP: [CompressedFormat; 32] = [
    CompressedFormat::CIW, // 0
    CompressedFormat::CI,
    CompressedFormat::CI,
    CompressedFormat::Unknown,
    CompressedFormat::Unknown,
    CompressedFormat::CI,
    CompressedFormat::Unknown,
    CompressedFormat::Unknown,
    CompressedFormat::CL,      // C.LW
    CompressedFormat::CI,      // C.LI
    CompressedFormat::Unknown, // 10
    CompressedFormat::Unknown,
    CompressedFormat::Unknown,
    CompressedFormat::CI, // C.LUI , ADDI16SP
    CompressedFormat::CI, // C.LDSP etc
    CompressedFormat::Unknown,
    CompressedFormat::Unknown,
    CompressedFormat::CS,
    CompressedFormat::CR,
    CompressedFormat::Unknown,
    CompressedFormat::Unknown, // 20
    CompressedFormat::CJ,
    CompressedFormat::Unknown,
    CompressedFormat::Unknown,
    CompressedFormat::Unknown,
    CompressedFormat::CB,
    CompressedFormat::Unknown,
    CompressedFormat::Unknown,
    CompressedFormat::CS,
    CompressedFormat::Unknown,
    CompressedFormat::CSS,     // C.SDSP,
    CompressedFormat::Unknown, // 31
];
// pub enum CsrInstructionType {
//     CSRRW = 0b001,
//     CSRRS = 0b010,
//     CSRRC = 0b011,
//     CSRRWI = 0b101,
//     CSRRSI = 0b110,
//     CSRRCI = 0b111,
// }
pub const FORMAT_MAP: [InstructionFormat; 128] = [
    InstructionFormat::Unknown, /*0000000 */
    InstructionFormat::Unknown, /*0000001 */
    InstructionFormat::Unknown, /*0000010 */
    InstructionFormat::I,       /*0000011 = 0x03 = LB/LH/LW */
    InstructionFormat::Unknown, /*0000100 */
    InstructionFormat::Unknown, /*0000101 */
    InstructionFormat::Unknown, /*0000110 */
    InstructionFormat::Unknown, /*0000111 */
    InstructionFormat::Unknown, /*0001000 */
    InstructionFormat::Unknown, /*0001001 */
    InstructionFormat::Unknown, /*0001010 */
    InstructionFormat::Unknown, /*0001011 */
    InstructionFormat::Unknown, /*0001100 */
    InstructionFormat::Unknown, /*0001101 */
    InstructionFormat::Unknown, /*0001110 */
    InstructionFormat::I,       /*0001111 = 0xf = FENCE*/
    InstructionFormat::Unknown, /*0010000 */
    InstructionFormat::Unknown, /*0010001 */
    InstructionFormat::Unknown, /*0010010 */
    InstructionFormat::I,       /*0010011 = 0x13 = ADDI/SLTI/SLTIU/XORI/ORI/ANDI/SLLI/SRLI/SRAI */
    InstructionFormat::Unknown, /*0010100 */
    InstructionFormat::Unknown, /*0010101 */
    InstructionFormat::Unknown, /*0010110 */
    InstructionFormat::U,       /*0010111 = 0x17 = AUIPC */
    InstructionFormat::Unknown, /*0011000 */
    InstructionFormat::Unknown, /*0011001 */
    InstructionFormat::Unknown, /*0011010 */
    InstructionFormat::I,       /*0011011 = 0x1b = ADDIW */
    InstructionFormat::Unknown, /*0011100 */
    InstructionFormat::Unknown, /*0011101 */
    InstructionFormat::Unknown, /*0011110 */
    InstructionFormat::Unknown, /*0011111 = 0x20*/
    InstructionFormat::Unknown, /*0100000 */
    InstructionFormat::Unknown, /*0100001 */
    InstructionFormat::Unknown, /*0100010 */
    InstructionFormat::S,       /*0100011 = SB/SH/SW */
    InstructionFormat::Unknown, /*0100100 */
    InstructionFormat::Unknown, /*0100101 */
    InstructionFormat::Unknown, /*0100110 */
    InstructionFormat::Unknown, /*0100111 */
    InstructionFormat::Unknown, /*0101000 */
    InstructionFormat::Unknown, /*0101001 */
    InstructionFormat::Unknown, /*0101010 */
    InstructionFormat::Unknown, /*0101011 */
    InstructionFormat::Unknown, /*0101100 */
    InstructionFormat::Unknown, /*0101101 */
    InstructionFormat::Unknown, /*0101110 */
    InstructionFormat::Unknown, /*0101111 */
    InstructionFormat::Unknown, /*0110000 */
    InstructionFormat::Unknown, /*0110001 */
    InstructionFormat::Unknown, /*0110010 */
    InstructionFormat::R,       /*0110011 XOR/XRL/SRA/OR/AND */
    InstructionFormat::Unknown, /*0110100 */
    InstructionFormat::Unknown, /*0110101 */
    InstructionFormat::Unknown, /*0110110 */
    InstructionFormat::U,       /*0110111 = LUI */
    InstructionFormat::Unknown, /*0111000 */
    InstructionFormat::Unknown, /*0111001 */
    InstructionFormat::Unknown, /*0111010 */
    InstructionFormat::Unknown, /*0111011 */
    InstructionFormat::Unknown, /*0111100 */
    InstructionFormat::Unknown, /*0111101 */
    InstructionFormat::Unknown, /*0111110 */
    InstructionFormat::Unknown, /*0111111 */
    InstructionFormat::Unknown, /*1000000 */
    InstructionFormat::Unknown, /*1000001 */
    InstructionFormat::Unknown, /*1000010 */
    InstructionFormat::Unknown, /*1000011 */
    InstructionFormat::Unknown, /*1000100 */
    InstructionFormat::Unknown, /*1000101 */
    InstructionFormat::Unknown, /*1000110 */
    InstructionFormat::Unknown, /*1000111 */
    InstructionFormat::Unknown, /*1001000 */
    InstructionFormat::Unknown, /*1001001 */
    InstructionFormat::Unknown, /*1001010 */
    InstructionFormat::Unknown, /*1001011 */
    InstructionFormat::Unknown, /*1001100 */
    InstructionFormat::Unknown, /*1001101 */
    InstructionFormat::Unknown, /*1001110 */
    InstructionFormat::Unknown, /*1001111 */
    InstructionFormat::Unknown, /*1010000 */
    InstructionFormat::Unknown, /*1010001 */
    InstructionFormat::Unknown, /*1010010 */
    InstructionFormat::Unknown, /*1010011 */
    InstructionFormat::Unknown, /*1010100 */
    InstructionFormat::Unknown, /*1010101 */
    InstructionFormat::Unknown, /*1010110 */
    InstructionFormat::Unknown, /*1010111 */
    InstructionFormat::Unknown, /*1011000 */
    InstructionFormat::Unknown, /*1011001 */
    InstructionFormat::Unknown, /*1011010 */
    InstructionFormat::Unknown, /*1011011 */
    InstructionFormat::Unknown, /*1011100 */
    InstructionFormat::Unknown, /*1011101 */
    InstructionFormat::Unknown, /*1011110 */
    InstructionFormat::Unknown, /*1011111 */
    InstructionFormat::Unknown, /*1100000 */
    InstructionFormat::Unknown, /*1100001 */
    InstructionFormat::Unknown, /*1100010 */
    InstructionFormat::B,       /*1100011 = BEQ/BNE/BLT/BGE/BLTU/BGEU */
    InstructionFormat::Unknown, /*1100100 */
    InstructionFormat::Unknown, /*1100101 */
    InstructionFormat::Unknown, /*1100110 */
    InstructionFormat::I,       /*1100111 = JALR */
    InstructionFormat::Unknown, /*1101000 */
    InstructionFormat::Unknown, /*1101001 */
    InstructionFormat::Unknown, /*1101010 */
    InstructionFormat::Unknown, /*1101011 */
    InstructionFormat::Unknown, /*1101100 */
    InstructionFormat::Unknown, /*1101101 */
    InstructionFormat::Unknown, /*1101110 */
    InstructionFormat::J,       /*1101111 = JAL */
    InstructionFormat::Unknown, /*1110000 */
    InstructionFormat::Unknown, /*1110001 */
    InstructionFormat::Unknown, /*1110010 */
    InstructionFormat::I,       /*1110011 = CSR* */
    InstructionFormat::Unknown, /*1110100 */
    InstructionFormat::Unknown, /*1110101 */
    InstructionFormat::Unknown, /*1110110 */
    InstructionFormat::Unknown, /*1110111 */
    InstructionFormat::Unknown, /*1111000 */
    InstructionFormat::Unknown, /*1111001 */
    InstructionFormat::Unknown, /*1111010 */
    InstructionFormat::Unknown, /*1111011 */
    InstructionFormat::Unknown, /*1111100 */
    InstructionFormat::Unknown, /*1111101 */
    InstructionFormat::Unknown, /*1111110 */
    InstructionFormat::Unknown, /*1111111 */
];
