#!/usr/local/bin/python3

import sys
import json


with open("Opcodes.json", mode="r", encoding="utf-8") as read_file:
    op_data = json.load(read_file)
    
def iprint(opcode, i):
    if i['mnemonic'].startswith('ILLEGAL'):
        print('None,');
        return 
    if i['mnemonic'] == 'PREFIX':
        print('None,');
        return 

    def parse_flag(t):
        if t == "-":
            return "ReadWrite::N"
        return "ReadWrite::W"

    zero = parse_flag(i['flags']['Z'])
    negative = parse_flag(i['flags']['N'])
    half_carry = parse_flag(i['flags']['H'])
    carry = parse_flag(i['flags']['C'])

    a = "ReadWrite::N"
    b = "ReadWrite::N"
    c = "ReadWrite::N"
    d = "ReadWrite::N"
    e = "ReadWrite::N"
    h = "ReadWrite::N"
    l = "ReadWrite::N"
    r = "ReadWrite::N"
    ixh = "ReadWrite::N"
    ixl = "ReadWrite::N"
    iyh = "ReadWrite::N"
    iyl = "ReadWrite::N"
    sp = "ReadWrite::N"

    first = True

    for operand in i['operands']:
        rw_if_first = "ReadWrite::W" if first else "ReadWrite::R"
        if operand['name'] in ["NZ", "Z"]:
            zero = "ReadWrite::R"
            first = False
            continue
        if operand['name'] in ["NC", "C"]:
            carry = "ReadWrite::R"
            first = False
            continue
        if operand['name'] == "n8":
            first = False
            continue
        if operand['name'] == "n16":
            first = False
            continue
        if operand['name'] == "a16":
            first = False
            continue
        if operand['name'] == "a8":
            first = False
            continue
        if operand['name'] == "e8":
            first = False
            continue
        if operand['name'] in ['0', '1', '2', '3', '4', '5', '6', '7',]:
            first = False
            continue
        if operand['name'] in ['$00', '$08', '$10', '$18', '$20', '$28', '$30', '$38', ]:
            first = False
            continue
        if operand['name'] == "BC":
            c = rw_if_first
            b = rw_if_first
            first = False
            continue
        if operand['name'] == "HL":
            if operand.get('decrement') or operand.get('increment'):
                h = "ReadWrite::Rmw"
                l = "ReadWrite::Rmw"
            else:
                h = rw_if_first
                l = rw_if_first
            first = False
            continue
        if operand['name'] == "DE":
            e = rw_if_first
            d = rw_if_first
            first = False
            continue
        if operand['name'] == "AF":
            a = rw_if_first
            first = False
            continue
        if operand['name'] == "A":
            a = rw_if_first
            first = False
            continue
        if operand['name'] == "B":
            b = rw_if_first
            first = False
            continue
        if operand['name'] == "D":
            d = rw_if_first
            first = False
            continue
        if operand['name'] == "E":
            e = rw_if_first
            first = False
            continue
        if operand['name'] == "H":
            h = rw_if_first
            first = False
            continue
        if operand['name'] == "L":
            l = rw_if_first
            first = False
            continue
        if operand['name'] == "SP":
            sp = rw_if_first
            first = False
            continue

        print(f"What does {operand['name']} mean")
        sys.exit(1)
        
    dasm = ' '.join([i['mnemonic']] + [op['name'] if op['immediate'] else f'({op["name"]})' for op in i['operands']])

    print('Some(InstructionData {')
    print(f'\t// {dasm}')
    print(f'\tmnemonic: "{i["mnemonic"].lower()}",')
    print(f'\topcode: {opcode.lower()},')
    print(f'\tbytes: {i["bytes"]},')
    print(f'\tcycles: {i["cycles"][0]},')
    print(f'\tzero: {zero},'),
    print(f'\tnegative: {negative},'),
    print(f'\thalf_carry: {half_carry},'),
    print(f'\tcarry: {carry},'),
    print(f"\ta: {a},");
    print(f"\tb: {b},");
    print(f"\tc: {c},");
    print(f"\td: {d},");
    print(f"\te: {e},");
    print(f"\th: {h},");
    print(f"\tl: {l},");
    print(f"\tr: {r},");
    print(f"\tixh: {ixh},");
    print(f"\tixl: {ixl},");
    print(f"\tiyh: {iyl},");
    print(f"\tiyl: {iyl},");
    print(f"\tsp: {sp},");
    print(f"\ti: ReadWrite::N,");
    print('}),')

print('use crate::sm83::InstructionData;')
print('use crate::sm83::ReadWrite;')

print('pub const UNPREFIXED: [Option<InstructionData>; 256] = [');
print(max(len(data['operands']) for data in op_data['unprefixed'].values()))
print('];');

print('pub const CBPREFIXED: [Option<InstructionData>; 256] = [');
print(max(len(data['operands']) for data in op_data['unprefixed'].values()))
print('];');

