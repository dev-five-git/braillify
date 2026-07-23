"""Fetch braille conversion results from 점사랑 7.0 (BrailleLove.exe)
and add ``jeomsarang`` field to each test case entry.

Usage:
  cd braillove-case-collector && uv run python ../scripts/fetch-jeomsarang.py

Requires:
  - 점사랑 7.0 installed at
    ``C:\\Program Files (x86)\\Jeomsarang7\\BrailleLove.exe``
  - pywinauto (installed via braillove-case-collector's uv env)

UI changes vs 6.0:
  - Window title now starts with "점사랑 7.0 - [..." instead of bare "점사랑 6.0".
  - On startup a blank document is opened automatically, so the v6
    "새문서" button click + "확인(O)" confirmation dialog are no longer needed.
  - 작업 영역 (Pane) for input and the bottom Edit for internal output are
    unchanged.

Robustness features:
  - Pre-start ``taskkill /F /IM BrailleLove.exe`` so we always start with a
    clean slate (no stray processes from prior aborted runs).
  - Startup alerts ("점사랑 알림" — "자동 저장된 파일 복구") are dismissed by
    walking ALL top-level desktop windows via ``Desktop(backend='uia')``,
    not just the started Application's windows. The dismiss helper tries
    button click first and falls back to keyboard ESC.
  - Every entry's processing checks for stray alerts before typing.
  - Empty / blank output from 점사랑 is treated as an error (preserve
    previous value) rather than overwriting with an empty string. This is
    the same lesson learned from fetch-world.ts.

Policy:
  - Update ``entry["jeomsarang"]`` only on success.
  - On skip (LaTeX, empty input) or failure (typing/read/conversion error,
    blank output) the previous value is preserved so a transient GUI hiccup
    cannot wipe earlier successful runs.

NOTE: This script takes over the active window. Run when the PC is idle.
      ~2000 entries × ~3-5 s each ≈ 2-3 hours.
"""

import io
import json
import os
import subprocess
import sys
import time
from typing import Any, Dict, List

from pywinauto import Desktop
from pywinauto.application import Application
from pywinauto.keyboard import send_keys

# Force UTF-8 stdout so Korean log lines survive cp949 default codepage.
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding="utf-8", errors="replace")

# fmt: off
PATTERN = " a1b'k2l`cif/msp\"e3h9o6r^djg>ntq,*5<-u8v.%[$+x!&;:4\\0z7(_?w]#y)="
BRAILLE = "⠀⠁⠂⠃⠄⠅⠆⠇⠈⠉⠊⠋⠌⠍⠎⠏⠐⠑⠒⠓⠔⠕⠖⠗⠘⠙⠚⠛⠜⠝⠞⠟⠠⠡⠢⠣⠤⠥⠦⠧⠨⠩⠪⠫⠬⠭⠮⠯⠰⠱⠲⠳⠴⠵⠶⠷⠸⠹⠺⠻⠼⠽⠾⠿"
# fmt: on

SPECIAL_MAP = {"@": 8, "|": 51}

EXE_PATH = r"C:\Program Files (x86)\Jeomsarang7\BrailleLove.exe"
EXE_BASENAME = "BrailleLove.exe"
TITLE_RE = r"점사랑 7\.0.*"
TEST_CASES_DIR = os.path.join(os.path.dirname(__file__), "..", "test_cases")

# Modal alerts to dismiss on startup and during the run.
# Each entry: (title_substring, preferred_button_text, fallback_key).
ALERT_PATTERNS = [
    ("점사랑 알림", "취소", "{ESC}"),
]


def internal_to_unicode(internal: str) -> str:
    """Convert 점사랑 internal notation to unicode braille."""
    result = []
    for ch in internal:
        if ch in PATTERN:
            result.append(BRAILLE[PATTERN.index(ch)])
        elif ch in SPECIAL_MAP:
            result.append(BRAILLE[SPECIAL_MAP[ch]])
        else:
            # Unknown character — bail out; caller treats this as error/preserve.
            return ""
    return "".join(result)


def should_skip(entry: Dict[str, Any]) -> bool:
    if entry.get("note") == "LaTeX":
        return True
    if not entry.get("input", "").strip():
        return True
    return False


