import importlib.util
import io
import stat
import tarfile
import tempfile
from pathlib import Path
from unittest import TestCase


REPO_ROOT = Path(__file__).resolve().parents[1]
BOOTSTRAP_PATH = REPO_ROOT / "bootstrap_rustc_session.py"


def load_bootstrap_module():
    spec = importlib.util.spec_from_file_location("bootstrap_rustc_session", BOOTSTRAP_PATH)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


def add_tar_file(tf: tarfile.TarFile, name: str, body: bytes, mode: int = 0o755) -> None:
    info = tarfile.TarInfo(name)
    info.size = len(body)
    info.mode = mode
    tf.addfile(info, io.BytesIO(body))


class BootstrapRustcSessionContract(TestCase):
    def test_have_toolchain_rejects_broken_executables(self):
        bootstrap = load_bootstrap_module()
        with tempfile.TemporaryDirectory() as tmp:
            prefix = Path(tmp) / "prefix"
            bin_dir = prefix / "bin"
            bin_dir.mkdir(parents=True)
            for tool in ["rustc", "cargo"]:
                path = bin_dir / tool
                path.write_text("#!/usr/bin/env sh\nexit 42\n", encoding="utf-8")
                path.chmod(path.stat().st_mode | stat.S_IXUSR)

            self.assertFalse(bootstrap.have_toolchain(prefix))

    def test_failed_component_extract_preserves_existing_prefix(self):
        bootstrap = load_bootstrap_module()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            prefix = root / "prefix"
            marker = prefix / "bin" / "rustc"
            marker.parent.mkdir(parents=True)
            marker.write_text("existing-prefix\n", encoding="utf-8")

            archive = root / "incomplete-rust.tar.gz"
            with tarfile.open(archive, "w:gz") as tf:
                add_tar_file(tf, "rust-test/rustc/bin/rustc", b"#!/usr/bin/env sh\n")

            with self.assertRaises(RuntimeError):
                bootstrap.extract_components_with_python_tarfile(archive, root / "extract-tmp", prefix)

            self.assertEqual(marker.read_text(encoding="utf-8"), "existing-prefix\n")

    def test_successful_component_extract_replaces_prefix_atomically(self):
        bootstrap = load_bootstrap_module()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            prefix = root / "prefix"
            old = prefix / "bin" / "rustc"
            old.parent.mkdir(parents=True)
            old.write_text("old\n", encoding="utf-8")

            archive = root / "complete-rust.tar.gz"
            with tarfile.open(archive, "w:gz") as tf:
                for component in [
                    "rustc/bin/rustc",
                    "cargo/bin/cargo",
                    "rustc/lib/librustc_driver.so",
                    "rust-std-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/lib/libstd.rlib",
                ]:
                    add_tar_file(tf, f"rust-test/{component}", component.encode("utf-8"))

            bootstrap.extract_components_with_python_tarfile(archive, root / "extract-tmp", prefix)

            self.assertEqual((prefix / "bin" / "rustc").read_bytes(), b"rustc/bin/rustc")
            self.assertEqual((prefix / "bin" / "cargo").read_bytes(), b"cargo/bin/cargo")
            self.assertFalse((root / "extract-tmp" / "prefix-stage").exists())


if __name__ == "__main__":
    import unittest

    unittest.main()