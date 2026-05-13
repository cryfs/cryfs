#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.9"
# dependencies = [
#   # Range pin: rich's Layout/Live/Text.from_ansi APIs are stable across
#   # 13.x and 14.x. A future 15.x might break them — re-test before bumping.
#   "rich>=13,<15",
# ]
# ///
"""
Run `cargo test` repeatedly to detect flaky tests.

This script uses `uv` (https://docs.astral.sh/uv/) to manage its own
dependencies — the shebang invokes `uv run --script`, which reads the
PEP 723 metadata block above, installs `rich` into an isolated cached
environment on first run, and then executes the script. No manual
`pip install` needed. Install uv once with:

    curl -LsSf https://astral.sh/uv/install.sh | sh

Build phase (excluded from per-run timing):
    cargo test --no-run [forwarded args]
Test phase:
    Run 1 uses a generous 2-hour timeout.
    Runs 2..N use 3 × the longest completed run's elapsed time + 30 s as the
    timeout (min 60 s), on the theory that a healthy test should never take
    substantially longer than its slowest previous successful run. The
    longest-run baseline is refreshed after every run, so a run that legitimately
    takes longer than its predecessors widens the timeout for the runs after it.

Output: when stdout is a TTY (and `--plain` wasn't passed), a `rich` TUI
shows scrolling cargo output on top and a status panel (run X/N, elapsed,
timeout, stats) pinned at the bottom. Otherwise cargo output streams
line-by-line as plain text. Every invocation gets its own log directory at
`.flaky-runs/{YYYYmmdd-HHMMSS-XXXXXX}/` under the repository root (outside
`target/` so `cargo clean` doesn't wipe it), with `build.log` and
`run-NN.log` files inside. `.flaky-runs/latest` points at the most recent
invocation. `RUST_BACKTRACE` is forced to `1` (overriding `0` or unset),
but a deliberately more verbose value like `full` is respected.

In TUI mode, cargo is run through a pseudo-terminal so its output is
line-buffered and colorized — same as you'd see running `cargo test`
directly. With `--plain`, cargo runs through a regular pipe and its output
is ANSI-stripped and block-buffered — suitable for `tee`/`grep` consumers.

On the first failing or timed-out run, exits 1 and points at the failing log.
Otherwise exits 0 after all runs succeed.

Usage:
    tools/detect-flaky-tests.py [--plain] <num_runs> [cargo test args...]

Examples:
    tools/detect-flaky-tests.py 50
    tools/detect-flaky-tests.py 50 -p cryfs-runner
    tools/detect-flaky-tests.py 50 -p cryfs-runner --test daemon_child_lifecycle
    tools/detect-flaky-tests.py 50 -p cryfs-runner -- --test-threads=1 --nocapture
"""

from __future__ import annotations

import abc
import argparse
import collections
import contextlib
import datetime
import errno
import fcntl
import os
import pty
import re
import select
import shutil
import signal
import struct
import subprocess
import sys
import tempfile
import termios
import threading
import time
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    # Used only in type annotations, which `from __future__ import annotations`
    # keeps as strings — no runtime cost.
    from collections.abc import Callable

from rich.layout import Layout
from rich.live import Live
from rich.panel import Panel
from rich.text import Text

GRACE_AFTER_SIGTERM_S: float = 30.0
FIRST_RUN_TIMEOUT_S: float = 2 * 60 * 60  # 2 hours
MIN_SUBSEQUENT_TIMEOUT_S: float = 60.0
SUBSEQUENT_TIMEOUT_MULTIPLIER: float = 3.0
SUBSEQUENT_TIMEOUT_JITTER_PAD_S: float = 30.0
LOG_SUBDIR = ".flaky-runs"
LOG_PANEL_LINES = 60  # last N lines of cargo output kept in the log panel
REFRESH_HZ = 8
READER_JOIN_TIMEOUT_S = 2.0
READER_POLL_S = 0.2  # how often the reader thread checks `shutting_down`
READ_CHUNK_BYTES = 4096

