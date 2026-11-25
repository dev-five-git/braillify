import importlib.util
import os
import struct
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


spec = importlib.util.spec_from_file_location("archive_verifier", Path(__file__).with_name("verify-macos-go-archive.py"))
verifier = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verifier)


class MachOArchiveTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory(prefix="braillify-macho-test-")
        cls.work = Path(cls.temp.name)
        sysroot = subprocess.check_output(["rustc", "--print", "sysroot"], text=True).strip()
        version = subprocess.check_output(["rustc", "-vV"], text=True)
        host = next(line.removeprefix("host: ") for line in version.splitlines() if line.startswith("host: "))
        cls.tools = Path(sysroot) / "lib" / "rustlib" / host / "bin"
        suffix = ".exe" if os.name == "nt" else ""
        target = host if host.endswith("apple-darwin") else "x86_64-apple-darwin"
        source = cls.work / "sample.rs"
        source.write_text('#![no_std]\n#[unsafe(no_mangle)] pub extern "C" fn sample(x: u32) -> u32 { x.wrapping_add(1) }\n', encoding="utf-8")
        obj = cls.work / "sample.o"
        cls.original = cls.work / "original.a"
        cls.candidate = cls.work / "candidate.a"
        subprocess.run(["rustc", "--crate-type=lib", "--emit=obj", "-Cembed-bitcode=yes", "--target", target, str(source), "-o", str(obj)], check=True)
        subprocess.run([str(cls.tools / ("llvm-ar" + suffix)), "rcs", str(cls.original), str(obj)], check=True)
        subprocess.run([str(cls.tools / ("llvm-objcopy" + suffix)), "--remove-section=__LLVM,__bitcode", "--remove-section=__LLVM,__cmdline", str(cls.original), str(cls.candidate)], check=True)

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def test_transformed_archive_preserves_metadata(self):
        for inspect in (verifier.members, verifier.symbols, verifier.sections, verifier.relocations, verifier.metadata, verifier.load_commands, verifier.linker_hints):
            self.assertEqual(inspect(self.tools, self.original), inspect(self.tools, self.candidate))

    def test_changed_native_symbol_is_rejected(self):
        data = self.candidate.read_bytes()
        self.assertIn(b"_sample\0", data)
        changed = self.work / "changed-symbol.a"
        changed.write_bytes(data.replace(b"_sample\0", b"_xample\0"))
        self.assertNotEqual(verifier.symbols(self.tools, self.candidate), verifier.symbols(self.tools, changed))
        result = subprocess.run([sys.executable, str(Path(verifier.__file__)), str(self.candidate), str(changed), str(self.tools)], capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("native symbols differ", result.stderr)

    def test_native_section_flag_change_is_rejected(self):
        data = bytearray(self.candidate.read_bytes())
        start = data.find(b"\xcf\xfa\xed\xfe")
        self.assertGreaterEqual(start, 0)
        command = start + 32
        for _ in range(struct.unpack_from("<I", data, start + 16)[0]):
            kind, size = struct.unpack_from("<II", data, command)
            if kind == 0x19:
                for i in range(struct.unpack_from("<I", data, command + 64)[0]):
                    section = command + 72 + 80 * i
                    if data[section:section + 16].rstrip(b"\0") == b"__text" and data[section + 16:section + 32].rstrip(b"\0") == b"__TEXT":
                        flags = struct.unpack_from("<I", data, section + 64)[0]
                        struct.pack_into("<I", data, section + 64, flags ^ 0x10000000)
                        changed = self.work / "changed.a"
                        changed.write_bytes(data)
                        self.assertEqual(verifier.sections(self.tools, self.candidate), verifier.sections(self.tools, changed))
                        self.assertNotEqual(verifier.metadata(self.tools, self.candidate), verifier.metadata(self.tools, changed))
                        result = subprocess.run([sys.executable, str(Path(verifier.__file__)), str(self.candidate), str(changed), str(self.tools)], capture_output=True, text=True)
                        self.assertNotEqual(result.returncode, 0)
                        self.assertIn("Mach-O metadata differ", result.stderr)
                        return
            command += size
        self.fail("native __TEXT,__text section not found")


if __name__ == "__main__":
    unittest.main()
