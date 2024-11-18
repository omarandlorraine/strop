use crate::dataflow::NotLiveIn;
use crate::dataflow::NotLiveOut;
use crate::test::Vals;
use crate::z80::dataflow::Register;
use crate::z80::register_pairs::RegPairFixup;
use crate::z80::subroutine::Subroutine;
use crate::z80::Emulator;
use crate::z80::Insn;
use crate::Callable;
use crate::DataFlow;
use crate::StropError;

fn allowed(insn: &Insn) -> bool {
    // I don't think that a functino should meddle with things like interrupts, halting the
    // processor, loading or storing the stack pointer, etc. So I have just removed these
    // instructions from the search.
    use crate::Encode;
    let enc = insn.encode();

    if enc[0] == 0x31 {
        // ld sp, nn
        return false;
    }

    if enc[0] == 0x76 {
        // halt
        return false;
    }

    if matches!(enc[0], 0xf3 | 0xfb) {
        // di and ei
        return false;
    }

    if enc[0] == 0xed {
        if enc[1] == 0x45 {
            // retn
            return false;
        }

        if enc[1] == 0x4d {
            // reti
            return false;
        }

        if matches!(enc[1], 0x46 | 0x56 | 0x5e) {
            // im 0/1/2
            return false;
        }

        if enc[1] == 0x73 {
            // ld (nn), sp
            return false;
        }
    }
    true
}

pub trait ParameterList: Copy + Vals {
    fn put(&self, emu: &mut Emulator);
    fn live_in() -> Vec<Register>;
}

impl ParameterList for u8 {
    fn put(&self, emu: &mut Emulator) {
        emu.set_a(*self);
    }
    fn live_in() -> Vec<Register> {
        vec![Register::A]
    }
}

impl ParameterList for u16 {
    fn put(&self, emu: &mut Emulator) {
        emu.set_hl(*self);
    }
    fn live_in() -> Vec<Register> {
        vec![Register::H, Register::L]
    }
}

// TODO: Implement this for more types. The calling convention supports return signed types, 32-bit
// types, and perhaps others which are not supported (yet)
pub trait ReturnValue: Copy + Vals + PartialEq {
    fn get(emu: &Emulator) -> Self;
    fn live_out() -> Vec<Register>;
}

impl ReturnValue for u8 {
    fn get(emu: &Emulator) -> u8 {
        emu.get_a()
    }
    fn live_out() -> Vec<Register> {
        vec![Register::A]
    }
}

impl ReturnValue for i8 {
    fn get(emu: &Emulator) -> i8 {
        emu.get_a() as i8
    }
    fn live_out() -> Vec<Register> {
        vec![Register::A]
    }
}

impl ReturnValue for u16 {
    fn get(emu: &Emulator) -> u16 {
        emu.get_hl()
    }
    fn live_out() -> Vec<Register> {
        vec![Register::H, Register::L]
    }
}

impl ReturnValue for i16 {
    fn get(emu: &Emulator) -> i16 {
        emu.get_hl() as i16
    }
    fn live_out() -> Vec<Register> {
        vec![Register::H, Register::L]
    }
}

/// Mimics the calling convention used by modern-day SDCC. SDCC's internal documentation calls this
/// `__sdcccall(1)`.
#[derive(Clone, Debug)]
pub struct SdccCall1<Params: Copy + Vals, RetVal: Copy + Vals> {
    subroutine: Subroutine,
    params: std::marker::PhantomData<Params>,
    return_value: std::marker::PhantomData<RetVal>,
    peep_enable: bool,
    pure_function: bool,
    leaf_function: bool,
}

impl<Params: ParameterList, RetVal: ReturnValue> Default for SdccCall1<Params, RetVal> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> SdccCall1<Params, RetVal> {
    /// Instantiates a new, empty SdccCall1.
    pub fn new() -> Self {
        use crate::Iterable;
        Self::first()
    }

    /// Instantiates a strop::BruteForce object that searches over functions complying with the
    /// sdcccall(1) ABI.
    pub fn bruteforce<C: Clone + Callable<Params, RetVal>>(
        mut self,
        target_function: C,
    ) -> crate::BruteForce<Params, RetVal, C, SdccCall1<Params, RetVal>, Insn> {
        self.peep_enable = true;
        crate::BruteForce::new(target_function, self)
    }

