# this_file: tests/test_packaging.py
"""Cargo include rules must exclude ignored local data in real checkouts."""

import shutil
import subprocess
import tarfile
from email.parser import BytesParser
from pathlib import Path


def test_maturin_sdist_contains_declared_root_license(tmp_path: Path) -> None:
    root = Path(__file__).resolve().parents[1]
    for name in ("Cargo.toml", "Cargo.lock", "pyproject.toml", "README.md", "LICENSE"):
        shutil.copy2(root / name, tmp_path)
    shutil.copytree(
        root / "crates",
        tmp_path / "crates",
        ignore=shutil.ignore_patterns("target", ".DS_Store", "__pycache__"),
    )
    subprocess.run(
        ["uvx", "maturin", "sdist", "--out", str(tmp_path / "dist")],
        cwd=tmp_path,
        check=True,
        capture_output=True,
        text=True,
    )
    sdist = next((tmp_path / "dist").glob("*.tar.gz"))
    with tarfile.open(sdist) as archive:
        metadata_path = next(n for n in archive.getnames() if n.endswith("/PKG-INFO"))
        metadata = BytesParser().parsebytes(archive.extractfile(metadata_path).read())
        licenses = metadata.get_all("License-File", [])
        assert "LICENSE" in licenses, "The sdist must declare its MIT license"
        for license_path in licenses:
            member = f"{metadata_path.rsplit('/', 1)[0]}/{license_path}"
            assert member in archive.getnames(), (
                f"Declared license is missing: {member}"
            )
            assert (
                archive.extractfile(member).read() == (root / license_path).read_bytes()
            )


def test_cargo_packages_exclude_local_data_from_source_directories(
    tmp_path: Path,
) -> None:
    root = Path(__file__).resolve().parents[1]
    shutil.copy2(root / "Cargo.toml", tmp_path)
    shutil.copy2(root / "Cargo.lock", tmp_path)
    shutil.copytree(
        root / "crates",
        tmp_path / "crates",
        ignore=shutil.ignore_patterns("target", ".DS_Store", "__pycache__"),
    )
    for crate in ("uubed-core", "uubed-tm", "uubed-python"):
        for folder in ("src", "tests", "benches", "examples"):
            directory = tmp_path / "crates" / crate / folder
            directory.mkdir(exist_ok=True)
            for name in (".DS_Store", "private.tmx", "memory.sqlite"):
                (directory / name).write_text("private packaging canary")
        output = subprocess.check_output(
            ["cargo", "package", "--list", "--allow-dirty", "--offline", "-p", crate],
            cwd=tmp_path,
            text=True,
        )
        assert "src/lib.rs" in output, f"Rust source missing from {crate}"
        for name in (".DS_Store", "private.tmx", "memory.sqlite"):
            assert name not in output, f"{crate} unexpectedly packages {name}"
