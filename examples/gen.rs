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

    let mut verbose = false;
    let mut keep_going = false;

    // First figure out which target we're generating code for
    let Some(target) = args.next() else {
        println!("Specify the target.");
        help_list_triplets();
    };

    let Some(target) = strop::Triplet::search(&target) else {
        println!("Unknown target {target}.");
        help_list_triplets();
    };

    while let Some(switch) = args.next() {
        if switch == "-v" {
            verbose = true;
        } else if switch == "-K" {
            keep_going = true;
        } else {
            println!("Unknown switch {switch:?}.");
            std::process::exit(1);
        }
    }

    let mut searcher = target.pure_leaf_function_search(add5 as fn(u8) -> RunResult<u8>);

    loop {
        searcher.increment();
        if searcher.pass() {
            println!("solution:");
            println!("{searcher:?}");
            if !keep_going {
                break;
            }
        } else if verbose{
            println!("next_try:");
            println!("{searcher:?}");
        }
    }
}
