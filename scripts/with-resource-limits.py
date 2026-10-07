#!/usr/bin/env python3
"""Run a development command at low priority and on at most two Linux CPUs."""
import os
import sys

if len(sys.argv) < 2:
    sys.exit('Usage: python3 scripts/with-resource-limits.py COMMAND [ARGS...]')
if hasattr(os, 'sched_getaffinity'):
    available = sorted(os.sched_getaffinity(0))
    os.sched_setaffinity(0, set(available[-2:]))
if hasattr(os, 'nice'):
    os.nice(15)
os.environ.setdefault('CARGO_BUILD_JOBS', '2')
os.environ.setdefault('RAYON_NUM_THREADS', '2')
os.environ.setdefault('UV_THREADPOOL_SIZE', '2')
os.execvp(sys.argv[1], sys.argv[1:])
