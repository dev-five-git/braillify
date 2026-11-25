"""Verify that removing embedded LLVM sections preserves a Mach-O archive."""

import hashlib
import re
import subprocess
import sys
from pathlib import Path


def lines(tool, *args):
    process = subprocess.Popen([str(tool), *map(str, args)], stdout=subprocess.PIPE, text=True, encoding="utf-8")
    assert process.stdout is not None
    with process.stdout as output:
        yield from output
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


def metadata(tools, archive):
    digest = hashlib.sha256()
    section = None
    segment = False
    count = 0
    for line in lines(tools / "llvm-readobj", "--file-headers", "--macho-segment", "--macho-version-min", "--macho-linker-options", "--macho-data-in-code", "--macho-indirect-symbols", "--sections", archive):
        if line.startswith("File:"):
            line = re.sub(r"^File: .*\(([^()]*)\)$", r"File: \1", line.rstrip()) + "\n"
            count += 1
        if line == "  Section {\n":
            section = [line]
            continue
        if section is not None:
            section.append(line)
            if line == "  }\n":
                if not any("Segment: __LLVM " in item for item in section):
                    for item in section:
                        if not re.match(r"\s*(Index|Offset|RelocationOffset):", item):
                            digest.update(item.encode())
                section = None
            continue
        if line == "Segment {\n":
            segment = True
        elif segment and line == "}\n":
            segment = False
        if re.match(r"\s*SizeOfLoadCommands:", line):
            continue
        if segment and re.match(r"\s*(Size|fileoff|filesize|nsects):", line):
            continue
        digest.update(line.encode())
    if not count or section is not None:
        raise ValueError("archive has invalid Mach-O metadata")
    return digest.hexdigest()


def load_commands(tools, archive):
    allowed = {"LC_SEGMENT_64", "LC_VERSION_MIN_MACOSX", "LC_BUILD_VERSION", "LC_SYMTAB", "LC_DYSYMTAB", "LC_DATA_IN_CODE", "LC_LINKER_OPTION", "LC_LINKER_OPTIMIZATION_HINT"}
    result = []
    block = None
    member = None

    def finish():
        if not block:
            return
        command = next((item.split()[1] for item in block if re.match(r"\s*cmd LC_", item)), None)
        if command not in allowed:
            raise ValueError(f"unsupported Mach-O load command: {command}")
        # Offsets and symbol-table counts legitimately change when LLVM sections disappear.
        ignored = r"\s*(fileoff|filesize|nsects|dataoff|symoff|stroff|nsyms|strsize|ilocalsym|nlocalsym|iextdefsym|nextdefsym|iundefsym|nundefsym|tocoff|ntoc|modtaboff|nmodtab|extrefsymoff|nextrefsyms|indirectsymoff|nindirectsyms|extreloff|nextrel|locreloff|nlocrel)\b"
        kept = []
        for item in block:
            if command == "LC_SEGMENT_64" and item.strip() == "Section":
                break
            if not re.match(ignored, item) and not (command == "LC_SEGMENT_64" and re.match(r"\s*cmdsize\b", item)):
                kept.append(item)
        result.append((member, tuple(kept)))

    for line in lines(tools / "llvm-objdump", "--macho", "--private-headers", archive):
        match = re.search(r"\(([^()]*)\):\s*$", line)
        if match:
            finish()
            block = None
            member = match.group(1)
        elif re.match(r"Load command \d+", line):
            finish()
            block = []
        elif member is not None and block is not None:
            block.append(line.rstrip())
    finish()
    if not result:
        raise ValueError("archive has no Mach-O load commands")
    return result


def linker_hints(tools, archive):
    digest = hashlib.sha256()
    count = 0
    for line in lines(tools / "llvm-objdump", "--macho", "--link-opt-hints", archive):
        if line.startswith("Archive :"):
            line = "Archive\n"
        elif re.search(r"\(([^()]*)\):\s*$", line):
            line = re.sub(r"^.*\(([^()]*)\):\s*$", r"member:\1", line.rstrip()) + "\n"
            count += 1
        digest.update(line.encode())
    if not count:
        raise ValueError("archive has no Mach-O members")
    return digest.hexdigest()


def main():
    if len(sys.argv) != 4:
        raise SystemExit("usage: verify-macos-go-archive.py ORIGINAL CANDIDATE LLVM_BIN_DIR")
    original, candidate, tools = map(Path, sys.argv[1:])
    for name, inspect in (("members", members), ("native symbols", symbols), ("non-LLVM sections", sections), ("relocations", relocations), ("Mach-O metadata", metadata), ("load commands", load_commands), ("linker optimization hints", linker_hints)):
        if inspect(tools, original) != inspect(tools, candidate):
            raise SystemExit(f"macOS archive {name} differ")
        print(f"macOS archive {name} match", flush=True)


if __name__ == "__main__":
    main()
