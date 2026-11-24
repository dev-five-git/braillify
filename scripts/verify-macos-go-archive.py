"""Verify that removing embedded LLVM sections preserves a Mach-O archive."""

import hashlib
import re
import subprocess
import sys
from pathlib import Path


def lines(tool, *args):
    process = subprocess.Popen([str(tool), *map(str, args)], stdout=subprocess.PIPE, text=True, encoding="utf-8")
    assert process.stdout is not None
    yield from process.stdout
    if process.wait():
        raise RuntimeError(f"{tool.name} failed")


def members(tools, archive):
    result = list(lines(tools / "llvm-ar", "t", archive))
    if not result:
        raise ValueError("archive has no members")
    return result


def symbols(tools, archive):
    result = []
    member = None
    for line in lines(tools / "llvm-nm", "--no-llvm-bc", "-m", archive):
        line = line.rstrip("\n")
        if line.endswith(":"):
            member = line
        elif member and "(" in line and "(__LLVM," not in line:
            result.append((member, line))
    if not result:
        raise ValueError("archive has no native symbols")
    return sorted(result)


def sections(tools, archive):
    digest = hashlib.sha256()
    section = False
    section_count = 0
    for line in lines(tools / "llvm-objdump", "-s", archive):
        member = re.search(r"\(([^()]*)\):\s*file format", line)
        if member:
            digest.update(f"member:{member.group(1)}\n".encode())
            section = False
            continue
        match = re.match(r"Contents of section (.*?):", line)
        if match:
            name = match.group(1)
            section = not name.startswith("__LLVM,")
            if section:
                section_count += 1
                digest.update(f"section:{name}\n".encode())
            continue
        if section and re.match(r"^\s*[0-9a-fA-F]{4,}\s", line):
            digest.update(line.encode())
    if not section_count:
        raise ValueError("archive has no native sections")
    return digest.hexdigest()


def relocations(tools, archive):
    digest = hashlib.sha256()
    count = 0
    for line in lines(tools / "llvm-readobj", "--relocations", archive):
        if line.startswith("File:"):
            line = re.sub(r"^File: .*\(([^()]*)\)$", r"File: \1", line.rstrip()) + "\n"
            count += 1
        digest.update(line.encode())
    if not count:
        raise ValueError("archive has no relocation records")
    return digest.hexdigest()


def main():
    if len(sys.argv) != 4:
        raise SystemExit("usage: verify-macos-go-archive.py ORIGINAL CANDIDATE LLVM_BIN_DIR")
    original, candidate, tools = map(Path, sys.argv[1:])
    for name, inspect in (("members", members), ("native symbols", symbols), ("non-LLVM sections", sections), ("relocations", relocations)):
        if inspect(tools, original) != inspect(tools, candidate):
            raise SystemExit(f"macOS archive {name} differ")
        print(f"macOS archive {name} match", flush=True)


if __name__ == "__main__":
    main()
