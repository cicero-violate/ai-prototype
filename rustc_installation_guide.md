# Rustc Installation Guide for This System

## Variables

- `A=/mnt/data/rust-1.75.0-x86_64-unknown-linux-gnu.tar.gz`
- `X=/mnt/data/rustc_python_install_test`
- `P=/mnt/data/rustc-python-install-prefix`
- `C=/mnt/data/rustc-python-cargo-home`
- `T=/mnt/data/rust-python-test-project`
- `K={rustc,cargo,rust-std-x86_64-unknown-linux-gnu}`

## Equation

```text
working_toolchain = python_tarfile_extract(A, X, K) + merge(K, P)
validation = cargo_run(T, PATH=P/bin:$PATH, CARGO_HOME=C, --offline)
good = max(working_rustc, working_cargo, reproducible_steps)
```

One-line explanation: extract only the needed Rust components using Python's standard `tarfile` module, merge them into one prefix, then run Cargo offline against a minimal project.

## Exact Internal Tools Used for This Update

- `file_search`: read the existing `/mnt/data/rustc_installation_guide.md` content that was available in the conversation.
- `container.exec`: listed `/mnt/data` inventory and performed the final root-owned file write.
- `python.exec`: extracted the Rust archive using Python `tarfile`, copied components, created the test Cargo project, and ran validation commands with `subprocess.run`.
- `python.exec` could not overwrite `/mnt/data/rustc_installation_guide.md` because the uploaded file was owned by `root`; final write used `container.exec` running Python.
- Shell `tar` was not used for the verified update procedure.

## Working Procedure

Run this Python script inside the environment:

```python
import os
import shutil
import subprocess
import tarfile
import textwrap
from pathlib import Path

A = Path("/mnt/data/rust-1.75.0-x86_64-unknown-linux-gnu.tar.gz")
X = Path("/mnt/data/rustc_python_install_test")
P = Path("/mnt/data/rustc-python-install-prefix")
C = Path("/mnt/data/rustc-python-cargo-home")
T = Path("/mnt/data/rust-python-test-project")

for p in [X, P, C, T]:
    if p.exists():
        shutil.rmtree(p)

(P / "bin").mkdir(parents=True)
(P / "lib").mkdir(parents=True)
C.mkdir(parents=True)
X.mkdir(parents=True)

wanted_components = {
    "rustc",
    "cargo",
    "rust-std-x86_64-unknown-linux-gnu",
}

def is_safe_member(member_name: str, destination: Path) -> bool:
    target = (destination / member_name).resolve()
    return str(target).startswith(str(destination.resolve()) + os.sep)

root_name = None

with tarfile.open(A, "r:gz") as tf:
    for member in tf:
        parts = member.name.split("/")
        if root_name is None and parts:
            root_name = parts[0]

        if len(parts) >= 2 and parts[1] in wanted_components:
            if not is_safe_member(member.name, X):
                raise RuntimeError(f"unsafe tar member: {member.name}")
            tf.extract(member, X)

root = X / root_name

def copy_tree_contents(src: Path, dst: Path) -> None:
    if not src.exists():
        raise RuntimeError(f"missing component path: {src}")

    for item in src.iterdir():
        target = dst / item.name
        if item.is_dir():
            shutil.copytree(item, target, dirs_exist_ok=True)
        else:
            shutil.copy2(item, target)

copy_tree_contents(root / "rustc" / "bin", P / "bin")
copy_tree_contents(root / "cargo" / "bin", P / "bin")
copy_tree_contents(root / "rustc" / "lib", P / "lib")
copy_tree_contents(root / "rust-std-x86_64-unknown-linux-gnu" / "lib", P / "lib")

(T / "src").mkdir(parents=True)

(T / "Cargo.toml").write_text(textwrap.dedent("""\
    [package]
    name = "rust_install_probe_python_tarfile"
    version = "0.1.0"
    edition = "2021"

    [dependencies]
    """), encoding="utf-8")

(T / "src" / "main.rs").write_text(textwrap.dedent("""\
    fn main() {
        println!("rustc cargo python-tarfile probe ok: {}", add(2, 3));
    }

    fn add(a: i32, b: i32) -> i32 {
        a + b
    }
    """), encoding="utf-8")

env = os.environ.copy()
env["PATH"] = f"{P / 'bin'}:{env.get('PATH', '')}"
env["CARGO_HOME"] = str(C)

commands = [
    ["rustc", "--version"],
    ["rustc", "--print", "sysroot"],
    ["rustc", "--print", "target-libdir"],
    ["cargo", "--version"],
    ["cargo", "run", "--offline"],
]

for cmd in commands:
    proc = subprocess.run(
        cmd,
        cwd=str(T) if cmd[0] == "cargo" else None,
        env=env,
        text=True,
        capture_output=True,
        timeout=120,
    )
    print("$", " ".join(cmd))
    if proc.stdout:
        print(proc.stdout, end="")
    if proc.stderr:
        print(proc.stderr, end="")
    if proc.returncode != 0:
        raise SystemExit(proc.returncode)
```

## Verified Output

```text
$ rustc --version
rustc 1.75.0 (82e1608df 2023-12-21)

$ rustc --print sysroot
/mnt/data/rustc-python-install-prefix

$ rustc --print target-libdir
/mnt/data/rustc-python-install-prefix/lib/rustlib/x86_64-unknown-linux-gnu/lib

$ cargo --version
cargo 1.75.0 (1d8b05cdd 2023-11-20)

$ cargo run --offline
   Compiling rust_install_probe_python_tarfile v0.1.0 (/mnt/data/rust-python-test-project)
    Finished dev [unoptimized + debuginfo] target(s) in 1.63s
     Running `target/debug/rust_install_probe_python_tarfile`
rustc cargo python-tarfile probe ok: 5
```

## Notes

- This is the corrected working method for this system.
- Use Python `tarfile`, not shell `tar`, when extracting Rust archives for these repo tasks.
- Required archive components are `rustc`, `cargo`, and `rust-std-x86_64-unknown-linux-gnu`.
- `cargo run --offline` works for a dependency-free project because no registry access is required.
- Use `PATH=/mnt/data/rustc-python-install-prefix/bin:$PATH` for commands that should use this extracted toolchain.
- Use `CARGO_HOME=/mnt/data/rustc-python-cargo-home` to isolate Cargo state from other toolchains.
- The old manual shell extraction method is superseded by this Python-verified procedure.
