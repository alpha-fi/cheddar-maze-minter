res:
	mkdir res
abi:
	mkdir abi

build: res
	@RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
	@cp target/wasm32-unknown-unknown/release/*.wasm res/

build-debug: res
	@RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown
	@cp target/wasm32-unknown-unknown/debug/*.wasm res/

build-abi: abi
	@cargo near abi
	@cp target/near/*_abi.json abi/

lint:
	cargo clippy  -- --no-deps

lint-fix:
	cargo clippy --fix  -- --no-deps

test-unit:
	@cargo test --lib

test-unit-debug:
	@RUST_BACKTRACE=1 cargo test --lib