ANSI_ESC_RE = re.compile(rb"\x1b\[[0-9;?]*[a-zA-Z]")

# Warnings collected during a run (e.g. stuck reader threads). Printed by
# `main()` after the TUI has cleared, so rich's stderr capture doesn't eat
# them. Module-level rather than threaded through 4 signatures.
_warnings: list[str] = []


class FlakyRunnerError(Exception):
    """Internal failure (reader-thread crash, unkillable process, etc.) that
    should abort with a clear stderr message and exit 1.
    """


class ReaderState:
    """Coordination between the main thread and the reader thread.

    `shutting_down` is set by the main thread before it force-closes the
    pipe; the reader uses this to distinguish an expected pipe-close-during-
    cleanup from a real I/O error. `error` captures any unexpected exception
    so the main thread can surface it instead of silently swallowing.
    """

    def __init__(self) -> None:
        self.shutting_down: threading.Event = threading.Event()
        self.error: BaseException | None = None


def fmt_duration(seconds: float) -> str:
    if seconds < 60:
        return f"{seconds:.1f}s"
    m, s = divmod(seconds, 60)
    if m < 60:
        return f"{int(m)}m{s:04.1f}s"
    h, m = divmod(m, 60)
    return f"{int(h)}h{int(m):02d}m{s:04.1f}s"


def stats_line(times: list[float]) -> str:
    if len(times) < 2:
        return ""
    mean = sum(times) / len(times)
    return (
        f"mean {fmt_duration(mean)}, "
        f"min {fmt_duration(min(times))}, "
        f"max {fmt_duration(max(times))}"
    )


def subsequent_timeout(longest_run_elapsed: float) -> float:
    return max(
        MIN_SUBSEQUENT_TIMEOUT_S,
        SUBSEQUENT_TIMEOUT_MULTIPLIER * longest_run_elapsed
        + SUBSEQUENT_TIMEOUT_JITTER_PAD_S,
    )


def split_argv(argv: list[str]) -> tuple[list[str], list[str]]:
    """Split argv at the first positional argument (num_runs).

    Everything up to and including num_runs is parsed by argparse; everything
    after is forwarded verbatim to `cargo test`. Avoids the deprecated
    `argparse.REMAINDER` and lets users route `--help` to either side
    depending on placement.
    """
    for i, arg in enumerate(argv):
        if not arg.startswith("-") or arg == "--":
            return argv[: i + 1], argv[i + 1 :]
    return argv, []


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--plain",
        action="store_true",
        help="fully plain text output: no TUI, no PTY (cargo runs through a "
        "regular pipe, so its output is ANSI-stripped and block-buffered — "
        "suitable for `tee`/`grep` consumers). Auto-enabled when stdout "
        "isn't a TTY.",
    )
    parser.add_argument(
        "num_runs",
        type=int,
        help="positive integer: number of times to run 'cargo test'",
    )
    args = parser.parse_args(argv)
    if args.num_runs < 1:
        parser.error(f"num_runs must be a positive integer, got: {args.num_runs}")
    return args


def repo_root() -> Path:
    script_dir = Path(__file__).resolve().parent
    candidate = script_dir.parent
    if not (candidate / "Cargo.toml").is_file():
        sys.exit(f"Error: could not locate Cargo.toml at repository root: {candidate}")
    return candidate


def check_platform() -> None:
    if os.name != "posix":
        sys.exit(
            "Error: this script requires a POSIX platform (Linux/macOS); "
            f"detected os.name={os.name!r}."
        )


