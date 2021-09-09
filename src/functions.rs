pub fn id(input: Vec<i8>) -> Vec<i8> {
	input
}

pub fn parity(input: Vec<i8>) -> Vec<i8> {
	vec![(input[0].count_ones() % 2) as i8]
}

pub fn popcount(input: Vec<i8>) -> Vec<i8> {
	vec![input[0].count_ones() as i8]
}
