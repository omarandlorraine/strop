use crate::triplets::Triplet;
use crate::Callable;
use std::marker::PhantomData;

pub trait Searchable<Input, Output> {
    fn step(&mut self);
    fn next(&mut self);
    fn tests_pass(&self) -> bool;
    fn dasm(&self);
}

#[derive(Debug, Default)]
struct PureFunction<Input, Output> {
    i:PhantomData<Input>,
    o:PhantomData<Output>,
}

impl<Input, Output> PureFunction<Input, Output> {
    pub fn leaf<C: Callable<Input, Output>>(triplet: Triplet, c: &C) -> Box<dyn Searchable<Input, Output>> {
        todo!()
    }
}

