# justfile
set dotenv-load

windows_target := "x86_64-pc-windows-msvc"

run:
  cargo run

build:
  cargo build --release

build-linux:
  cargo build --release

# Build for Windows by cross-compile with cargo-xwin (da Linux/macOS).
# Requires: rustup target add x86_64-pc-windows-msvc
#           cargo install cargo-xwin
#           nasm + cmake installed on the host (needed by aws-lc-sys)
build-windows:
  cargo xwin build --release --target {{windows_target}}

install:
  cargo install --path .

check:
  cargo clippy -- -A clippy::pedantic

[unix]
clean:
  rm ~/.local/share/minastirith/minastirith.db

[windows]
clean:
  powershell -Command "Remove-Item $env:LOCALAPPDATA\minastirith\data\minastirith.db"
