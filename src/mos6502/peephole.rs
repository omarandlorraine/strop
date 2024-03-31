use crate::Peephole;
use crate::Candidate;
use crate::SearchCull;
use crate::mos6502::Nmos6502Instruction;
use crate::mos6502::Cmos6502Instruction;
use crate::mos6502::instruction_set::Mos6502Compatibility;

fn check<I: Mos6502Compatibility>(cand: &Candidate<I>, a: u8, b: &[u8]) -> Option<(usize, SearchCull<I>)> {
    let vec = &cand.instructions;
    for i in 0..vec.len() - 1 {
        if vec[i].encode()[0] == a {
            if b.contains(&vec[i + 1].encode()[0]) {
                return Some((i, vec[i].next_opcode()))
            }
        }
    }
    None
}

fn commutative<I: Mos6502Compatibility>(candidate: &Candidate<I>) -> Option<(usize, SearchCull<I>)> {
    // If any pair of instructions in the program could be equivalently executed the other way
    // 'round, then if that program has already been proposed, skip this program
    
    let flags: Vec<u8> = vec![
        0x18, // clc 
        0x38, // sec 
        0x58, // cli 
        0x78, // sei 
        0x8a, // txa 
        0x88, // dey 
        0x98, // tya 
        0xa8, // tay 
        0xaa, // tax 
        0xb8, // clv 
        0xc8, // iny 
        0xca, // dex 
        0xd8, // cld 
        0xe8, // inx 
        0xea, // nop
        0xf8, // sed 
    ];

    for (index, &a) in flags.iter().enumerate() {
        let rest = &flags[index..];
        if let Some(res) = check(candidate, a, &rest) {
            return Some(res);
        }
    }
    None
}

fn dead_code<I: Mos6502Compatibility>(candidate: &Candidate<I>) -> Option<(usize, SearchCull<I>)> {
    // If the program contains an instruction followed by one which overwrites its effects, then
    // the first instruction is obvious dead code. This can obviously be culled from the bruteforce
    // search.

    let destroy_carry = [
        // asl
        0x0a, 0x06, 0x16, 0x0e, 0x1e,
        // cmp
        0xc9, 0xc5, 0xd5, 0xcd, 0xdd, 0xd9, 0xc1, 0xd1,
        // cpx
        0xe0, 0xe4, 0xec,
        // cpy
        0xc0, 0xc4, 0xcc,
        // clc
        0x18,
        // sec
        0x38,
        // lsr
        0x4a, 0x46, 0x56, 0x4e, 0x5e];

    let destroy_accumulator = [
        // lda
        0xa9, 0xa5, 0xb5, 0xad, 0xbd, 0xb9, 0xa1, 0xb1, 
        // pha
        0x48,
        // txa
        0x8a,
        // tya
        0x98,
    ];

    for a in [0x18, 0x38] {
        // There's no point setting or clearing the carry flag, just to set it to some other value
        if let Some(res) = check(candidate, a, &destroy_carry) {
            return Some(res);
        }
    }

    for a in [0xa9, 0xa5, 0xb5, 0xad, 0xbd, 0xb9, 0xa1, 0xb1, 0x8a, 0x98] {
        // There's no point using tya, txa or lda to load the accumulator, just to overwrite the
        // accumulator
        if let Some(res) = check(candidate, a, &destroy_accumulator) {
            return Some(res);
        }
    }
    None
}

impl Peephole for Nmos6502Instruction {
    fn peephole(candidate: &Candidate<Self>) -> (usize, SearchCull<Self>) {
        if let Some(res) = dead_code(candidate) {
            res
        } else {
            (0, SearchCull::Okay)
        }
    }
}

impl Peephole for Cmos6502Instruction {
    fn peephole(candidate: &Candidate<Self>) -> (usize, SearchCull<Self>) {
        if let Some(res) = dead_code(candidate) {
            res
        } else {
            (0, SearchCull::Okay)
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    #[ignore] // the test takes too long, because I don't have other ways to cull bruteforce searches (yet)
    fn t() {
        use crate::Bruteforce;
        use crate::search::BruteForce;
        use crate::mos6502::Nmos6502Instruction;
        use crate::SearchAlgorithm;
        let unoptimized_count = Nmos6502Instruction::bruteforce_search().limit_length(2).iter().count();
        let optimized_count = Nmos6502Instruction::bruteforce_search().peephole().limit_length(2).iter().count();
        println!("There are {} 6502 programs of length 1 or 2", unoptimized_count);
        println!("The peephole optimizer removed {} from consideration", unoptimized_count - optimized_count);
        panic!();
    }
}
