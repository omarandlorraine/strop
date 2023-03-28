trait Search<T> {
    fn search(&mut self) -> Option<T>;
}

trait ConstrainedSearch<T>: Search<T> {
    /// Given an instruction type T, return either the very same one, or if the search constraint
    /// disallows this particular T, mutate it to another one that *is* allowed.
    /// For example, a ConstrainedSearch could be used to yield only programs that do not have any
    /// conditional branches (i.e., basic blocks). Such a ConstrainedSearch would return some other
    /// instruction if T is a conditional branch instruction.
    fn mutate(t: T) -> T;
}

trait Parameter<T> {
    /// If the instruction T reads from the parameter's location, return true. Otherwise, ask the
    /// next parameter. But if there are no more parameters in the list, then return false.
    fn permit(t: T) -> bool;

}

struct StochasticSearch<T> {
    a: Vec<T>,
    b: Vec<T>,
}

impl<T> Default for StochasticSearch<T> {
    fn default() -> Self {
        Self {
            a: vec![],
            b: vec![]
        }
    }
}

struct ExhaustiveSearch<T> {
    current: Vec<T>,
}

impl<T> Default for ExhaustiveSearch<T> {
    fn default() -> Self {
        Self {
            current: vec![]
        }
    }
}
