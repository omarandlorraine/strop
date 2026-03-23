use crate::backends::x80::X80;

// checks if the flow control bit is set correctly in the instruction data thing. basically it has
// to agree with what the opcode says it is.
fn flow_control<I: X80>(insn: &I) {
}

fn length<I: X80>(insn: &I) {
}
