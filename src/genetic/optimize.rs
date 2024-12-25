use crate::test;
use crate::test::Vals;
use crate::Callable;
use crate::Crossover;
use crate::Mutate;
use crate::Objective;
use crate::StropError;

use crate::genetic::ScoredCandidate;

/// A genetic algorithm for optimizing functions
#[allow(dead_code)] // TODO: target_function is not used.
#[derive(Debug)]
pub struct Optimize<
    InputParameters,
    ReturnValue,
    T: Callable<InputParameters, ReturnValue>,
    U: Callable<InputParameters, ReturnValue> + Mutate + Crossover,
    Obj: Objective<U>,
> {
    target_function: T,
    population: Vec<ScoredCandidate<InputParameters, ReturnValue, U>>,
    objective: Obj,
    tests: Vec<(InputParameters, ReturnValue)>,
    popsize: usize,
    input: std::marker::PhantomData<InputParameters>,
    ret: std::marker::PhantomData<ReturnValue>,
    crossover_rate: f64,
    mutation_rate: f64,
}

impl<
        InputParameters: Vals,
        ReturnValue: Vals,
        T: Callable<InputParameters, ReturnValue>,
        U: Callable<InputParameters, ReturnValue> + Mutate + Crossover + Clone,
        Obj: Objective<U>,
    > Optimize<InputParameters, ReturnValue, T, U, Obj>
{
    /// Constructs a new `GeneticSearch`
    pub fn new(target_function: T, objective: Obj) -> Self {
        Self {
            tests: crate::test::quick_tests(&target_function),
            target_function,
            population: vec![],
            popsize: 100,
            input: Default::default(),
            ret: Default::default(),
            crossover_rate: 2.0,
            mutation_rate: 1.0,
            objective,
        }
    }

    /// Sets the population size
    pub fn population_size(&mut self, population_size: usize) {
        self.popsize = population_size;
    }

    /// Sets the crossover_rate (the default is 2.0)
    pub fn crossover_rate(&mut self, crossover_rate: f64) {
        self.crossover_rate = crossover_rate;
    }

    /// Sets the mutation_rate (the default is 1.0)
    pub fn mutation_rate(&mut self, mutation_rate: f64) {
        self.mutation_rate = mutation_rate;
    }

    fn roulette_selection(&mut self) -> Vec<ScoredCandidate<InputParameters, ReturnValue, U>> {
        use rand::thread_rng;
        use rand::Rng;

        let mut next_generation = vec![];

        for _ in 0..self.popsize {
            let total_fitness: f64 = self.population.iter().map(|ind| ind.score).sum();

            let mut pick = thread_rng().gen_range(0.0..total_fitness);

            for i in 0..self.population.len() {
                pick -= self.population[i].score;
                if pick <= 0.0 {
                    next_generation.push(self.population.remove(i));
                }
            }
        }
        next_generation
    }

    /// Adds a candidate to the population (but only if it passes the tests!)
    pub fn push(&mut self, candidate: U) -> bool {
        match test::passes(&candidate, &self.tests) {
            Err(StropError::DidntReturn) => {
                // The candidate does not pass the test case(s)
                false
            }
            Err(StropError::Undefined) => {
                // The candidate does not pass the test case(s)
                false
            }
            Ok(false) => {
                // The candidate does not pass the test case(s)
                false
            }
            Ok(true) => {
                // the candidate passes all known test cases; let's fuzz test it
                if let Some(test_case) = test::fuzz(&self.target_function, &candidate, 5000) {
                    // We've fuzzed the functions against eachother and found another test case.
                    // So keep hold of this new test case
                    self.tests.push(test_case);
                    false
                } else {
                    // The candidate passed all known test cases and also a fuzz test, so let's say
                    // it's good enough and add it to the population
                    let score = self.objective.score(&candidate);
                    self.population
                        .push(ScoredCandidate::new_with_score(candidate, score));
                    true
                }
            }
        }
    }

    /// Keeps the best specimens and removes the worst ones
    pub fn selection(&mut self) {
        let next_generation = self.roulette_selection();
        self.population = next_generation;
    }

    /// Adds more specimens to the population by crossover (this is like taking parents A and B,
    /// and the first half of A and the second half of B, making child C, and vice-versa making
    /// child D)
    fn crossover(
        &mut self,
        parent_a: &ScoredCandidate<InputParameters, ReturnValue, U>,
        parent_b: &ScoredCandidate<InputParameters, ReturnValue, U>,
    ) {
        self.push(U::crossover(parent_a.as_ref(), parent_b.as_ref()));
        self.push(U::crossover(parent_b.as_ref(), parent_a.as_ref()));
    }

    /// Adds more specimens to the population by mutation (this is taking one specimen, copying it,
    /// and then mutating the copy. The mutated copy is then added to the population).
    fn mutate(&mut self, parent: &ScoredCandidate<InputParameters, ReturnValue, U>) {
        let mut child = parent.candidate.clone();
        child.mutate();
        self.push(child);
    }

    /// computes the next generation of the search
    pub fn next_generation(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for _ in 0..((self.popsize as f64 * self.crossover_rate) as usize) {
            let parent_a = rng.gen_range(0..self.population.len());
            let parent_b = rng.gen_range(0..self.population.len());
            let parent_a = self.population[parent_a].clone();
            let parent_b = self.population[parent_b].clone();
            self.crossover(&parent_a, &parent_b);
        }
        for _ in 0..((self.popsize as f64 * self.mutation_rate) as usize) {
            let parent = rng.gen_range(0..self.population.len());
            let parent = &self.population[parent].clone();
            self.mutate(parent);
        }
        self.selection();
    }

    fn offset_to_first_zero(&self) -> Option<usize> {
        self.population.iter().position(|x| x.score == 0.0)
    }

    /// Returns one specimen having a score of 0.0, removing it from the population. If now such
    /// specimen exists, then `.next_generation()` is called until one is found.
    pub fn next_specimen(&mut self) -> U {
        while self.offset_to_first_zero().is_none() {
            self.next_generation();
        }
        let specimen = self.population.remove(self.offset_to_first_zero().unwrap());
        specimen.candidate
    }
}