def escape_for_typekeys(text: str) -> str:
    """Escape special characters for pywinauto type_keys."""
    return (
        text.replace("{", "{{}")
        .replace("}", "{}}")
        .replace("(", "{(}")
        .replace(")", "{)}")
        .replace("+", "{+}")
        .replace("^", "{^}")
        .replace("%", "{%}")
        .replace("~", "{~}")
        .replace(" ", "{SPACE}")
    )


def discover_json_files() -> List[str]:
    files: List[str] = []
    for subdir in sorted(os.listdir(TEST_CASES_DIR)):
        subdir_path = os.path.join(TEST_CASES_DIR, subdir)
        if not os.path.isdir(subdir_path):
            continue
        for f in sorted(os.listdir(subdir_path)):
            if f.endswith(".json"):
                files.append(os.path.join(subdir_path, f))
    return files


def kill_stray_braillelove() -> int:
    """taskkill /F /IM BrailleLove.exe — returns number of processes killed."""
    try:
        result = subprocess.run(
            ["taskkill", "/F", "/IM", EXE_BASENAME],
            capture_output=True,
            text=True,
            timeout=10,
        )
        if result.returncode == 0:
            # Count lines like 'SUCCESS: ...'
            n = result.stdout.count("SUCCESS")
            return n
    except Exception:
        pass
    return 0


def dismiss_alerts_on_desktop(quiet: bool = False) -> int:
    """Walk all top-level desktop windows (BOTH uia and win32 backends) and
    dismiss any matching modal alerts.

    The '점사랑 알림' modal is a classic Win32 #32770 dialog and pywinauto's
    uia backend silently misses it. The win32 backend sees it. We try both.

    Returns the number of distinct alerts dismissed in this pass.
    """
    dismissed_keys: set = set()  # (pid, title) tuples already handled

    def _try_dismiss_win32(w, title: str, pid: int) -> bool:
        """First click 취소 button via win32 children, then fall back to keyboard."""
        # 1) Try to find a button child named "취소" or "확인" and click it directly.
        for desired in ("취소", "확인"):
            try:
                for child in w.children():
                    try:
                        ct = child.window_text() or ""
                    except Exception:
                        continue
                    if ct == desired:
                        try:
                            child.click()
                            if not quiet:
                                print(
                                    f"  alert dismissed via win32 button {desired!r}: {title!r}",
                                    flush=True,
                                )
                            return True
                        except Exception:
                            try:
                                child.click_input()
                                if not quiet:
                                    print(
                                        f"  alert dismissed via click_input {desired!r}: {title!r}",
                                        flush=True,
                                    )
                                return True
                            except Exception:
                                pass
            except Exception:
                pass

        # 2) Keyboard fallback: focus the alert and send ESC (cancel).
        for key, label in (("{ESC}", "ESC"), ("{ENTER}", "ENTER")):
            try:
                w.set_focus()
                time.sleep(0.2)
                send_keys(key)
                time.sleep(0.4)
                # Re-check if alert is gone
                still = False
                try:
                    for w2 in Desktop(backend="win32").windows():
                        try:
                            t2 = w2.window_text() or ""
                            p2 = w2.process_id()
                        except Exception:
                            continue
                        if p2 == pid and t2 == title:
                            still = True
                            break
                except Exception:
                    pass
                if not still:
                    if not quiet:
                        print(f"  alert dismissed via {label}: {title!r}", flush=True)
                    return True
            except Exception:
                continue
        if not quiet:
            print(f"  alert dismiss FAILED: {title!r}", flush=True)
        return False

    # ----- win32 backend (catches #32770 modal dialogs) -----
    try:
        for w in Desktop(backend="win32").windows():
            try:
                title = w.window_text() or ""
                pid = w.process_id()
            except Exception:
                continue
            if not any(pat in title for pat, *_ in ALERT_PATTERNS):
                continue
            key = (pid, title)
            if key in dismissed_keys:
                continue
            if not quiet:
                print(f"  alert detected (win32): PID={pid} title={title!r}", flush=True)
            if _try_dismiss_win32(w, title, pid):
                dismissed_keys.add(key)
            time.sleep(0.2)
    except Exception as e:
        if not quiet:
            print(f"  alert scan (win32) failed: {e}", flush=True)

    # ----- uia backend (catches non-modal popups missed by win32) -----
    try:
        for w in Desktop(backend="uia").windows():
            try:
                title = w.window_text() or ""
                pid = w.process_id()
            except Exception:
                continue
            if not any(pat in title for pat, *_ in ALERT_PATTERNS):
                continue
            key = (pid, title)
            if key in dismissed_keys:
                continue
            if not quiet:
                print(f"  alert detected (uia): PID={pid} title={title!r}", flush=True)
            # Try descendant Button click
            clicked = False
            try:
                for b in w.descendants(control_type="Button"):
                    try:
                        bt = b.window_text()
                    except Exception:
                        continue
                    if bt in ("취소", "확인"):
                        try:
                            b.click()
                            clicked = True
                            break
                        except Exception:
                            try:
                                b.click_input()
                                clicked = True
                                break
                            except Exception:
                                pass
            except Exception:
                pass
            if not clicked:
                try:
                    w.set_focus()
                    time.sleep(0.2)
                    send_keys("{ESC}")
                    time.sleep(0.3)
                except Exception:
                    pass
            dismissed_keys.add(key)
            time.sleep(0.2)
    except Exception as e:
        if not quiet:
            print(f"  alert scan (uia) failed: {e}", flush=True)

    return len(dismissed_keys)


