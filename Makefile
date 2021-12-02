.DEFAULT_GOAL: all

all:: fmt lint build

build::
	cargo build --all-targets

doc::
	cargo doc

fmt::
	cargo +nightly fmt --all

lint::
	cargo +nightly clippy --all-features --all-targets

rr::
	cargo run --release

test::
	cargo test
