run:
  cargo run -- --file./examples/01-factorial.txt 5

build:
  cargo build --release

install:
  cargo install --path .

check:
  cargo clippy -- -A clippy::pedantic
