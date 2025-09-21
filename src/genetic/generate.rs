use crate::Callable;
use crate::Crossover;
use crate::Mutate;
use crate::TestSuite;
use crate::test::{Input, Output};

use crate::genetic::ScoredCandidate;

/// A genetic algorithm for finding good functions
#[allow(dead_code)] // TODO: target_function is not used.
#[derive(Debug)]
pub struct Generate<
    InputParameters,
    ReturnValue,
    T: Callable<InputParameters, ReturnValue>,
    U: Callable<InputParameters, ReturnValue> + Mutate + Crossover,
> {
    target_function: T,
    population: Vec<ScoredCandidate<InputParameters, ReturnValue, U>>,
    tests: TestSuite<InputParameters, ReturnValue>,
    popsize: usize,
    input: std::marker::PhantomData<InputParameters>,
    ret: std::marker::PhantomData<ReturnValue>,
    crossover_rate: f64,
    mutation_rate: f64,
}

impl<
    InputParameters: Input,
    ReturnValue: Output,
    T: Callable<InputParameters, ReturnValue>,
    U: Callable<InputParameters, ReturnValue> + Mutate + Crossover + Clone + crate::Disassemble,
> Generate<InputParameters, ReturnValue, T, U>
{
    /// Constructs a new `GeneticSearch`
    pub fn new(target_function: T) -> Self {
        Self {
            tests: TestSuite::generate(&target_function),
            target_function,
            population: vec![],
            popsize: 100,
            input: Default::default(),
            ret: Default::default(),
            crossover_rate: 2.0,
            mutation_rate: 1.0,
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
        use rand::Rng;
        use rand::rng;

        let mut next_generation = vec![];

        for _ in 0..self.popsize {
            let total_fitness: f64 = self.population.iter().map(|ind| ind.score).sum();

            let mut pick = rng().random_range(0.0..total_fitness);

            for i in 0..(self.population.len() - 1) {
                pick -= self.population[i].score;
                if pick <= 0.0 {
                    next_generation.push(self.population.remove(i));
                    break;
                }
            }
        }
        next_generation
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
        self.population.push(ScoredCandidate::new(
            U::crossover(parent_a.as_ref(), parent_b.as_ref()),
            &self.tests,
        ));
        self.population.push(ScoredCandidate::new(
            U::crossover(parent_b.as_ref(), parent_a.as_ref()),
            &self.tests,
        ));
    }

    /// Adds more specimens to the population by mutation (this is taking one specimen, copying it,
    /// and then mutating the copy. The mutated copy is then added to the population).
    fn mutate(&mut self, parent: &ScoredCandidate<InputParameters, ReturnValue, U>) {
        let mut child = parent.candidate.clone();
        child.mutate();
        self.population
            .push(ScoredCandidate::new(child, &self.tests));
    }

    /// computes the next generation of the search
    pub fn next_generation(&mut self) {
        use rand::Rng;
        let mut rng = rand::rng();
        if self.population.is_empty() {
            self.population
                .push(ScoredCandidate::new(U::random(), &self.tests));
        }
        let total_fitness: f64 = self.population.iter().map(|ind| ind.score).sum();
        println!(
            "population size: {} total_fitness: {}",
            self.population.len(),
            total_fitness
        );
        if self.population.len() == 1 {
            self.population[0].candidate.dasm();
        }

        for _ in 0..((self.popsize as f64 * self.crossover_rate) as usize) {
            let parent_a = rng.random_range(0..self.population.len());
            let parent_b = rng.random_range(0..self.population.len());
            let parent_a = self.population[parent_a].clone();
            let parent_b = self.population[parent_b].clone();
            self.crossover(&parent_a, &parent_b);
        }
        for _ in 0..((self.popsize as f64 * self.mutation_rate) as usize) {
            let parent = rng.random_range(0..self.population.len());
            let parent = &self.population[parent].clone();
            self.mutate(parent);
        }
        self.selection();
    }

    fn offset_to_first_zero(&self) -> Option<usize> {
        self.population.iter().position(|x| x.score == 0.0)
    }

    /// Returns one specimen having a score of 0.0, removing it from the population. If no such
    /// specimen exists, then `.next_generation()` is called until one is found.
    pub fn search(&mut self) -> U {
        while self.offset_to_first_zero().is_none() {
            self.next_generation();
        }
        let specimen = self.population.remove(self.offset_to_first_zero().unwrap());
        specimen.candidate
    }
}
