#!/usr/bin/env -S just --justfile

export RUST_BACKTRACE := "1"

[group: 'CI/CD']
ci-stable:
	# There is a github workflow which performs miscellaneous checks in the
	# event of a push to a feature-branch.
	# This ought to perform the same checks, but locally.
	cargo fmt --check
	cargo clippy -- -Dwarnings
	cargo test

[group: 'CI/CD']
ci-nightly:
	cargo +nightly test --features m68k --no-default-features --lib
	cargo +nightly clippy --features m68k --no-default-features -- -Dwarnings

[group: 'CI/CD']
ci: ci-stable ci-nightly

[group: 'Long-running tests']
mips_no_duplicates:
	# Finds two MIPS instructions that disassemble the same. This would
	# presumably be because of don't-cares in the instruction decoding; for
	# example the `add` instruction ignores inthe `shamt` field.
	# Ideally the bruteforce search should skip these duplicates so if this
	# test finds something, then there's room for improvement here.
	cargo test --release -- mips::subroutine::test::no_duplicates --nocapture --include-ignored


[group: 'Long-running tests']
mips_all_two_instruction_subroutines:
	# Finds the first MIPS instruction which crashes the emulator, if there
	# is such an instruction
	cargo test --release -- mips::subroutine::test::all_two_instruction_subroutines --nocapture --include-ignored | tail -n 50

[group: 'Long-running tests']
mips_can_iterate_over_all_instructions:
	cargo test --release -- mips::isa::test::can_iterate_over_all_instructions
