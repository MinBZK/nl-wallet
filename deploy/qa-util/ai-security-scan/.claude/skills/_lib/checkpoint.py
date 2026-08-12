#!/usr/bin/env python3
"""Checkpoint helper for the NL Wallet security scan skills.

Persists phase/stage output to disk so a failed or interrupted run can
resume from the last completed step rather than starting over.

Usage:
    save   <state_dir> <N> [<name>] --from F [--key K]
    shard  <state_dir> <shard_id> --from F
    done   <state_dir> <N> [--key K]
    load   <state_dir>
    append <output_file> --from F
    reset  <state_dir>

All payloads are read from a file supplied via --from, never from stdin or
a heredoc. This keeps any repo-derived bytes out of the Bash argument list,
which prevents content from colliding with shell delimiters.

All writes are atomic (write to a .tmp sibling, then os.replace) so a kill
mid-write never leaves a partial file that corrupts resume state.

All paths are confined to CHECKPOINT_ROOT (default: cwd). This bounds the
blast radius if a prompt-injected agent tries to write outside the repo.
"""
from __future__ import annotations

import json
import os
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path


_ROOT = Path(os.environ.get("CHECKPOINT_ROOT", ".")).resolve()


def _confined(p: str | Path, *, suffix: str | None = None) -> Path:
    """Resolve p and verify it stays within _ROOT.

    Raises SystemExit(2) if the resolved path escapes the root, or if a
    required filename suffix is not present."""
    resolved = Path(p).resolve()
    if not resolved.is_relative_to(_ROOT):
        print(f"checkpoint: path outside allowed root {_ROOT}: {p}", file=sys.stderr)
        raise SystemExit(2)
    if suffix and not resolved.name.endswith(suffix):
        print(f"checkpoint: {p} must end with {suffix!r}", file=sys.stderr)
        raise SystemExit(2)
    return resolved


def _safe_name(s: str, label: str) -> str:
    """Reject names that contain path separators or parent-directory tokens."""
    if "/" in s or os.sep in s or ".." in s:
        print(f"checkpoint: {label} must not contain path separators: {s!r}", file=sys.stderr)
        raise SystemExit(2)
    return s


def _atomic_write(dest: Path, text: str) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    tmp = dest.with_suffix(dest.suffix + ".tmp")
    tmp.write_text(text)
    os.replace(tmp, dest)


def _pop_flag(argv: list[str], flag: str, default: str | None = None) -> tuple[list[str], str | None]:
    """Remove --flag value from argv and return (remaining_argv, value)."""
    if flag in argv:
        i = argv.index(flag)
        value = argv[i + 1]
        return argv[:i] + argv[i + 2:], value
    return argv, default


def _read_from(path: str | None) -> str:
    """Read payload from --from <path>.  Refuses to read from stdin."""
    if path is None:
        print(
            "checkpoint: payload must come from --from <file>; "
            "stdin is disabled to prevent shell-injection via heredoc delimiters",
            file=sys.stderr,
        )
        raise SystemExit(2)
    return _confined(path).read_text()


def _read_json_from(path: str | None) -> str:
    raw = _read_from(path)
    try:
        json.loads(raw)
    except json.JSONDecodeError as exc:
        print(f"checkpoint: --from {path} is not valid JSON: {exc}", file=sys.stderr)
        raise SystemExit(1)
    return raw


def _update_progress(state_dir: Path, *, status: str, key: str, n: int, shards: list[str]) -> None:
    _atomic_write(
        state_dir / "progress.json",
        json.dumps({
            "status": status,
            f"{key}_done": n,
            "shards_done": shards,
            "updated": datetime.now(timezone.utc).isoformat(),
        }),
    )


# ---------------------------------------------------------------------------
# Sub-commands
# ---------------------------------------------------------------------------

