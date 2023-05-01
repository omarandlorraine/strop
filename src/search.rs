pub trait SearchConstraint<T> {
    /// Stochastic search over the constrained search space
    fn stochastic_search(self) -> StochasticSearch<T>
    where
        Self: Sized,
    {
        StochasticSearch::<T>::default()
    }

    /// Exhaustive search over the constrained search space
    fn exhaustive_search(self) -> ExhaustiveSearch<T>
    where
        Self: Sized,
    {
        ExhaustiveSearch::<T>::default()
    }

    /// Given an instruction of type `T`, return either `true`, or if the search constraint
    /// disallows this particular T, return `false`. This is used by objects of type `Search<T>` to
    /// constrain the search space.
    /// For example, a `SearchConstraint<T>` could be used to yield only programs that do not have
    /// any conditional branches (i.e., basic blocks), jumps, returns or other instructions
    /// affecting control flow. Such a SearchConstraint would return `true` if `t` is such an
    /// instruction.
    fn reject(&self, t: T) -> bool;
}

trait Search<T> {
    /// Return one putative program
    fn search(&mut self) -> Vec<T>;
}

trait Cost<T> {
    /// Given a program (that is, a sequence of type T), return a value to optimize for. The
    /// exhaustive search strategy stops when this function returns zero. The stochastic search
    /// strategy also stops when this function returns zero, but also uses information about *how
    /// wrong* a program is to inform its next move.
    fn cost(p: &[T]) -> f32;
}

trait Parameter<T> {
    /// If the instruction T reads from the parameter's location, return true. Otherwise, ask the
    /// next parameter. But if there are no more parameters in the list, then return false.
    fn permit(t: T) -> bool;
}

pub struct StochasticSearch<T> {
    a: Vec<T>,
    b: Vec<T>,
}

impl<T> Default for StochasticSearch<T> {
    fn default() -> Self {
        Self {
            a: vec![],
            b: vec![],
        }
    }
}

pub struct ExhaustiveSearch<T> {
    current: Vec<T>,
}

impl<T> Default for ExhaustiveSearch<T> {
    fn default() -> Self {
        Self { current: vec![] }
    }
}
