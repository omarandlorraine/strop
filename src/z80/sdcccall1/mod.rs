use crate::dataflow::DataFlow;
use crate::dataflow::NotLiveIn;
use crate::test::Vals;
use crate::z80::dataflow::Register;
use crate::z80::register_pairs::RegPairFixup;
use crate::z80::subroutine::Subroutine;
use crate::z80::Emulator;
use crate::z80::Insn;
use crate::Callable;
use crate::Goto;
use crate::Iterable;
use crate::Sequence;
use crate::StropError;

mod constraints;

pub use constraints::SdccCall1Constraint;

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
    seq: Sequence<Insn>,
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

    /// Instantiates a strop::Generate object that searches over functions complying with the
    /// sdcccall(1) ABI.
    pub fn stochastic<C: Clone + Callable<Params, RetVal>>(
        self,
        target_function: C,
    ) -> crate::Generate<Params, RetVal, C, SdccCall1<Params, RetVal>> {
        crate::genetic::Generate::new(target_function)
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
        self.seq.dasm()
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> AsRef<crate::Sequence<Insn>>
    for SdccCall1<Params, RetVal>
{
    fn as_ref(&self) -> &crate::Sequence<Insn> {
        &self.seq
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> std::ops::Deref for SdccCall1<Params, RetVal> {
    type Target = Sequence<Insn>;

    fn deref(&self) -> &Self::Target {
        &self.seq
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> std::ops::DerefMut for SdccCall1<Params, RetVal> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.seq
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> SdccCall1<Params, RetVal> {
    /// Performs dataflow analysis on the function
    pub fn dataflow_analysis(&mut self) {
        for f in Params::live_in() {
            self.seq.make_read(&f);
        }
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> Callable<Params, RetVal>
    for SdccCall1<Params, RetVal>
{
    fn call(&self, input: Params) -> Result<RetVal, StropError> {
        let mut emu = Emulator::default();
        input.put(&mut emu);
        emu.run(&self.seq)?;
        Ok(RetVal::get(&emu))
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> crate::Goto<Insn> for SdccCall1<Params, RetVal> {
    fn goto(&mut self, t: &[Insn]) {
        self.seq.goto(t);
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> crate::Iterable for SdccCall1<Params, RetVal> {
    fn first() -> Self {
        Self {
            seq: crate::Iterable::first(),
            params: Default::default(),
            return_value: Default::default(),
            leaf_function: false,
            pure_function: false,
            peep_enable: false,
        }
    }

    fn step(&mut self) -> bool {
        use crate::Constrain;
        use crate::Goto;
        let mut sub = self.seq.clone();

        sub.step();
        while let Some(_r) = self.fixup(&mut sub) {}
        self.goto(&sub);
        true
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> crate::Constrain<Insn>
    for SdccCall1<Params, RetVal>
{
    fn fixup(&self, seq: &mut crate::Sequence<Insn>) -> Option<(usize, &'static str)> {
        if let Some(r) = Subroutine.fixup(seq) {
            return Some(r);
        }
        if let Some(r) = RegPairFixup().fixup(seq) {
            return Some(r);
        }
        if let Some(r) = SdccCall1Constraint().fixup(seq) {
            return Some(r);
        }
        if self.peep_enable {
            if let Some(r) = crate::peephole::PeepholeOptimizer::default().fixup(seq) {
                return Some(r);
            }
        }
        for reg in Register::all() {
            if !Params::live_in().contains(&reg) {
                if let Some(r) = NotLiveIn::new(reg).fixup(seq) {
                    return Some(r);
                }
            }
        }
        None
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> crate::Crossover for SdccCall1<Params, RetVal> {
    fn crossover(a: &Self, b: &Self) -> Self {
        use crate::Constrain;
        let mut sub = Sequence::<Insn>::crossover(&a.seq, &b.seq);

        sub.step();
        while let Some(_r) = a.fixup(&mut sub) {}
        let mut s = a.clone();
        s.goto(&sub);
        s
    }
}

impl<Params: ParameterList, RetVal: ReturnValue> crate::Mutate for SdccCall1<Params, RetVal> {
    fn random() -> Self {
        use crate::Iterable;
        let mut s = Self::first();
        s.seq = Sequence::random();
        s
    }

    fn mutate(&mut self) {
        use crate::Constrain;
        let mut sub = self.seq.clone();

        sub.mutate();
        while let Some(_r) = self.fixup(&mut sub) {}
        self.goto(&sub);
    }
}
