use crate::instruction::Instruction;
use crate::snippets::Snippet;
use rand::random;
use rand::thread_rng;
use rand::Rng;

pub struct Random<I: Instruction> {
    /// Iterator yielding random snippets of the given instruction type, and having any number of
    /// instructions up to the specified length.
    org: usize,
    max_length: usize,
    fitness: fn(&Snippet<I>) -> f64,

    // for some reason I need this because an unused type parameter is a type error
    #[allow(dead_code)]
    dummy: I,
}

impl<I: Instruction> Random<I> {
    pub fn new(org: usize, max_length: usize, fitness: fn(&Snippet<I>) -> f64) -> Self {
        Self {
            org,
            max_length,
            fitness,
            dummy: I::new(),
        }
    }
}

impl<I: Instruction + std::fmt::Display + Copy> Iterator for Random<I> {
    type Item = (f64, Snippet<I>);

    fn next(&mut self) -> Option<Self::Item> {
        let sn = Snippet::<I>::new_with_org_and_length(self.org, self.max_length);
        Some(((self.fitness)(&sn), sn))
    }
}

pub struct McmcSynth<'a, I: Instruction> {
    /// Iterator yielding mutations (children) of a given snippet, depending on their relative
    /// scores. If the child snippet has a better (i.e. lower) score, then it's yielded. But if the
    /// child snippet has a worse score, then with some probability depending on how much worse it
    /// is, it's discarded.
    /// Use this together with a cost function which evaluates the program's fitness
    parent: &'a Snippet<I>,
    child: Snippet<I>,
    fitness: fn(&Snippet<I>) -> f64,
    cost: f64,
}

impl<'a, I: Instruction> McmcSynth<'a, I> {
    pub fn new(parent: &'a Snippet<I>, fitness: fn(&Snippet<I>) -> f64) -> Self {
        let cost = fitness(parent);
        let child = parent.clone();
        Self {
            parent,
            child,
            fitness,
            cost,
        }
    }
}

impl<I: Instruction + std::fmt::Display + Copy> Iterator for McmcSynth<'_, I> {
    type Item = (f64, Snippet<I>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // mutate the child randomly
            self.child.mutate();
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
