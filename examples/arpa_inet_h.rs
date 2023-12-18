use strop::InstructionSet;

// A program to generate the arpa/inet.h library for the Z80.


fn htonl(val:u32) -> Option<u32> {
    Some(u32::from_be_bytes(val.to_le_bytes()))
}
fn htons(val:u16) -> u16{
    u16::from_be_bytes(val.to_le_bytes())
}
fn ntohl(val:u32) -> u32{
    u32::from_be_bytes(val.to_le_bytes())
}
fn ntohs(val:u16) -> u16{
    u16::from_be_bytes(val.to_le_bytes())
}

fn main() {
    // These functions are so simple, mapping to a single ARM instruction, that a bruteforce search works okay.
    let htons = strop::z80::z80()
        .bruteforce_with_maximum_length(5)
        .z88dkfastcall(htonl)
        .next()
        .unwrap();
}
