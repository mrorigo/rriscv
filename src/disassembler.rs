use crate::cpu::Xlen;
use crate::instructions::InstructionSelector;
use crate::{
    instructions::decoder::{DecodedInstruction, InstructionDecoder},
    pipeline::RawInstruction,
};

pub struct Disassembler {}

impl Disassembler {
    pub fn disassemble(word: u32, xlen: Xlen) -> String {
        let mut instruction_decoder = InstructionDecoder::create();
        let raw = RawInstruction::from_word(word, 0);
        let decoded = instruction_decoder.decode_instruction(raw);
        let s = match decoded {
            DecodedInstruction::I(inst) => inst.select(xlen).to_string(),
            DecodedInstruction::U(inst) => inst.select(xlen).to_string(),
            DecodedInstruction::CI(param) => param.select(xlen).to_string(),
            DecodedInstruction::J(param) => param.select(xlen).to_string(),
            DecodedInstruction::CR(param) => param.select(xlen).to_string(),
            DecodedInstruction::B(param) => param.select(xlen).to_string(),
            DecodedInstruction::S(param) => param.select(xlen).to_string(),
            DecodedInstruction::R(param) => param.select(xlen).to_string(),
            DecodedInstruction::CSS(param) => param.select(xlen).to_string(),
            DecodedInstruction::CIW(param) => param.select(xlen).to_string(),
            DecodedInstruction::CL(param) => param.select(xlen).to_string(),
            DecodedInstruction::CS(param) => param.select(xlen).to_string(),
            DecodedInstruction::CB(param) => param.select(xlen).to_string(),
            DecodedInstruction::CJ(param) => param.select(xlen).to_string(),
        };
        let sp = s.split(" ").collect::<Vec<&str>>();
        if sp.len() > 0 && sp[0].len() < 4 {
            s.replace(" ", "\t\t")
        } else {
            s.replace(" ", "\t")
        }
    }
}
