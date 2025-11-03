use strop::RunResult;

fn add5(i: u8) -> RunResult<u8> {
    i.checked_add(5).ok_or(strop::RunError::NotDefined)
}

fn help_list_triplets() -> ! {
    println!("Try one of these:");
    for triplet in strop::Triplet::all() {
        println!(" - {triplet}");
    }
    std::process::exit(1);
}

fn main() {
    let mut args = std::env::args();
    let _ = args.next().unwrap();

    // First figure out which target we're generating code for
    let Some(target) = args.next() else {
        println!("Specify the target.");
        help_list_triplets();
    };

    let Some(target) = strop::Triplet::search(&target) else {
        println!("Unknown target {target}.");
        help_list_triplets();
    };

    let mut searcher = target.pure_leaf_function_search(add5 as fn(u8) -> RunResult<u8>);

    loop {
        searcher.increment();
        if searcher.pass() {
            println!("solution:");
            println!("{searcher}");
            break;
        } else {
            println!("next_try:");
            println!("{searcher:?}");
        }
    }
}
