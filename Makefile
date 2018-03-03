export RUST_LOG=state=TRACE
export RUST_BACKTRACE=1
test:
	cargo test

ci:release help_t testacc_t wallet_t

release:
	cargo build --all-targets --release

bench:
	cargo +nightly bench --features="unstable"

cov:
	docker run -it --rm --security-opt seccomp=unconfined --volume "$$PWD:/volume" elmtai/docker-rust-kcov

fmt:
	cargo fmt -- --write-mode=diff

wfmt:
	cargo fmt -- --write-mode=overwrite

help_t:release
	./target/release/loomd -h
	./target/release/loom -h

wallet_t:release
	echo foobar | ./target/release/loom -l -W testdata/loom.wallet


testacc_t:release
	./target/release/loomd -t ./testdata/test_accounts.json