def _kill_group(
    proc: subprocess.Popen,
    grace_s: float = GRACE_AFTER_SIGTERM_S,
    kill_wait_s: float = 10.0,
) -> None:
    """Tear down a process group started with start_new_session=True.

    SIGTERM the whole group, give it `grace_s` to wind down, then SIGKILL
    unconditionally — closes the window where cargo exits gracefully but a
    test-binary descendant it spawned is still alive in the group. Always
    reaps `proc` itself; orphan descendants of cargo are reaped by init.

    `proc.pid == pgid` because start_new_session=True made `proc` a session
    leader. Timeouts parameterized so the user-interrupt path can use
    shorter values than the normal timeout-driven kill.
    """
    try:
        os.killpg(proc.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    try:
        proc.wait(timeout=grace_s)
    except subprocess.TimeoutExpired:
        pass
    try:
        os.killpg(proc.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    try:
        proc.wait(timeout=kill_wait_s)
    except subprocess.TimeoutExpired as e:
        raise FlakyRunnerError(
            f"Process {proc.pid} (cargo) did not exit within {kill_wait_s}s "
            "after SIGKILL. Likely stuck in uninterruptible kernel I/O "
            "(D state). Aborting."
        ) from e


def _open_pty() -> tuple[int, int]:
    """Open a PTY pair, sized to the user's terminal, with ONLCR disabled.

    Size and termios tweaks are best-effort: a PTY without them still works,
    cargo just won't size its progress bars correctly or may emit CRLF
    (which `_normalize_for_log` handles anyway).
    """
    try:
        master_fd, slave_fd = pty.openpty()
    except OSError as e:
        raise FlakyRunnerError(
            f"Failed to allocate pseudo-terminal: {e}. "
            "Either the system is out of PTYs, or run with --plain."
        ) from e
    with contextlib.suppress(OSError, termios.error):
        size = shutil.get_terminal_size()
        fcntl.ioctl(
            slave_fd,
            termios.TIOCSWINSZ,
            struct.pack("HHHH", size.lines, size.columns, 0, 0),
        )
        attrs = termios.tcgetattr(slave_fd)
        attrs[1] &= ~termios.ONLCR
        termios.tcsetattr(slave_fd, termios.TCSANOW, attrs)
    return master_fd, slave_fd


def _normalize_for_log(raw_line: bytes) -> bytes:
    """Sanitize a line for the on-disk log: convert CRLF→LF, then collapse
    progress-bar updates by keeping only the last non-empty state.

    Cargo's progress bars use \\r to overwrite within a single logical line
    (one \\n at the end). Without this, `cat`-ing the log shows truncated or
    overlapping text because the terminal interprets the embedded \\r's.

    Edge cases handled:
    - `b"a\\rb\\rc\\n"` → `b"c\\n"`   (intermediate states dropped)
    - `b"progress\\r"`  → `b"progress"` (no trailing \\n; keep the state)
    - `b"a\\rb\\r"`     → `b"b"`       (trailing \\r dropped, last state kept)
    - `b"\\r"` / `b""`  → `b""`        (no useful content)
    """
    raw = raw_line.replace(b"\r\n", b"\n")
    parts = [p for p in raw.split(b"\r") if p]
    return parts[-1] if parts else b""


def _drain_reader(
    reader: threading.Thread,
    state: ReaderState,
    log_path: Path,
) -> None:
    """Wait for the reader thread to exit.

    First join lets a healthy reader drain on its own (clean cargo exit →
    read returns b""). Then we signal shutdown — the reader's `select` loop
    notices on its next `READER_POLL_S` tick and exits. Second join collects
    it. A reader still alive after both joins is genuinely stuck (very
    unusual with the select-based design); we record a warning and move on.
    """
    reader.join(timeout=READER_JOIN_TIMEOUT_S)
    state.shutting_down.set()
    reader.join(timeout=READER_JOIN_TIMEOUT_S)
    if reader.is_alive():
        _warnings.append(
            f"log reader for {log_path} did not exit within "
            f"{2 * READER_JOIN_TIMEOUT_S:g}s; output in that file may be "
            "incomplete (a daemonized grandchild of cargo may have "
            "inherited the pipe)."
        )


def _stream_to_log(
    pipe_fd: int,
    log_path: Path,
    on_line: "Callable[[bytes], None]",
    state: ReaderState,
) -> None:
    """Reader thread body: read raw bytes from `pipe_fd`, write to `log_path`,
    dispatch lines to `on_line`.

    Uses `select` with `READER_POLL_S` timeouts so the loop periodically
    checks `state.shutting_down`. This is critical: a blocking `readline()`
    (the obvious shape) can never be interrupted by another thread when a
    daemonized grandchild of cargo holds the pipe's write end open — closing
    the fd from outside doesn't reliably unblock an in-progress `read()`,
    and `BufferedReader.close()` deadlocks on its own internal lock against
    the in-progress read. Polling sidesteps both.

    Owns the log file lifecycle. Captures unexpected exceptions to
    `state.error` so the main thread surfaces them.
    """
    buf = b""
    try:
        with open(log_path, "wb") as log_file:
            while not state.shutting_down.is_set():
                try:
                    ready, _, _ = select.select([pipe_fd], [], [], READER_POLL_S)
                except (OSError, ValueError) as e:
                    # fd was closed underneath us, or invalid.
                    if state.shutting_down.is_set():
                        return
                    if isinstance(e, OSError) and e.errno == errno.EBADF:
                        return
                    state.error = e
                    return
                if not ready:
                    continue
                try:
                    chunk = os.read(pipe_fd, READ_CHUNK_BYTES)
                except OSError as e:
                    # Linux PTY quirk: when the slave end closes (cargo
                    # exits), reading the master end returns EIO instead of
                    # the normal 0-byte EOF.
                    if e.errno == errno.EIO:
                        break
                    if state.shutting_down.is_set():
                        return
                    state.error = e
                    return
                if not chunk:
                    break  # EOF (regular pipe, all writers closed)
                buf += chunk
                # Emit every complete line; keep any trailing partial line
                # in `buf` for the next iteration.
                while b"\n" in buf:
                    head, _, buf = buf.partition(b"\n")
                    line = head + b"\n"
                    log_file.write(ANSI_ESC_RE.sub(b"", _normalize_for_log(line)))
                    on_line(line)
            # Flush any trailing partial line (cargo exited without final \n,
            # or we're shutting down with bytes buffered). Both sinks are
            # thread-safe, so we emit unconditionally — the log file would
            # otherwise have bytes plain mode never saw on stdout.
            if buf:
                log_file.write(ANSI_ESC_RE.sub(b"", _normalize_for_log(buf)))
                on_line(buf)
    except Exception as e:
        if not state.shutting_down.is_set():
            state.error = e


def run_cargo(
    cmd: list[str],
    cwd: Path,
    env: dict[str, str],
    timeout_s: float | None,
    log_path: Path,
    on_line: Callable[[bytes], None],
    *,
    use_pty: bool,
) -> tuple[int, bool]:
    """Spawn cargo in a new session, stream output via `on_line` and to
    `log_path`. Returns (exit_code, timed_out).

    On timeout the whole process group is torn down. On KeyboardInterrupt
    the group is torn down (with shorter grace) before re-raising; a second
    Ctrl-C during cleanup abandons with one final best-effort SIGKILL.
    """
    try:
        log_path.parent.mkdir(parents=True, exist_ok=True)
    except OSError as e:
        raise FlakyRunnerError(
            f"Failed to create log directory {log_path.parent}: {e}"
        ) from e

    master_fd: int | None = None
    slave_fd: int | None = None
    if use_pty:
        master_fd, slave_fd = _open_pty()
        popen_stdout: int = slave_fd
    else:
        popen_stdout = subprocess.PIPE

    try:
        proc = subprocess.Popen(
            cmd,
            cwd=cwd,
            env=env,
            stdout=popen_stdout,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        )
    except OSError as e:
        for fd in (master_fd, slave_fd):
            if fd is not None:
                with contextlib.suppress(OSError):
                    os.close(fd)
        raise FlakyRunnerError(
            f"Failed to spawn cargo (is it in $PATH?): {e}"
        ) from e

    if slave_fd is not None:
        # Parent doesn't need the slave end; cargo holds it via dup2.
        os.close(slave_fd)
        pipe_fd = master_fd
    else:
        # `os.dup` gives the reader its own independent fd so it can never
        # race with `proc.stdout` (a BufferedReader) prefetching into its
        # internal buffer and stealing bytes. Both fds are closed in `finally`.
        try:
            pipe_fd = os.dup(proc.stdout.fileno())  # type: ignore[union-attr]
        except OSError as e:
            # fd exhaustion right after a successful Popen is extremely rare,
            # but if it happens we must not leak the running cargo group.
            with contextlib.suppress(ProcessLookupError, OSError):
                os.killpg(proc.pid, signal.SIGKILL)
            with contextlib.suppress(Exception):
                proc.wait(timeout=5.0)
            raise FlakyRunnerError(
                f"Failed to duplicate cargo stdout fd: {e}"
            ) from e

    reader_state = ReaderState()
    reader = threading.Thread(
        target=_stream_to_log,
        args=(pipe_fd, log_path, on_line, reader_state),
        daemon=True,
    )
    reader.start()

    try:
        rc = proc.wait(timeout=timeout_s)
        result = (rc, False)
    except subprocess.TimeoutExpired:
        try:
            _kill_group(proc)
        except KeyboardInterrupt:
            with contextlib.suppress(ProcessLookupError, OSError):
                os.killpg(proc.pid, signal.SIGKILL)
            raise
        result = (-1, True)
    except KeyboardInterrupt:
        # Short cleanup window for user-initiated abort. A second Ctrl-C
        # during cleanup abandons (one final SIGKILL, then propagate).
        try:
            _kill_group(proc, grace_s=5.0, kill_wait_s=5.0)
        except KeyboardInterrupt:
            with contextlib.suppress(ProcessLookupError, OSError):
                os.killpg(proc.pid, signal.SIGKILL)
        raise
    finally:
        _drain_reader(reader, reader_state, log_path)
        # Release the fd the reader was using. For PTY: we own master_fd.
        # For PIPE: the dup'd fd we made for the reader; proc.stdout will
        # close its own fd via Popen's cleanup.
        with contextlib.suppress(OSError):
            os.close(pipe_fd)
        if slave_fd is None and proc.stdout is not None:
            with contextlib.suppress(OSError, ValueError):
                proc.stdout.close()

    if reader_state.error is not None:
        raise FlakyRunnerError(
            f"Log reader thread failed: "
            f"{type(reader_state.error).__name__}: {reader_state.error}"
        ) from reader_state.error

    return result


# ---------------------------------------------------------------------------
# Display abstraction: how the build/run loop reports progress.
# `PlainDisplay` prints to stdout; `TuiDisplay` drives a rich Live layout.
# ---------------------------------------------------------------------------


class Display(abc.ABC):
    """Reports build/run progress. Concrete impls drive a TUI or print plainly.

    Uses `abc.ABC` + `@abstractmethod` so a partial implementation raises a
    clear `TypeError` at instantiation rather than `TypeError: 'NoneType'
    object is not callable` deep in the reader thread.
    """

    def __enter__(self) -> "Display":
        return self

    def __exit__(self, *exc) -> None:
        pass

    @abc.abstractmethod
    def announce_build(self, log_rel: Path) -> None: ...
    @abc.abstractmethod
    def announce_run(self, i: int, total: int, timeout: float, log_rel: Path) -> None: ...
    @abc.abstractmethod
    def announce_run_success(self, i: int, elapsed: float, elapsed_times: list[float]) -> None: ...
    @abc.abstractmethod
    def line_consumer(self) -> Callable[[bytes], None]: ...


class PlainDisplay(Display):
    """Stream cargo output to stdout, print run boundaries between."""

    def announce_build(self, log_rel: Path) -> None:
        print(f"\n===== Building tests (log: {log_rel}) =====", flush=True)

    def announce_run(self, i: int, total: int, timeout: float, log_rel: Path) -> None:
        print(
            f"\n===== Run {i} / {total} "
            f"(timeout {fmt_duration(timeout)}, log: {log_rel}) =====",
            flush=True,
        )

    def announce_run_success(
        self, i: int, elapsed: float, elapsed_times: list[float]
    ) -> None:
        stats = stats_line(elapsed_times)
        suffix = f" ({stats})" if stats else ""
        print(f"Run {i} succeeded in {fmt_duration(elapsed)}.{suffix}", flush=True)

    def line_consumer(self) -> Callable[[bytes], None]:
        out = sys.stdout.buffer

        def consume(raw_line: bytes) -> None:
            out.write(raw_line)
            out.flush()

        return consume


class LogBuffer:
    """Bounded thread-safe ring of recent display lines, rendered as a Panel."""

    def __init__(self, max_lines: int) -> None:
        self._lines: collections.deque[str] = collections.deque(maxlen=max_lines)
        self._lock = threading.Lock()

    def append(self, line: str) -> None:
        with self._lock:
            self._lines.append(line)

    def clear(self) -> None:
        with self._lock:
            self._lines.clear()

    def __rich__(self) -> Panel:
        with self._lock:
            snapshot = "\n".join(self._lines)
        return Panel(Text.from_ansi(snapshot), title="cargo output", border_style="dim")


class StatusState:
    """Mutable status displayed in the bottom panel of the TUI.

    Lock protects `elapsed_times` (append vs sum/min/max iteration) on
    free-threaded Python (PEP 779) and serves as a transaction marker for
    multi-attribute updates on GIL-CPython.
    """

    def __init__(self) -> None:
        self._lock: threading.Lock = threading.Lock()
        self.phase: str = ""
        self.run: int = 0
        self.total: int = 0
        self.run_start: float | None = None
        self.timeout: float | None = None
        self.elapsed_times: list[float] = []
        self.log_path: str = ""

    def update(self, **kwargs) -> None:
        with self._lock:
            for k, v in kwargs.items():
                setattr(self, k, v)

    def complete_run(self, elapsed: float) -> None:
        """Atomically clear run_start and append to elapsed_times — one lock
        acquisition so the render thread can't catch us in between (which
        would briefly show "no run in progress" but N-1 completed runs).
        """
        with self._lock:
            self.run_start = None
            self.elapsed_times.append(elapsed)

    def __rich__(self) -> Panel:
        with self._lock:
            phase = self.phase
            run = self.run
            total = self.total
            run_start = self.run_start
            timeout = self.timeout
            elapsed_times = list(self.elapsed_times)
            log_path = self.log_path

        parts: list[str] = [f"Run {run}/{total}" if run > 0 else (phase or "starting")]
        if run_start is not None:
            parts.append(f"elapsed {fmt_duration(time.monotonic() - run_start)}")
        if timeout is not None:
            parts.append(f"timeout {fmt_duration(timeout)}")
        line1 = "  •  ".join(parts)

        if elapsed_times:
            mean = sum(elapsed_times) / len(elapsed_times)
            line2 = (
                f"Stats: {len(elapsed_times)} done  •  "
                f"mean {fmt_duration(mean)}  •  "
                f"min {fmt_duration(min(elapsed_times))}  •  "
                f"max {fmt_duration(max(elapsed_times))}"
            )
        else:
            line2 = "Stats: (no completed runs yet)"

        line3 = f"Log: {log_path}" if log_path else ""

        body = "\n".join(line for line in (line1, line2, line3) if line)
        return Panel(Text(body), title="status", border_style="cyan")


class TuiDisplay(Display):
    """Drive a rich Live layout: scrolling cargo output + pinned status."""

    def __init__(self) -> None:
        self._state = StatusState()
        self._buf = LogBuffer(LOG_PANEL_LINES)
        layout = Layout()
        layout.split_column(
            Layout(self._buf, name="log", ratio=1),
            Layout(self._state, name="status", size=5),
        )
        # transient=True so the TUI region clears on exit, leaving clean
        # scrollback for the preamble + final summary.
        self._live = Live(
            layout, refresh_per_second=REFRESH_HZ, screen=False, transient=True
        )

    def __enter__(self) -> "TuiDisplay":
        self._live.__enter__()
        return self

    def __exit__(self, *exc) -> None:
        self._live.__exit__(*exc)

    def announce_build(self, log_rel: Path) -> None:
        self._buf.clear()
        self._state.update(
            phase="Building tests",
            run=0,
            run_start=time.monotonic(),
            timeout=None,
            log_path=str(log_rel),
        )

    def announce_run(self, i: int, total: int, timeout: float, log_rel: Path) -> None:
        self._buf.clear()
        self._state.update(
            phase="Running tests",
            run=i,
            total=total,
            run_start=time.monotonic(),
            timeout=timeout,
            log_path=str(log_rel),
        )

    def announce_run_success(
        self, i: int, elapsed: float, elapsed_times: list[float]
    ) -> None:
        self._state.complete_run(elapsed)

    def line_consumer(self) -> Callable[[bytes], None]:
        buf = self._buf

        def consume(raw_line: bytes) -> None:
            # Display: keep ANSI codes (rich's Text.from_ansi renders them
            # as colors) but collapse progress-bar history to the last
            # non-empty state — rich does NOT process bare \r as cursor-back.
            text = raw_line.decode("utf-8", errors="replace").replace("\r\n", "\n")
            states = [s for s in text.split("\r") if s]
            buf.append(states[-1].rstrip("\r\n") if states else "")

        return consume


# ---------------------------------------------------------------------------
# Main loop: build, then N runs.
# ---------------------------------------------------------------------------


def run_all(
    args: argparse.Namespace,
    repo: Path,
    log_dir: Path,
    env: dict[str, str],
    forward: list[str],
    use_pty: bool,
    display: Display,
) -> tuple[str, bool]:
    """Build + N test runs. Returns (final_message, is_error)."""
    elapsed_times: list[float] = []
    width = len(str(args.num_runs))

    build_log = log_dir / "build.log"
    display.announce_build(build_log.relative_to(repo))
    build_rc, _ = run_cargo(
        ["cargo", "test", "--no-run", *forward],
        cwd=repo,
        env=env,
        timeout_s=None,
        log_path=build_log,
        on_line=display.line_consumer(),
        use_pty=use_pty,
    )
    if build_rc != 0:
        return (
            f"FAILURE: 'cargo test --no-run' failed with exit code "
            f"{build_rc}. See {build_log}",
            True,
        )

    for i in range(1, args.num_runs + 1):
        timeout = (
            FIRST_RUN_TIMEOUT_S
            if not elapsed_times
            else subsequent_timeout(max(elapsed_times))
        )
        log_path = log_dir / f"run-{i:0{width}d}.log"
        display.announce_run(i, args.num_runs, timeout, log_path.relative_to(repo))

        start = time.monotonic()
        exit_code, timed_out = run_cargo(
            ["cargo", "test", *forward],
            cwd=repo,
            env=env,
            timeout_s=timeout,
            log_path=log_path,
            on_line=display.line_consumer(),
            use_pty=use_pty,
        )
        elapsed = time.monotonic() - start

        if timed_out:
            return (
                f"TIMEOUT: run {i} exceeded {fmt_duration(timeout)} "
                f"after {fmt_duration(elapsed)}. See {log_path}",
                True,
            )
        if exit_code != 0:
            return (
                f"FAILURE: run {i} failed with exit code {exit_code} "
                f"after {fmt_duration(elapsed)}. See {log_path}",
                True,
            )

        elapsed_times.append(elapsed)
        display.announce_run_success(i, elapsed, elapsed_times)

    return (f"All {args.num_runs} run(s) of 'cargo test' succeeded.", False)


def _setup_log_dir(repo: Path) -> Path:
    """Create a fresh `.flaky-runs/{timestamp}-{rand}/` dir and update the
    `latest` symlink to point at it. Race-safe against concurrent invocations.
    """
    log_base = repo / LOG_SUBDIR
    try:
        log_base.mkdir(parents=True, exist_ok=True)
    except OSError as e:
        sys.exit(f"Error: failed to create log base {log_base}: {e}")
    prefix = datetime.datetime.now().strftime("%Y%m%d-%H%M%S-")
    try:
        log_dir = Path(tempfile.mkdtemp(prefix=prefix, dir=log_base))
    except OSError as e:
        sys.exit(f"Error: failed to create per-run log dir under {log_base}: {e}")
    # mkdtemp creates mode 0o700; loosen so a co-developer on a shared box
    # can `tail -f` the logs. The log files inside use the umask default.
    with contextlib.suppress(OSError):
        os.chmod(log_dir, 0o755)

    # Best-effort `latest` symlink. Race-safe via temp+os.replace. Skipped
    # if `latest` exists as a non-symlink (user file/dir we shouldn't
    # clobber). Failures here are non-fatal.
    latest = log_base / "latest"
    if not (latest.exists() and not latest.is_symlink()):
        tmp = log_base / f".latest.tmp.{os.getpid()}"
        try:
            tmp.unlink(missing_ok=True)
            tmp.symlink_to(log_dir.name, target_is_directory=True)
            os.replace(tmp, latest)
        except OSError:
            with contextlib.suppress(OSError):
                tmp.unlink()

    return log_dir


def main(argv: list[str]) -> int:
    check_platform()
    parser_argv, forward = split_argv(argv[1:])
    args = parse_args(parser_argv)
    repo = repo_root()

    log_dir = _setup_log_dir(repo)
    env = os.environ.copy()
    # Force RUST_BACKTRACE on for this run: the script is a debugging tool,
    # the user is hunting flaky tests, they want backtraces. A `setdefault`
    # would silently respect an inherited `RUST_BACKTRACE=0` from the user's
    # shell rc, defeating the purpose. Deliberately more verbose settings
    # (`full`, etc.) are respected — only unset or `"0"` gets promoted.
    if env.get("RUST_BACKTRACE") in (None, "0", ""):
        env["RUST_BACKTRACE"] = "1"

    stdout_is_tty = sys.stdout.isatty()
    use_tui = (not args.plain) and stdout_is_tty
    # PTY only in TUI mode. `--plain` means "fully plain": no TUI, no PTY,
    # so cargo's stdout is a plain pipe with ANSI-stripped block-buffered
    # output suitable for grep/tee.
    use_pty = use_tui

    print(f"Repo:            {repo}")
    print(f"Logs:            {log_dir}")
    print(f"cargo args:      {' '.join(forward) if forward else '(none)'}")
    print(f"RUST_BACKTRACE:  {env['RUST_BACKTRACE']}")
    print(f"Output mode:     {'TUI' if use_tui else 'plain'}")
    print(f"Plan: build, then run 'cargo test' {args.num_runs} time(s).")
    print(f"  Run 1 timeout:        {fmt_duration(FIRST_RUN_TIMEOUT_S)}")
    print(
        f"  Subsequent timeouts:  "
        f"{SUBSEQUENT_TIMEOUT_MULTIPLIER:g}× longest completed run's elapsed time "
        f"+ {fmt_duration(SUBSEQUENT_TIMEOUT_JITTER_PAD_S)} "
        f"(min {fmt_duration(MIN_SUBSEQUENT_TIMEOUT_S)})"
    )
    sys.stdout.flush()

    display: Display = TuiDisplay() if use_tui else PlainDisplay()
    with display:
        final_message, is_error = run_all(
            args, repo, log_dir, env, forward, use_pty, display
        )

    print()
    # Any warnings collected during the run (rare; e.g. stuck reader). Printed
    # after the TUI has cleared so rich's stderr capture doesn't eat them.
    for w in _warnings:
        print(f"Warning: {w}", file=sys.stderr)
    print(final_message, file=sys.stderr if is_error else sys.stdout)
    return 1 if is_error else 0


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv))
    except KeyboardInterrupt:
        print("\nInterrupted.", file=sys.stderr)
        sys.exit(130)
    except FlakyRunnerError as e:
        print(f"\nFAILURE: {e}", file=sys.stderr)
        sys.exit(1)
