"""The cargo-mutex hook's matcher verification table — the fourteen cases two
commit messages cite as evidence (2026-08-02). Run directly; exits loudly on
mismatch. Rescued to the repo at wrap: cited evidence may not live in temp.
"""

import importlib.util

spec = importlib.util.spec_from_file_location(
    "hook", r"B:\repos\borwood\deepcraft\scripts\cargo_mutex_hook.py"
)
hook = importlib.util.module_from_spec(spec)
spec.loader.exec_module(hook)
pat = hook.CARGO_RE

cases = {
    "cargo build": True,
    '& "$env:USERPROFILE\\.cargo\\bin\\cargo.exe" check 2>&1': True,
    "cd x && cargo test": True,
    "cargo --version": True,
    "cargo +nightly fmt": True,
    'grep -n "cargo test --workspace" CLAUDE.md': False,
    'echo "run cargo build later"': False,
    'git commit -m "the cargo hook"': False,
    "foo; cargo run": True,
    "~/.cargo/bin/cargo fmt --all --check": True,
    'Set-Content lock "x"; & "cargo.exe" test': True,
    "git add scripts/cargo_mutex_hook.py && git commit": False,
    'msg line one\n"cargo" while guarding a build': False,
    "prose mentioning\ncargo whose matcher": False,
}
bad = 0
for c, want in cases.items():
    got = bool(pat.search(c))
    if got != want:
        bad += 1
        print("MISMATCH", repr(c), "->", got, "want", want)
print("mismatches:", bad, "of", len(cases))

import sys
sys.exit(1 if bad else 0)
