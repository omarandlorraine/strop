// A program to generate the arpa/inet.h library for the Z80.

fn htonl(val: u32) -> Option<u32> {
    Some(u32::from_be_bytes(val.to_le_bytes()))
}

fn htons(val: u16) -> Option<u16> {
    Some(u16::from_be_bytes(val.to_le_bytes()))
}

fn ntohl(val: u32) -> Option<u32> {
    Some(u32::from_be_bytes(val.to_le_bytes()))
}

fn ntohs(val: u16) -> Option<u16> {
    Some(u16::from_be_bytes(val.to_le_bytes()))
}

fn bruteforce<T: strop::Scalar>(label: &'static str, func: fn(T) -> Option<T>) {
    use strop::z80::instruction_set::Z80Instruction;
    use strop::z80::Z80Search;
    use strop::z80::ZilogZ80;
    use strop::SearchAlgorithm;
    use strop::Stochastic;

    println!("{}:", label);
    Z80Instruction::stochastic_search()
        .compatibility(ZilogZ80)
        .z88dkfastcall(func)
        .iter()
        .next()
        .unwrap()
        .disassemble();
}

fn main() {
    bruteforce("htons", htons);
    bruteforce("htonl", htonl);
    bruteforce("ntohl", ntohl);
    bruteforce("ntohs", ntohs);
}
