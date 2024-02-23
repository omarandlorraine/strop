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

fn bruteforce32(label: &'static str, func: fn(u32) -> Option<u32>) {
    use strop::search::StochasticSearch;
    use strop::z80::instruction_set::Z80Instruction;
    use strop::z80::ZilogZ80;
    use strop::SearchAlgorithm;

    let mut iterator = StochasticSearch::<Z80Instruction>::new()
        .compatibility(ZilogZ80)
        .z88dkfastcall(func);
    let mut iterator = iterator.iter();

    println!("{}:", label);
    iterator.next().unwrap().disassemble();
}

fn bruteforce16(label: &'static str, func: fn(u16) -> Option<u16>) {
    use strop::search::StochasticSearch;
    use strop::z80::instruction_set::Z80Instruction;
    use strop::z80::ZilogZ80;
    use strop::SearchAlgorithm;

    let mut iterator = StochasticSearch::<Z80Instruction>::new()
        .compatibility(ZilogZ80)
        .z88dkfastcall(func);
    let mut iterator = iterator.iter();

    println!("{}:", label);
    iterator.next().unwrap().disassemble();
}

fn main() {
    bruteforce16("htons", htons);
    bruteforce32("htonl", htonl);
    bruteforce32("ntohl", ntohl);
    bruteforce16("ntohs", ntohs);
}