def dismiss_alerts_until_clear(max_passes: int = 4) -> None:
    """Repeat alert dismissal until no alerts remain (cascading alerts)."""
    for _ in range(max_passes):
        n = dismiss_alerts_on_desktop(quiet=True)
        if n == 0:
            return


def main() -> int:
    app = None
    try:
        # ----- 1. Clean slate ---------------------------------------------
        killed = kill_stray_braillelove()
        if killed:
            print(f"Killed {killed} stray {EXE_BASENAME} process(es) before start.", flush=True)
            time.sleep(1.5)

        # ----- 2. Start app -----------------------------------------------
        print(f"Starting {EXE_PATH} ...", flush=True)
        app = Application(backend="uia").start(EXE_PATH)
        time.sleep(3)

        # ----- 3. Dismiss startup alerts ----------------------------------
        # Some alerts cascade (dismissing one reveals another). Sweep a few times.
        for attempt in range(1, 5):
            n = dismiss_alerts_on_desktop()
            if n == 0:
                if attempt == 1:
                    print("  no startup alerts.", flush=True)
                break
            time.sleep(0.6)

        # ----- 4. Acquire main window + control handles -------------------
        main = app.window(title_re=TITLE_RE)
        if not main.exists(timeout=5):
            # The first app handle may point at a different PID (the splash
            # process). Re-search across the desktop.
            print("  app.window did not see main; falling back to Desktop scan ...", flush=True)
            for w in Desktop(backend="uia").windows():
                try:
                    if "점사랑 7.0" in (w.window_text() or "") and "알림" not in (w.window_text() or ""):
                        pid = w.process_id()
                        print(f"  found main window via Desktop scan: PID={pid}", flush=True)
                        app = Application(backend="uia").connect(process=pid)
                        main = app.window(title_re=TITLE_RE)
                        break
                except Exception:
                    continue
            if not main.exists(timeout=5):
                raise RuntimeError(
                    "Could not locate Jeomsarang 7 main window — UI changed unexpectedly"
                )

        try:
            main.set_focus()
        except Exception as e:
            print(f"  warning: set_focus failed: {e}", flush=True)
        time.sleep(0.3)
        try:
            main.maximize()
        except Exception as e:
            print(f"  warning: maximize failed: {e}", flush=True)
        time.sleep(0.3)

        # Final alert sweep after focus/maximize
        dismiss_alerts_until_clear()

        pane = main.child_window(title="작업 영역", control_type="Pane")
        if not pane.exists(timeout=3):
            raise RuntimeError("작업 영역 Pane not found in v7 main window")
        output = main.child_window(title="", control_type="Edit")
        if not output.exists(timeout=3):
            raise RuntimeError("Output Edit not found in v7 main window")

        # ----- 5. Iterate test cases --------------------------------------
        json_files = discover_json_files()
        # Optional ENV-based limit for quick dry-runs:
        #   set JEOMSARANG_LIMIT=2 → process only the first 2 files.
        limit_env = os.environ.get("JEOMSARANG_LIMIT", "").strip()
        if limit_env.isdigit() and int(limit_env) > 0:
            json_files = json_files[: int(limit_env)]
            print(f"JEOMSARANG_LIMIT={limit_env} → processing only first {len(json_files)} file(s)", flush=True)
        print(f"Discovered {len(json_files)} json files", flush=True)

        grand_total = 0
        grand_fetched = 0
        grand_skipped = 0
        grand_preserved = 0
        grand_errors = 0
        run_start = time.time()

        # Cheap counter to amortize per-entry alert sweep cost.
        entries_since_alert_sweep = 0
        ALERT_SWEEP_EVERY = 25

        for fi, filepath in enumerate(json_files, 1):
            filename = os.path.basename(filepath)
            dirpart = os.path.basename(os.path.dirname(filepath))
            label = f"{dirpart}/{filename}"

            with open(filepath, "r", encoding="utf-8") as f:
                entries = json.load(f)

            fetched = 0
            skipped = 0
            preserved = 0
            errors = 0
            file_start = time.time()

            for entry in entries:
                if should_skip(entry):
                    skipped += 1
                    continue

                text = entry["input"]
                prev = entry.get("jeomsarang", "")

                # Periodically sweep for late-arriving alerts (e.g. autosave hits).
                entries_since_alert_sweep += 1
                if entries_since_alert_sweep >= ALERT_SWEEP_EVERY:
                    dismiss_alerts_until_clear(max_passes=2)
                    entries_since_alert_sweep = 0
                    # Re-focus main pane in case dismiss changed focus.
                    try:
                        main.set_focus()
                    except Exception:
                        pass

                try:
                    escaped = escape_for_typekeys(text)
                    pane.type_keys(escaped, pause=0.02)
                    time.sleep(0.3)

                    internal = output.get_value()
                    unicode_result = internal_to_unicode(internal)

                    # Empty / unparseable output → treat as error (preserve).
                    if not internal or not unicode_result:
                        if prev:
                            preserved += 1
                        errors += 1
                        # Possibly an alert ate our keystrokes — dismiss and continue.
                        dismiss_alerts_until_clear(max_passes=2)
                        try:
                            main.set_focus()
                        except Exception:
                            pass
                    else:
                        entry["jeomsarang"] = unicode_result
                        fetched += 1

                    # Clear: Ctrl+A then Delete
                    pane.type_keys("^a")
                    time.sleep(0.05)
                    pane.type_keys("{DELETE}")
                    time.sleep(0.1)

                except Exception as e:
                    if prev:
                        preserved += 1
                    errors += 1
                    print(f"  Error for {text[:30]!r}: {e}", flush=True)
                    # Recovery: dismiss any alert and reset pane.
                    dismiss_alerts_until_clear(max_passes=2)
                    try:
                        main.set_focus()
                        pane.type_keys("^a")
                        time.sleep(0.05)
                        pane.type_keys("{DELETE}")
                        time.sleep(0.2)
                    except Exception:
                        pass

            # Save after each file so partial progress survives a crash.
            with open(filepath, "w", encoding="utf-8") as f:
                json.dump(entries, f, ensure_ascii=False, indent=2)
                f.write("\n")

            file_dur = time.time() - file_start
            extras = []
            if preserved:
                extras.append(f"{preserved} preserved")
            extras_str = ", " + ", ".join(extras) if extras else ""
            print(
                f"  [{fi:>3}/{len(json_files)}] {label} ... "
                f"{fetched} fetched, {skipped} skipped, {errors} errors"
                f"{extras_str} ({len(entries)} total) [{file_dur:.1f}s]",
                flush=True,
            )
            grand_total += len(entries)
            grand_fetched += fetched
            grand_skipped += skipped
            grand_preserved += preserved
            grand_errors += errors

        run_dur = time.time() - run_start
        print()
        print("=" * 60)
        print(f"Total: {grand_total} entries (in {run_dur / 60:.1f} min)")
        print(
            f"Fetched: {grand_fetched} | Skipped: {grand_skipped} | "
            f"Errors: {grand_errors} | Preserved: {grand_preserved}"
        )
        print("=" * 60)
        return 0

    except Exception as e:
        print(f"Fatal error: {e}", flush=True)
        import traceback

        traceback.print_exc()
        return 1
    finally:
        if app:
            try:
                app.kill()
            except Exception:
                pass
        # Final cleanup pass via taskkill for any stragglers.
        try:
            kill_stray_braillelove()
        except Exception:
            pass
        print("BrailleLove terminated.", flush=True)


if __name__ == "__main__":
    sys.exit(main())
