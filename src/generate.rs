use crate::instruction::Instruction;
use crate::snippets::Snippet;
use crate::static_analysis::VarState;
use rand::random;
use rand::thread_rng;
use rand::Rng;
use std::marker::PhantomData;

pub struct Random<'a, I: Instruction> {
    /// Iterator yielding random snippets of the given instruction type, and having any number of
    /// instructions up to the specified length.
    org: usize,
    max_length: usize,
    fitness: fn(&Snippet<I>) -> f64,

    // for some reason I need this because an unused type parameter is a type error
    #[allow(dead_code)]
    _marker: PhantomData<&'a I>,
}

impl<I: Instruction> Random<'_, I> {
    pub fn new(org: usize, max_length: usize, fitness: fn(&Snippet<I>) -> f64) -> Self {
        Self {
            org,
            max_length,
            fitness,
            _marker: PhantomData
        }
    }
}

impl<'a, I: Instruction + std::fmt::Display + Copy> Iterator for Random<'a, I> {
    type Item = (f64, Snippet<'a, I>);


    fn next(&mut self) -> Option<Self::Item> {
        let sn = Snippet::<I>::new_with_org_and_length(self.org, self.max_length);
        Some(((self.fitness)(&sn), sn))
    }
}

pub struct Constraints<I: Instruction> {
    /// In case there's something or other you want to exclude from a search, such as branches, or
    /// instructions touching this or that register, etc. etc., you can add them to this struct.
    /// This has the effect of constraining the search space to programs you want to permit.
    constraints: Vec<fn(&I) -> bool>,
    statics: Vec<fn(VarState, &I) -> VarState>,
}

impl<I: Instruction> Default for Constraints<I> {
    fn default() -> Self {
        Self {
            constraints: vec![],
            statics: vec![],
        }
    }
}

impl<I: Instruction> Constraints<I> {
    pub fn new(exclude: Vec<fn(&I) -> bool>) -> Self {
        Self {
            constraints: exclude,
            statics: vec![]
        }
    }

    pub fn allow(&self, insn: &I) -> bool {
        for f in &self.constraints {
            if !f(insn) {
                return false;
            }
        }
        true
    }

    pub fn new_instruction(&self) -> Option<I> {
        for _ in 0..5 {
            let insn = I::new();
            if self.allow(&insn) {
                return Some(insn);
            }
        }
        None
    }
}

pub struct McmcSynth<'a, I: Instruction> {
    /// Iterator yielding mutations (children) of a given snippet, depending on their relative
    /// scores. If the child snippet has a better (i.e. lower) score, then it's yielded. But if the
    /// child snippet has a worse score, then with some probability depending on how much worse it
    /// is, it's discarded.
    /// Use this together with a cost function which evaluates the program's fitness
    parent: Snippet<'a, I>,
    child: Snippet<'a, I>,
    fitness: fn(&Snippet<I>) -> f64,
    cost: f64,
    constraints: Constraints<I>,
}

impl<'a, I: Instruction> McmcSynth<'a, I> {
    pub fn new(
        parent: Snippet<'a, I>,
        constraints: Constraints<I>,
        fitness: fn(&Snippet<I>) -> f64,
    ) -> Self {
        let cost = fitness(&parent);
        let child = parent.clone();
        Self {
            parent,
            child,
            fitness,
            cost,
            constraints,
        }
    }
}

impl<'a, I: Instruction + std::fmt::Display + Copy> Iterator for McmcSynth<'a, I> {
    type Item = (f64, Snippet<'a, I>);


    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // mutate the child randomly
            self.child.mutate(&self.constraints);
            let cost = (self.fitness)(&self.child);

            // if the random mutation was an improvement, return the child
            if cost <= self.cost {
                return Some((cost, self.child.clone()));
            }

            // otherwise, with some probability depending on how much worse it was, return it
            // anyway
            let m: f64 = thread_rng().gen_range(0.0..cost);
            if m > self.cost {
                return Some((cost, self.child.clone()));
            }

            // still not returned? Then maybe replace the child with the parent. We might have
            // wondered off into the weeds and not going to find anything.
            if random() {
                self.child = self.parent.clone();
            }
        }
    }
}

pub struct CorrectPrograms<'a, I: Instruction> {
    /// Iterator yielding programs for which the supplied cost function returns 0.
    parent: Snippet<'a, I>,
    synth: McmcSynth<'a, I>
}

impl<'a, I: Instruction> CorrectPrograms<'a, I> {
    pub fn new(
        constraints: Constraints<I>,
        fitness: fn(&Snippet<I>) -> f64,
    ) -> Self {
        let parent = Snippet::<I>::default();
        Self {
            parent: parent.clone(),
            synth: McmcSynth::new(Snippet::<I>::default(), constraints, fitness)
        }
    }
}

impl<'a, I: Instruction + std::fmt::Display + Copy> Iterator for CorrectPrograms<'a, I> {
    type Item = (f64, Snippet<'a, I>);


    fn next(&mut self) -> Option<Self::Item> {

        // loop until we find at least one candidate program that at least computes the right result
        loop {
            let (score, sn) = self.synth.next().unwrap();

            if score == 0.0 {
                return Some((score, sn));
            }
        }
    }
}