def cmd_save(argv: list[str]) -> int:
    argv, key = _pop_flag(argv, "--key", "phase")
    assert key
    argv, src = _pop_flag(argv, "--from")
    if len(argv) < 2:
        print("usage: checkpoint.py save <state_dir> <N> [<name>] --from <file> [--key K]", file=sys.stderr)
        return 2
    state_dir = _confined(argv[0], suffix="-state")
    n = int(argv[1])
    key = _safe_name(key, "--key")
    label = argv[2] if len(argv) > 2 else f"{key}{n}"
    payload = _read_json_from(src)
    _atomic_write(state_dir / f"{key}{n}.json", payload)
    _update_progress(state_dir, status="running", key=key, n=n, shards=[])
    print(f"checkpoint: {key} {n} ({label}) saved → {state_dir}/")
    return 0


def cmd_shard(argv: list[str]) -> int:
    argv, src = _pop_flag(argv, "--from")
    if len(argv) != 2:
        print("usage: checkpoint.py shard <state_dir> <shard_id> --from <file>", file=sys.stderr)
        return 2
    state_dir = _confined(argv[0], suffix="-state")
    shard_id = _safe_name(argv[1], "shard_id")
    payload = _read_json_from(src)
    _atomic_write(state_dir / f"shard_{shard_id}.json", payload)
    progress_file = state_dir / "progress.json"
    progress: dict = json.loads(progress_file.read_text()) if progress_file.exists() else {"status": "running"}
    shards: list = progress.get("shards_done", [])
    if shard_id not in shards:
        shards.append(shard_id)
    progress["shards_done"] = shards
    progress["updated"] = datetime.now(timezone.utc).isoformat()
    _atomic_write(progress_file, json.dumps(progress))
    print(f"checkpoint: shard {shard_id} saved ({len(shards)} total)")
    return 0


def cmd_done(argv: list[str]) -> int:
    argv, key = _pop_flag(argv, "--key", "phase")
    assert key
    if len(argv) != 2:
        print("usage: checkpoint.py done <state_dir> <N> [--key K]", file=sys.stderr)
        return 2
    _update_progress(
        _confined(argv[0], suffix="-state"),
        status="complete",
        key=_safe_name(key, "--key"),
        n=int(argv[1]),
        shards=[],
    )
    print("checkpoint: marked complete")
    return 0


def cmd_load(argv: list[str]) -> int:
    if len(argv) != 1:
        print("usage: checkpoint.py load <state_dir>", file=sys.stderr)
        return 2
    progress_file = _confined(argv[0], suffix="-state") / "progress.json"
    sys.stdout.write(progress_file.read_text() if progress_file.exists() else '{"status": "absent"}')
    return 0


def cmd_append(argv: list[str]) -> int:
    argv, src = _pop_flag(argv, "--from")
    if len(argv) != 1:
        print("usage: checkpoint.py append <output_file> --from <file>", file=sys.stderr)
        return 2
    dest = _confined(argv[0])
    dest.parent.mkdir(parents=True, exist_ok=True)
    chunk = _read_from(src)
    with open(dest, "a") as fh:
        fh.write(chunk)
        if not chunk.endswith("\n"):
            fh.write("\n")
    print(f"checkpoint: appended {len(chunk)} bytes → {dest}")
    return 0


def cmd_reset(argv: list[str]) -> int:
    if len(argv) != 1:
        print("usage: checkpoint.py reset <state_dir>", file=sys.stderr)
        return 2
    target = _confined(argv[0], suffix="-state")
    if target.exists():
        shutil.rmtree(target)
        print(f"checkpoint: removed {target}/")
    return 0


COMMANDS = {
    "save": cmd_save,
    "shard": cmd_shard,
    "done": cmd_done,
    "load": cmd_load,
    "append": cmd_append,
    "reset": cmd_reset,
}


def main(argv: list[str]) -> int:
    if not argv or argv[0] not in COMMANDS:
        print(f"usage: checkpoint.py {{{'|'.join(COMMANDS)}}} ...", file=sys.stderr)
        return 2
    return COMMANDS[argv[0]](argv[1:])


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
