"""The memory cap on each mutant's tests (R-88, issue #40). A rule file.

    engine/tools/test-memcap CAP_MB <test binary> [args…]   (nextest's `mutants` profile
                                                             runs every test through it)
    engine/tools/test-memcap --self-test                    a planted runaway is ended

It runs one test process and watches its memory every 50 ms: the resident set and swap on
Linux (/proc), the physical footprint on macOS (proc_pid_rusage, what top's MEM column
shows). When it passes CAP_MB megabytes the test is killed, one line on stderr says so,
and the wrapper exits 86, so nextest fails the test and cargo-mutants ends the mutant
there, instead of a runaway taking the machine down (a GitHub runner, 16 GB, is shut down
before cargo-mutants' --timeout fires). It caps the test process only: cargo and rustc
run outside it. Every other exit passes through unchanged, a signal as 128 + its number.
"""

import ctypes
import os
import signal
import subprocess
import sys
import time

CAPPED = 86  # the exit that says "ended by the memory cap"; tools/mutants-survivors reads it
LINE = "test-memcap: ended at {mb} MB, past the {cap} MB cap"
POLL_S = 0.05


def linux_mb(pid):
    kb = 0
    with open(f"/proc/{pid}/status", encoding="ascii") as f:
        for line in f:
            if line.startswith(("VmRSS:", "VmSwap:")):
                kb += int(line.split()[1])
    return kb // 1024


class RusageInfoV2(ctypes.Structure):
    _fields_ = [("uuid", ctypes.c_uint8 * 16)] + [(n, ctypes.c_uint64) for n in (
        "user_time", "system_time", "pkg_idle_wkups", "interrupt_wkups", "pageins", "wired_size",
        "resident_size", "phys_footprint", "start_abstime", "exit_abstime", "child_user_time",
        "child_system_time", "child_pkg_idle_wkups", "child_interrupt_wkups", "child_pageins",
        "child_elapsed_abstime", "diskio_bytesread", "diskio_byteswritten")]


def darwin_reader():
    lib = ctypes.CDLL("/usr/lib/libproc.dylib", use_errno=True)

    def mb(pid):
        info = RusageInfoV2()
        if lib.proc_pid_rusage(pid, 2, ctypes.byref(info)) != 0:  # RUSAGE_INFO_V2
            raise OSError(ctypes.get_errno(), "proc_pid_rusage")
        return info.phys_footprint // (1024 * 1024)

    return mb


def reader():
    if sys.platform.startswith("linux"):
        return linux_mb
    if sys.platform == "darwin":
        return darwin_reader()
    raise SystemExit(f"test-memcap: no memory reader for {sys.platform}")


def run(cap, argv):
    mb_of = reader()
    child = subprocess.Popen(argv)
    # nextest ends a test's process group on a timeout or a cancel: pass it on
    for sig in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(sig, lambda s, _f: child.send_signal(s))
    while child.poll() is None:
        try:
            mb = mb_of(child.pid)
        except OSError:
            mb = 0  # it has just exited; poll() says so next
        if mb > cap:
            child.kill()
            child.wait()
            print(LINE.format(mb=mb, cap=cap), file=sys.stderr, flush=True)
            return CAPPED
        time.sleep(POLL_S)
    code = child.returncode
    return 128 - code if code < 0 else code


def self_test():
    """A planted runaway (it grows by 64 MB every 10 ms, for ever) is ended at a 256 MB cap
    with the line and exit 86; a test that stays under it keeps its own exit."""
    grow = "b=[]\nwhile True:\n    b.append(bytearray(64 << 20))\n    __import__('time').sleep(0.01)\n"
    py = [sys.executable, "-I", "-B", "-c"]
    problems = []
    began = time.monotonic()
    code = run(256, [*py, grow])
    if code != CAPPED:
        problems.append(f"the runaway ended with {code}, not {CAPPED}")
    if time.monotonic() - began > 30:
        problems.append("the runaway took over 30 s to end")
    if run(256, [*py, "raise SystemExit(3)"]) != 3:
        problems.append("a test's own exit code was changed")
    if run(256, [*py, "import os, signal; os.kill(os.getpid(), signal.SIGKILL)"]) != 128 + 9:
        problems.append("a test killed by a signal didn't exit 128 + its number")
    print(f"test-memcap self-test {'PASSED' if not problems else 'FAILED: ' + '; '.join(problems)}")
    return 0 if not problems else 1


def main():
    args = sys.argv[1:]
    if args == ["--self-test"]:
        return self_test()
    if len(args) < 2 or not args[0].isdigit():
        print("usage: engine/tools/test-memcap CAP_MB <test binary> [args…] | --self-test", file=sys.stderr)
        return 64
    return run(int(args[0]), args[1:])


if __name__ == "__main__":
    sys.exit(main())
