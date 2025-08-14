# strop
[![Build Status](https://github.com/omarandlorraine/strop/workflows/Rust/badge.svg)](https://github.com/omarandlorraine/strop/actions?workflow=Rust)
[![crates.io](https://img.shields.io/crates/v/strop)](https://crates.io/crates/strop)

strop, the *st*ochastic *op*timizer, written in *R*ust.

Like a compiler, strop generates assembly that computes a given function. But
unlike a compiler, it generates assembly-language subroutines by a random
search or a brute-force search.

### Core components

* **Input function**: This is a pure function which strop can execute.
* **Backends**: A back-end targets a specific machine architecture. It usually
  includes a symbolic representation of the machine instruction, and ways to
  execute these in emulation.
* **Search space**: The search space encompasses all sequences of instructions
  that could compute the given function. This search space is vast, so *strop*
  includes ways of pruning it down considerably.
* **Objective function**: A function for evaluating the quality of a candidate
  program. It could be a combination of correctness metrics, or efficiency
  metrics, such as code size, or number of branch mispredictions.
* **Validation mechanism**: A fuzz tester makes sure that the generated
  function matches the input function.
* **Search algorithms**: The core of *strop* is the search algorithms. These
  explore the search space, trying to minimize the objective function.

### Operation workflow

1. Usually at compilation time, a back-end is selected, and a function is
   selected. The function may be one that is compiled and linked into the
   executable, and callable from it (i.e., an ordinary function that runs as
   part of the executable's normal operation), or it may be a function that is
   executable in some back-end.
2. A new program is generated. Usually, this is an empty program, one that does
   nothing.
3. The search algorithm searches the space, each time yielding a putative
   candidate program.
4. The validation mechanism checks that the putative candidate program is
   well-formed, and feeds information about this back to the search algorithm.
   For example, a malformed subroutine may be one which lacks a return
   instruction, or one which reads a variable before it has been initialized.
5. Any putative candidate program that passes the validation mechanism is run
   against randomly generated input parameters, and so is the input function.
   Information about how well these functions match are passed to the search
   algorithm, so that it may be guided to the correct solution(s). Any input
   parameters that have been found to yield erroneous results are remembered,
   and tried again on the next candidate program under test.

### Search strategies

Strop includes two main search strategies:

* **Simulated Annealing**: This method involves exploring the search space by
  accepting both improving and, occasionally, worsening candidates, with the
  likelihood of accepting worse solutions decreasing over time. This mimics the
  physical process of annealing, helping avoid local optima.

* **Bruteforce search**: This method tries all possible sequences, in some
predetermined order. First, it tries all sequences of length 1, then all
sequences of length 2, and so forth. To somewhat mitigate the time-consuming
nature of bruteforce searches, after each step through the search space, strop
will perform static analysis on the instruction sequence, and in so doing,
eliminate candidates which, for example, read from a register known to be
uninitialized, can be effectively eliminated from the search space.

### Challenges and considerations

Scalability: The vastness of the search space poses a significant challenge.
There are more ten-line x86 programs than there are atoms in the universe.
Efficient search algorithms, and good heuristics are essential to the proper
operation of strop.

Validation costs: Ensuring the correctness of candidate programs can be
computationally expensive, so strop includes some basic tests only, which are a
fuzz-tester and is static analysis.

Convergence: Stochastic methods do not guarantee finding the global optimum
(that is, the most optimal program), nor do they guarantee finding a solution
at all. In such a case, strop will keep going, until a human (or operating
system, or another mechanism) terminates the program.

### Conclusion

A stochastic superoptimizer offers a powerful but computationally expensive way
to approach code generation and optimization. It can be made to explore
unconventional solutions and achieve optimizations beyond the reach of
traditional techniques, as currently used by compiler back-ends.
