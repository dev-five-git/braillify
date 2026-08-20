import { mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const target = resolve(root, "target", "debug");
const source = resolve(root, "packages", "c", "tests", "smoke.c");
const include = resolve(root, "packages", "c", "include");
mkdirSync(target, { recursive: true });

function run(command: string, args: string[]) {
  const result = spawnSync(command, args, { cwd: root, stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}

run("cargo", ["build", "-p", "braillify-c"]);

if (process.platform === "win32") {
  const importLibrary = resolve(target, "braillify_c.dll.lib");
  for (const [language, flag] of [
    ["c", "/TC"],
    ["cpp", "/TP"],
  ] as const) {
    const output = resolve(target, `braillify-${language}-smoke.exe`);
    run(process.env.CC ?? "cl.exe", [
      "/nologo",
      flag,
      "/utf-8",
      "/W4",
      "/WX",
      `/I${include}`,
      source,
      "/link",
      `/LIBPATH:${target}`,
      importLibrary,
      `/OUT:${output}`,
    ]);
    run(output, []);
  }
} else {
  const rpath = `-Wl,-rpath,${target}`;
  for (const [language, compiler, standard] of [
    ["c", process.env.CC ?? "cc", "c11"],
    ["cpp", process.env.CXX ?? "c++", "c++17"],
  ] as const) {
    const output = resolve(target, `braillify-${language}-smoke`);
    const object = resolve(target, `braillify-${language}-smoke.o`);
    run(compiler, [
      `-std=${standard}`,
      "-x",
      language === "c" ? "c" : "c++",
      "-Wall",
      "-Wextra",
      "-Werror",
      `-I${include}`,
      "-c",
      source,
      "-o",
      object,
    ]);
    run(compiler, [
      object,
      `-L${target}`,
      "-lbraillify_c",
      rpath,
      "-o",
      output,
    ]);
    run(output, []);
  }
}