    /// Makes sure that the search space includes only pure functions (i.e., those that do not have
    /// side effects, and do not observe any state other than its parameters).
    pub fn pure(&mut self) -> Self {
        self.pure_function = true;
        self.clone()
    }

    /// Enables peephole optimization
    pub fn peephole_optimization(&mut self) -> Self {
        self.peep_enable = true;
        self.clone()
    }

    /// Makes sure that the search space includes only leaf functions (i.e., those that do not call
    /// other functions)
    pub fn leaf(&mut self) -> Self {
        self.leaf_function = true;
        self.clone()
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> crate::Disassemble for SdccCall1<Params, RetVal> {
    fn dasm(&self) {
        self.subroutine.dasm()
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> AsRef<crate::Sequence<Insn>>
    for SdccCall1<Params, RetVal>
{
    fn as_ref(&self) -> &crate::Sequence<Insn> {
        self.subroutine.as_ref()
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> std::ops::Deref for SdccCall1<Params, RetVal> {
    type Target = Subroutine;

    fn deref(&self) -> &Self::Target {
        &self.subroutine
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> std::ops::DerefMut for SdccCall1<Params, RetVal> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.subroutine
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> SdccCall1<Params, RetVal> {
    /// Performs dataflow analysis on the function
    pub fn dataflow_analysis(&mut self) {
        for f in Params::live_in() {
            self.subroutine.make_read(&f);
        }
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> Callable<Params, RetVal>
    for SdccCall1<Params, RetVal>
{
    fn call(&self, input: Params) -> Result<RetVal, StropError> {
        let mut emu = Emulator::default();
        input.put(&mut emu);
        emu.run(&self.subroutine)?;
        Ok(RetVal::get(&emu))
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> crate::Goto<Insn> for SdccCall1<Params, RetVal> {
    fn goto(&mut self, t: &[Insn]) {
        self.subroutine.goto(t);
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> crate::Iterable for SdccCall1<Params, RetVal> {
    fn first() -> Self {
        Self {
            subroutine: crate::Iterable::first(),
            params: Default::default(),
            return_value: Default::default(),
            leaf_function: false,
            pure_function: false,
            peep_enable: false,
        }
    }

    fn step(&mut self) -> bool {
        self.subroutine.step()
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> crate::Constrain<Insn>
    for SdccCall1<Params, RetVal>
{
    fn fixup(&mut self) {
        self.subroutine.fixup();
        for offset in 0..(self.len() - 1) {
            if !allowed(&self.subroutine[offset]) {
                self.mut_at(Insn::next_opcode, offset);
            }
            if !self.subroutine[offset].allowed_in_pure_functions() && self.pure_function {
                self.mut_at(Insn::next_opcode, offset);
            }
            RegPairFixup(&mut self.subroutine).fixup();
            if self.peep_enable {
                crate::peephole::PeepholeOptimizer::new(&mut self.subroutine).fixup();
            }
            for reg in Register::all() {
                if !Params::live_in().contains(&reg) {
                    NotLiveIn::new(&mut self.subroutine, reg).fixup();
                }
                if !RetVal::live_out().contains(&reg) {
                    NotLiveOut::new(&mut self.subroutine, reg).fixup();
                }
            }
        }
    }

    fn report(&self, offset: usize) -> Vec<String> {
        let mut report = self.subroutine.report(offset);
        if !allowed(&self.subroutine[offset]) {
            report.push("This opcode is disallowed in sdcccall(1)".to_string());
        }
        if !self.subroutine[offset].allowed_in_pure_functions() && self.pure_function {
            report.push("This instruction is disallowed in pure functions".to_string());
        }
        for r in RegPairFixup(&mut self.subroutine.clone()).report(offset) {
            report.push(r);
        }
        for r in
            crate::peephole::PeepholeOptimizer::new(&mut self.subroutine.clone()).report(offset)
        {
            report.push(r);
        }
        for reg in Register::all() {
            if !Params::live_in().contains(&reg) {
                for r in NotLiveIn::new(&mut self.subroutine.clone(), reg).report(offset) {
                    report.push(r);
                }
            }
        }
        report
    }
}
