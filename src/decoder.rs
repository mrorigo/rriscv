use crate::{
    cpu::{Core, Register},
    instruction_format::{self, CompressedInstructionFormat, InstructionFormat},
    opcodes::{CompressedOpcode, MajorOpcode},
    pipeline::RawInstruction,
};

// 2.2 Base Instruction Formats

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Btype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub rs1: Register,
    pub rs2: Register,
    pub funct3: u8,
    pub imm12: u16,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Jtype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub imm20: u32,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Itype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub rs1: Register,
    pub funct3: u8,
    pub imm12: u16,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Stype {
    pub opcode: MajorOpcode,
    pub rs1: Register,
    pub rs2: Register,
    pub imm12: u16,
    pub funct3: u8,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Utype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub imm20: u32,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Rtype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub rs1: Register,
    pub rs2: Register,
    pub funct3: u8,
    pub funct7: u8,
}

// Table 12.1: Compressed 16-bit RVC instruction formats.

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CRtype {
    pub opcode: CompressedOpcode,
    pub rs2: Register,
    pub rs1: Register,
    pub funct4: u8,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CItype {
    pub opcode: CompressedOpcode,
    pub rd: Register,
    pub imm: u16,
    pub funct3: u8,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CSStype {
    pub opcode: CompressedOpcode,
    pub uimm: u16,
    pub funct3: u8,
    pub rs2: Register,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CIWtype {
    pub opcode: CompressedOpcode,
    pub imm: u16,
    pub rd: Register,
    pub funct3: u8,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CLtype {
    pub opcode: CompressedOpcode,
    pub rd: Register,
    pub rs1: Register,
    pub imm: u16,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CStype {
    pub opcode: CompressedOpcode,
    pub rs1: Register,
    pub rs2: Register,
    pub funct: u8,
    pub funct6: u8,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CBtype {
    pub opcode: CompressedOpcode,
    pub rs1: Register,
    pub offset: u16,
    pub funct3: u8,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CJtype {
    pub opcode: CompressedOpcode,
    pub target: u16,
    pub funct3: u8,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DecodedInstruction {
    I(Itype),
    J(Jtype),
    B(Btype),
    S(Stype),
    U(Utype),
    R(Rtype),
    CR(CRtype),
    CI(CItype),
    CSS(CSStype),
    CIW(CIWtype),
    CL(CLtype),
    CS(CStype),
    CB(CBtype),
    CJ(CJtype),
}

pub trait InstructionDecoder {
    fn decode_instruction(&self, instruction: RawInstruction) -> DecodedInstruction;
}

trait ImmediateDecoder<T, T2> {
    fn decode_immediate(i: T) -> T2;
}

impl InstructionDecoder for Core<'_> {
    fn decode_instruction(&self, instruction: RawInstruction) -> DecodedInstruction {
        let word = instruction.word;
        match instruction.compressed {
            false => {
                let opcode = num::FromPrimitive::from_u8((word & 0x7f) as u8).unwrap();
                let funct3 = ((word >> 12) & 7) as u8;
                let funct7 = ((word >> 25) & 0x7f) as u8;

                match instruction_format::FORMAT_MAP[opcode as usize] {
                    InstructionFormat::R => DecodedInstruction::R(Rtype {
                        opcode: opcode,
                        rd: ((word >> 7) & 31) as Register,
                        rs1: ((word >> 15) & 31) as Register,
                        rs2: ((word >> 20) & 31) as Register,
                        funct3,
                        funct7,
                    }),
                    InstructionFormat::U => DecodedInstruction::U(Utype {
                        opcode: opcode,
                        rd: ((word >> 7) & 31) as Register,
                        imm20: (word >> 12),
                    }),
                    InstructionFormat::S => DecodedInstruction::S(Stype {
                        opcode: opcode,
                        rs1: ((word >> 15) & 31) as Register,
                        rs2: ((word >> 20) & 31) as Register,
                        imm12: Stype::decode_immediate(word),
                        funct3,
                    }),
                    InstructionFormat::B => DecodedInstruction::B(Btype {
                        opcode: opcode,
                        rd: ((word >> 7) & 31) as Register,
                        rs1: ((word >> 15) & 31) as Register,
                        rs2: ((word >> 20) & 31) as Register,
                        imm12: Btype::decode_immediate(word),
                        funct3,
                    }),
                    InstructionFormat::I => DecodedInstruction::I(Itype {
                        opcode: opcode,
                        rd: ((word >> 7) & 31) as Register,
                        rs1: ((word >> 15) & 31) as Register,
                        imm12: (word >> 20) as u16,
                        funct3,
                    }),
                    InstructionFormat::J => DecodedInstruction::J(Jtype {
                        opcode: opcode,
                        rd: ((word >> 7) & 31) as Register,
                        imm20: Jtype::decode_immediate(word),
                    }),
                    _ => panic!(
                        "invalid format {:?}",
                        crate::instruction_format::FORMAT_MAP[opcode as usize]
                    ),
                }
            }
            true => {
                let opcode = num::FromPrimitive::from_u8((word & 0x3) as u8).unwrap();
                let funct3 = ((word >> 13) & 0x7) as u8;
                let cop = ((opcode as usize) & 3) as usize | (funct3 << 2) as usize;
                match instruction_format::COMPRESSED_FORMAT_MAP[cop as usize] {
                    CompressedInstructionFormat::CSS => DecodedInstruction::CSS(CSStype {
                        opcode,
                        uimm: CSStype::decode_immediate(word as u16),
                        funct3: funct3,
                        rs2: (word >> 2) as u8 & 31,
                    }),
                    CompressedInstructionFormat::CB => DecodedInstruction::CB(CBtype {
                        opcode,
                        rs1: 8 + ((word >> 7) & 3) as u8,
                        offset: CBtype::decode_immediate(word as u16),
                        funct3,
                    }),
                    CompressedInstructionFormat::CS => DecodedInstruction::CS(CStype {
                        opcode,
                        rs1: ((word >> 7) & 3) as u8 + 8,
                        rs2: ((word >> 2) & 7) as u8 + 8,
                        funct: (word as u8 >> 5) & 3,
                        funct6: (word >> 10) as u8,
                    }),
                    CompressedInstructionFormat::CI => DecodedInstruction::CI(CItype {
                        opcode,
                        rd: ((word >> 7) & 31) as u8,
                        imm: CItype::decode_immediate(word as u16),
                        funct3,
                    }),
                    CompressedInstructionFormat::CR => DecodedInstruction::CR(CRtype {
                        opcode,
                        rs2: ((word >> 2) & 31) as Register,
                        rs1: (word >> 7 & 31) as Register,
                        funct4: (word >> 12) as u8 & 0b1111,
                    }),
                    CompressedInstructionFormat::CIW => DecodedInstruction::CIW(CIWtype {
                        opcode,
                        imm: (word >> 5) as u16,
                        rd: ((word >> 2) & 7) as Register + 8,
                        funct3,
                    }),
                    CompressedInstructionFormat::CL => DecodedInstruction::CL(CLtype {
                        opcode,
                        rd: ((word >> 2) & 7) as Register + 8,
                        rs1: ((word >> 7) & 31) as Register + 8,
                        imm: CLtype::decode_immediate(word as u16),
                    }),
                    CompressedInstructionFormat::CJ => todo!(),
                    CompressedInstructionFormat::Unknown => panic!(),
                    // _ => panic!(
                    //     "invalid format for {:?} {:?}",
                    //     opcode,
                    //     crate::instruction_format::COMPRESSED_FORMAT_MAP[opcode as usize]
                    // ),
                }
            }
        }
    }

    // fn decode_instruction(&self, instruction: (bool, u32)) -> DecodedInstruction {
    //     let i = instruction.1;
    //     let op = (i & 0x7f) as u8;

    //     let mut rs1 = ((i >> 15) & 31) as u8;
    //     let mut rs2 = ((i >> 20) & 31) as u8;
    //     let funct7 = ((i >> 25) & 7) as u8;
    //     let mut funct3 = ((i >> 12) & 7) as u8;
    //     let shamt = ((i >> 20) & 31) as u8;
    //     let mut imm5 = ((i >> 7) & 31) as u8;
    //     let mut mem_offset = None;

    //     let mut rd = ((i >> 7) & 31) as u8;
    //     let mut imm12 = 0;
    //     let mut imm20: u32 = 0;

    //     //        let memory_access_width = MemoryAccessWidth::WORD;

    //     // Determine opcode type, decode immediates, and set up `op` for lookup
    //     // in OpCodes by combining correct bits from raw opcode.
    //     let instruction_format = match instruction.0 {
    //         false => {
    //             println!(
    //                 "decode: decoding instruction: {:#x?} ({:#x?})",
    //                 (i & 0x7f),
    //                 i
    //             );
    //             crate::optypes::FORMAT_MAP[(i & 0x7f) as usize]
    //         }
    //         true => {
    //             funct3 = ((i >> 13) & 0x7) as u8;
    //             let ci = (i & 3) | (funct3 << 2) as u32;
    //             println!(
    //                 "decode: decoding compressed instruction: {:#x?} ({:#x?})",
    //                 (i & 0x7f),
    //                 ci
    //             );
    //             crate::optypes::COMPRESSED_FORMAT_MAP[ci as usize]
    //         }
    //     };

    //     println!(
    //         "decode: instruction_format={:?} funct3={:?}  funct7={:?}",
    //         instruction_format, funct3, funct7
    //     );

    //     // @TODO: This has side-effects (updating local mutables), not so nice
    //     let opcode: Option<OpCodes> = match instruction_format {
    //         InstructionFormat::CR => {
    //             rd = (i >> 7 & 31) as u8;
    //             rs1 = rd;
    //             rs2 = ((i >> 2) & 31) as u8;
    //             Some(OpCodes::ADD)
    //         }

    //         InstructionFormat::CI => {
    //             let nzimm1612 = (i >> 2) & 31;
    //             let nzimm17 = (i >> 12) & 1;
    //             imm20 = nzimm1612 | (nzimm17 << 5); // @TODO: Sign extend imm20
    //             rd = (i >> 7 & 31) as u8;

    //             //imm12 = (((i >> 2) & 31) | ((i >> 12) & 1) << 5) as u16;
    //             match funct3 {
    //                 0b000 => {
    //                     rs1 = rd;
    //                     imm5 = ((i >> 2) & 0x31) as u8;
    //                     Some(OpCodes::ADDI)
    //                 }
    //                 // C.LI expands into addi rd, x0, imm[5:0].
    //                 0b010 => {
    //                     imm12 = nzimm1612 as u16;
    //                     rs1 = 0;
    //                     Some(OpCodes::ADDI)
    //                 }
    //                 0b011 => {
    //                     // C.ADDI16SP shares the opcode with C.LUI, but has a destination field of x2
    //                     match rd {
    //                         2 => {
    //                             // @TODO: ADDI16SP
    //                             todo!("decode nzimm46875: expands into addi x2, x2, nzimm[9:4]");
    //                             Some(OpCodes::ADDI)
    //                         }
    //                         //0 => 0 as u32,
    //                         _ => Some(OpCodes::LUI),
    //                     }
    //                 }
    //                 0b111 => {
    //                     // C.ADDI expands into addi rd, rd, nzimm[5:0]
    //                     rs1 = rd;
    //                     imm12 = (((i >> 2) & 0b11111) | (((i >> 12) & 1) << 5)) as u16;
    //                     Some(OpCodes::ADDI)
    //                 }
    //                 _ => todo!(),
    //             }
    //         }

    //         InstructionFormat::CSS => match funct3 {
    //             // C.SDSP
    //             0b111 => {
    //                 rd = 2;
    //                 rs2 = ((i >> 2) & 31) as u8;

    //                 let offset = ((i >> 7) & 0x38) | // offset[5:3] <= [12:10]
    //                     ((i >> 1) & 0x1c0); // offset[8:6] <= [9:7]
    //                 let imm11_5 = (offset >> 5) & 0x3f;
    //                 let imm4_0 = offset & 0x1f;
    //                 imm20 = (imm11_5 << 5) | (imm4_0);
    //                 mem_offset = Some(imm20 as u64);
    //                 Some(OpCodes::SD)
    //             }
    //             _ => todo!(),
    //         },

    //         InstructionFormat::CIW => match funct3 {
    //             0b000 => {
    //                 // C.ADDI4SPN
    //                 rs1 = 2;
    //                 rd = ((i >> 2) & 0x7) as u8 + 8;
    //                 imm12 = (((i >> 7) & 0x30) | // nzuimm[5:4] <= [12:11]
    // 					((i >> 1) & 0x3c0) | // nzuimm{9:6] <= [10:7]
    // 					((i >> 4) & 0x4) | // nzuimm[2] <= [6]
    // 					((i >> 2) & 0x8)) as u16; // nzuimm[3] <= [5]

    //                 Some(OpCodes::ADDI)
    //             }
    //             _ => todo!(),
    //         },

    //         InstructionFormat::CL => todo!(),

    //         InstructionFormat::CS => match funct3 {
    //             // "MISC-ALU", Table 12.3: RVC opcode map
    //             0b100 => {
    //                 rd = ((i >> 7) & 7) as u8 + 8;
    //                 rs1 = rd;
    //                 rs2 = ((i >> 2) & 7) as u8 + 8;
    //                 Some(OpCodes::AND)
    //             }
    //             _ => todo!(),
    //         },

    //         InstructionFormat::CB => todo!(),

    //         InstructionFormat::CJ => todo!(),

    //         InstructionFormat::R => self.make_op(op, funct3, funct7),
    //         InstructionFormat::I => {
    //             imm12 = ((i >> 20) & ((1 << 12) - 1)) as u16;
    //             // ADDIW is an exception, no funct7 if funct3=0 and handles as such in unwrap_or
    //             match funct3 {
    //                 0b000 => self.make_op(op, 0, 0),
    //                 _ => self.make_op(op, funct3, funct7),
    //             }
    //         }

    //         InstructionFormat::S => {
    //             imm12 = (((i >> 7) & 0b11111) | ((i >> 20) & 0xffffe0)) as u16;
    //             imm20 = imm12 as u32 | imm5 as u32;
    //             mem_offset = Some(imm20 as u64);
    //             self.make_op(op, funct3, 0)
    //         }

    //         InstructionFormat::B => {
    //             imm12 = ((i >> 31) & 1) as u16;
    //             let imm105 = ((i >> 25) & 0b111111) as u16;
    //             let imm41 = ((i >> 8) & 0xf) as u16;
    //             let imm11 = ((i >> 7) & 1) as u16;
    //             imm12 = (imm12 << 12) | (imm105 << 5) | (imm41 << 1) | (imm11 << 11);
    //             self.make_op(op, funct3, 0)

    //             //imm12 is only 12 bits in struct, but we have 13 for branches, so to preserve
    //             //top bit, we need to shift down once, and then account for that when
    //             // calculating jumpTarget
    //             //                imm12 = bimm >> 1;
    //         }

    //         InstructionFormat::U => {
    //             imm20 = ((i >> 12) & 0xfffff) as u32;
    //             self.make_op(op, 0, 0)
    //         }

    //         InstructionFormat::J => {
    //             imm20 = ((i >> 31) & 0b1) as u32;
    //             let imm101 = ((i >> 21) & 0b1111111111) as u32;
    //             let imm11 = ((i >> 20) & 0b1) as u32;
    //             let imm1912 = ((i >> 12) & 0b11111111) as u32;

    //             let imm = (imm20 << 20) | (imm101 << 1) | (imm11 << 11) | (imm1912 << 12);
    //             imm20 = ((imm) << 11) >> 12;
    //             self.make_op(op, 0, 0)
    //         }

    //         InstructionFormat::C => {
    //             imm12 = (i >> 20) as u16;
    //             self.make_op(op, funct3, 0)
    //         }

    //         InstructionFormat::Unknown => panic!(),
    //     };
    //     // println!("opcode: {:?}", opcode);
    //     //        let opcode: OpCodes = num::FromPrimitive::from_u32(op).unwrap();
    //     //        println!("op: {:#x?}  opcode: {:?}", op, opcode);

    //     DecodedInstruction {
    //         instruction_format,
    //         opcode,
    //         rs1,
    //         rs2,
    //         rd,
    //         funct7,
    //         funct3,
    //         shamt,
    //         imm5,
    //         imm12,
    //         imm20,
    //         mem_offset,
    //     }
    // }
}

impl ImmediateDecoder<u32, u16> for Btype {
    fn decode_immediate(i: u32) -> u16 {
        let imm12 = ((i >> 31) & 1) as u16;
        let imm105 = ((i >> 25) & 0b111111) as u16;
        let imm41 = ((i >> 8) & 0xf) as u16;
        let imm11 = ((i >> 7) & 1) as u16;
        (imm12 << 12) | (imm105 << 5) | (imm41 << 1) | (imm11 << 11)
    }
}
impl ImmediateDecoder<u32, u16> for Stype {
    fn decode_immediate(i: u32) -> u16 {
        let imm12 = (((i >> 7) & 0b11111) | ((i >> 20) & 0xffffe0)) as u16;
        let imm5 = ((i >> 7) & 31) as u16;
        imm12 | imm5 as u16
    }
}
impl ImmediateDecoder<u32, u32> for Jtype {
    fn decode_immediate(i: u32) -> u32 {
        let imm20 = ((i >> 31) & 0b1) as u32;
        let imm101 = ((i >> 21) & 0b1111111111) as u32;
        let imm11 = ((i >> 20) & 0b1) as u32;
        let imm1912 = ((i >> 12) & 0b11111111) as u32;

        let imm = (imm20 << 20) | (imm101 << 1) | (imm11 << 11) | (imm1912 << 12);
        ((imm) << 11) >> 12
    }
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
impl ImmediateDecoder<u16, u16> for CItype {
    fn decode_immediate(i: u16) -> u16 {
        let nzimm1612 = (i >> 2) & 31;
        let nzimm17 = (i >> 12) & 1;
        nzimm1612 | (nzimm17 << 5)
    }
}

impl ImmediateDecoder<u16, u16> for CLtype {
    fn decode_immediate(i: u16) -> u16 {
        ((i >> 7) & 0x38) | // offset[5:3] <= [12:10]
        ((i >> 4) & 0x4) | // offset[2] <= [6]
        ((i << 1) & 0x40) // offset[6] <= [5]
    }
}
impl ImmediateDecoder<u16, u16> for CBtype {
    fn decode_immediate(i: u16) -> u16 {
        todo!()
    }
}