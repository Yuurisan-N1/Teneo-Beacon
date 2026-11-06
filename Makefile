BIN := target/release/teneo-bot

.PHONY: build run probe selftest fmt clean dist

build:
	cargo build --release

run: build
	./$(BIN) run

probe: build
	./$(BIN) probe

selftest: build
	./$(BIN) selftest

fmt:
	cargo fmt

clean:
	cargo clean

dist: build
	mkdir -p dist
	cp $(BIN) dist/
