#[derive(Copy, Clone)]
pub enum OptimisationStrategy{
	BySize,
	BySpeed,
}

pub fn optimise_for(x: OptimisationStrategy) {
	let _strategy = &x;
	
	match x {
		OptimisationStrategy::BySpeed => println!("Optimising for Speed"),
		OptimisationStrategy::BySize  => println!("Optimising for size"),
	}
}
