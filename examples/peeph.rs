use strop::RunResult;

fn identity(i: u32) -> RunResult<u32> {
    Ok(i)
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

    let mut verbose = false;

    // First figure out which target we're generating code for
    let Some(target) = args.next() else {
        println!("Specify the target.");
        help_list_triplets();
    };

    let Some(target) = strop::Triplet::search(&target) else {
        println!("Unknown target {target}.");
        help_list_triplets();
    };

    for r in args {
        if r == "-v" {
            verbose = true;
        } else {
            eprintln!("Unknown command line option \"{r}\"");
        }
    }

    let mut searcher = target.pure_leaf_function_search(identity as fn(u32) -> RunResult<u32>);

    let mut number = 0usize;
    loop {
        searcher.increment();
        if searcher.pass() {
            number += 1;
            println!("solution_{number}:");
            println!("{searcher:?}");
        } else if verbose {
            println!("not_a_solution:");
            println!("{searcher:?}");
        }
    }
}
