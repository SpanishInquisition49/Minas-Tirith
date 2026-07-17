set dotenv-load

run:
  cargo run

build:
  cargo build --release

install:
  cargo install --path .

check:
  cargo clippy -- -A clippy::pedantic

clean:
  rm ~/.local/share/minastirith/minastirith.db
