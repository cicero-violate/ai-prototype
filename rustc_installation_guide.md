# Rustc Installation Guide for This System

## Variables

- `A=/mnt/data/rust-1.75.0-x86_64-unknown-linux-gnu.tar.gz`
- `X=/mnt/data/rustc_install_test`
- `P=/mnt/data/rustc-install-prefix`
- `C=/mnt/data/rustc-cargo-home`
- `T=/mnt/data/rust-test-project`

## Equation

```text
working_toolchain = extract(A, X) + merge(rustc, cargo, rust-std, P)
validation = cargo_run(T, PATH=P/bin:$PATH, CARGO_HOME=C, --offline)
good = max(working_rustc, working_cargo, reproducible_steps)
```

One-line explanation: extract the official Rust archive, merge only the required components into one prefix, then run Cargo offline against a minimal project.

## Working Procedure

```bash
set -euo pipefail

rm -rf /mnt/data/rustc_install_test \
       /mnt/data/rustc-install-prefix \
       /mnt/data/rustc-cargo-home \
       /mnt/data/rust-test-project

mkdir -p /mnt/data/rustc_install_test \
         /mnt/data/rustc-install-prefix/bin \
         /mnt/data/rustc-install-prefix/lib \
         /mnt/data/rustc-cargo-home

cd /mnt/data/rustc_install_test
tar -xzf /mnt/data/rust-1.75.0-x86_64-unknown-linux-gnu.tar.gz

cd /mnt/data/rustc_install_test/rust-1.75.0-x86_64-unknown-linux-gnu

cp -a rustc/bin/. /mnt/data/rustc-install-prefix/bin/
cp -a cargo/bin/. /mnt/data/rustc-install-prefix/bin/
cp -a rustc/lib/. /mnt/data/rustc-install-prefix/lib/
cp -a rust-std-x86_64-unknown-linux-gnu/lib/. /mnt/data/rustc-install-prefix/lib/

/mnt/data/rustc-install-prefix/bin/rustc --version
/mnt/data/rustc-install-prefix/bin/rustc --print sysroot
/mnt/data/rustc-install-prefix/bin/rustc --print target-libdir
/mnt/data/rustc-install-prefix/bin/cargo --version
```

## Test Project

```bash
mkdir -p /mnt/data/rust-test-project/src
cd /mnt/data/rust-test-project

cat > Cargo.toml <<'EOF'
[package]
name = "rust_install_probe"
version = "0.1.0"
edition = "2021"

[dependencies]
EOF

cat > src/main.rs <<'EOF'
fn main() {
    println!("rustc cargo probe ok: {}", add(2, 3));
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
EOF

PATH=/mnt/data/rustc-install-prefix/bin:$PATH \
CARGO_HOME=/mnt/data/rustc-cargo-home \
cargo run --offline
```

## Verified Output

```text
rustc 1.75.0 (82e1608df 2023-12-21)
/mnt/data/rustc-install-prefix
/mnt/data/rustc-install-prefix/lib/rustlib/x86_64-unknown-linux-gnu/lib
cargo 1.75.0 (1d8b05cdd 2023-11-20)
   Compiling rust_install_probe v0.1.0 (/mnt/data/rust-test-project)
    Finished dev [unoptimized + debuginfo] target(s) in 6.59s
     Running `target/debug/rust_install_probe`
rustc cargo probe ok: 5
```

## Notes

- The working method is a manual component merge into `/mnt/data/rustc-install-prefix`.
- Required components are `rustc`, `cargo`, and `rust-std-x86_64-unknown-linux-gnu`.
- `cargo run --offline` works for a dependency-free project because no registry access is required.
- Keep `PATH=/mnt/data/rustc-install-prefix/bin:$PATH` for commands that should use this extracted toolchain.
- Keep `CARGO_HOME=/mnt/data/rustc-cargo-home` to isolate Cargo state from other toolchains.