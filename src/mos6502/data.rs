//! The `Instruction6502` type, for representing a MOS 6502 instruction.
use yaxpeax_6502::Opcode;
use yaxpeax_6502::Opcode::*;

use yaxpeax_6502::Instruction;

use rand::prelude::SliceRandom;

pub const ABS_OPCODES: [Opcode; 23] = [JMP, ADC, SBC, INC, DEC, STA, STX, STY, LDA, LDX, LDY, AND, ASL, BIT, CMP, CPX, CPY, EOR, JSR, LSR, ORA, ROL, ROR];

pub const ABSX_OPCODES: [Opcode; 15] = [ADC, SBC, INC, DEC, AND, ASL, CMP, EOR, LDA, LDY, LSR, ORA, ROL, ROR, STA];

pub const ABSY_OPCODES: [Opcode; 9] = [ ADC, AND, CMP, EOR, LDA, LDX, ORA, SBC, STA];

pub const ACC_OPCODES: [Opcode; 4] = [ASL, ROR, ROL, LSR];

pub const IMM_OPCODES: [Opcode; 11] = [ORA, ADC, SBC, AND, CMP, CPX, CPY, EOR, LDA, LDX, LDY ];

pub const IMP_OPCODES: [Opcode; 25] = [CLC, CLD, SEC, SED, SEI, CLI, RTI, BRK, CLV, DEX, DEY, INX, INY, NOP, PHA, PLA, PLP, RTS, TAX, TAY, TSX, TXS, TYA, TXA, PHP ];

pub const IND_OPCODES: [Opcode; 1] = [JMP];

pub const INDX_OPCODES: [Opcode; 8] = [ ADC, AND, CMP, EOR, LDA, ORA, SBC, STA];

pub const INDY_OPCODES: [Opcode; 8] = [ ADC, AND, CMP, EOR, LDA, ORA, SBC, STA];

pub const REL_OPCODES: [Opcode; 8] = [BCC, BCS, BEQ, BNE, BVC, BVS, BMI, BPL];

pub const ZP_OPCODES: [Opcode; 21] = [ROL, INC, DEC, LSR, ROR, ASL, CPY, ADC, AND, BIT, CMP, CPX, EOR, LDA, LDX, LDY, ORA, SBC, STA, STX, STY];

pub const ZPX_OPCODES: [Opcode; 16] = [ ADC , AND , ASL , CMP , DEC , EOR , INC , LDA , LDY , LSR , ORA , ROL , ROR , SBC , STA , STY ];

pub const ZPY_OPCODES: [Opcode; 2] = [LDX, STX];
