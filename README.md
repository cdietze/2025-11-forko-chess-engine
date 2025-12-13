# Forko

Forko is a chess engine written in Rust.

## Develop

Install the pre-commit hooks using:

    ln -sf ../../scripts/hooks/pre-commit .git/hooks/pre-commit

Run tests:

    cargo test

Run tests including longer running perft tests:

    cargo test --release

Build the WebAssembly package for the web target:

    wasm-pack build --target web

## Links

- https://www.chessprogramming.org
