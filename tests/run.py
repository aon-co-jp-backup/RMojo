# -*- coding: utf-8 -*-
"""Differential test: run each tests/*.py with CPython and with `rmojo run`, compare stdout. Usage: python tests/run.py <rmojo binary>"""
import glob
import subprocess
import sys

exe = sys.argv[1]
bad = 0
files = sorted(glob.glob("tests/*.py"))
files.remove("tests/run.py") if "tests/run.py" in files else None
for f in files:
    src = open(f, encoding="utf-8").read()
    # CPython has no implicit main(): append a call like Mojo's entry point
    py = subprocess.run([sys.executable, "-c", src + "\nmain() if 'main' in globals() else None"], capture_output=True, text=True)
    mj = subprocess.run([exe, "run", f], capture_output=True, text=True)
    ok = py.stdout == mj.stdout and py.returncode == mj.returncode
    print(("OK   " if ok else "DIFF ") + f)
    if not ok:
        bad += 1
        print("--- cpython\n" + py.stdout + py.stderr + "--- rmojo\n" + mj.stdout + mj.stderr)
print("differences=%d" % bad)
sys.exit(1 if bad else 0)
