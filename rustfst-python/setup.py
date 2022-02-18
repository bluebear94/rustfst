import os
import sys
from pathlib import Path

from setuptools import setup, find_packages
from setuptools_rust import Binding, RustExtension

packages = [p for p in find_packages() if "tests" not in p]

PACKAGE_NAME = "rustfst"
RUST_EXTENSION_NAME = "rustfst.dylib"
REPO_ROOT_PATH = Path(__file__).resolve().parents[1]
CARGO_ROOT_PATH = REPO_ROOT_PATH / "rustfst-ffi"
CARGO_FILE_PATH = CARGO_ROOT_PATH / "Cargo.toml"
CARGO_TARGET_DIR = REPO_ROOT_PATH / "target"
os.environ["CARGO_TARGET_DIR"] = str(CARGO_TARGET_DIR)

if "PROFILE" in os.environ:
    if os.environ.get("PROFILE") == "release":
        is_debug_profile = False
    elif os.environ.get("PROFILE") == "debug":
        is_debug_profile = True
    else:
        print("Invalid PROFILE %s" % os.environ.get("PROFILE"))
        sys.exit(1)
else:
    is_debug_profile = "develop" in sys.argv

setup(
    name=PACKAGE_NAME,
    version="0.1.0",
    description="Python wrapper for Rust FST",
    extras_require={"tests": ["pytest>=6,<7"]},
    options={"bdist_wheel": {"universal": True}},
    packages=packages,
    include_package_data=True,
    rust_extensions=[
        RustExtension(
            RUST_EXTENSION_NAME,
            str(CARGO_FILE_PATH),
            debug=is_debug_profile,
            binding=Binding.NoBinding,
        )
    ],
    zip_safe=False,
)
