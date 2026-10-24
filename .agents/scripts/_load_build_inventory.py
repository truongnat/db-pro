"""Load build-knowledge-inventory.py (hyphenated module name)."""

from __future__ import annotations

import importlib.util
from pathlib import Path
from types import ModuleType


def load() -> ModuleType:
    path = Path(__file__).resolve().parent / "build-knowledge-inventory.py"
    spec = importlib.util.spec_from_file_location("build_knowledge_inventory", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod
