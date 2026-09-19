import { readdir, stat } from "node:fs/promises";
import { join, relative } from "node:path";

// The `/test-case` route once shipped a 108 MB HTML document because the whole
// NIKL corpus (83k rows) was handed to a client component as a prop, which
// serializes every row into the route's RSC payload. That broke the CI prerender
// (`ERR_ENCODING_INVALID_ENCODED_DATA`) and made the live page unusable.
//
// Any single prerendered document above the limit means route data is unbounded
// again: paginate it across static routes instead of raising this number.
const OUT_DIR = "apps/landing/out";
const MAX_DOCUMENT_BYTES = 6 * 1024 * 1024;
const DOCUMENT_EXTENSIONS = [".html", ".txt"];

async function collectDocuments(dir: string): Promise<string[]> {
  const entries = await readdir(dir, { withFileTypes: true });
  const nested = await Promise.all(
    entries.map(async (entry) => {
      const path = join(dir, entry.name);
      if (entry.isDirectory()) return collectDocuments(path);
      return DOCUMENT_EXTENSIONS.some((ext) => entry.name.endsWith(ext))
        ? [path]
        : [];
    }),
  );
  return nested.flat();
}

const documents = await collectDocuments(OUT_DIR);
const sized = await Promise.all(
  documents.map(async (path) => ({ path, bytes: (await stat(path)).size })),
);
sized.sort((a, b) => b.bytes - a.bytes);

const megabytes = (bytes: number) => `${(bytes / 1024 / 1024).toFixed(2)} MB`;

console.log(`Largest prerendered documents in ${OUT_DIR}:`);
for (const { path, bytes } of sized.slice(0, 5)) {
  console.log(`  ${megabytes(bytes).padStart(10)}  ${relative(OUT_DIR, path)}`);
}

const oversized = sized.filter(({ bytes }) => bytes > MAX_DOCUMENT_BYTES);
if (oversized.length > 0) {
  console.error(
    `\nPrerendered documents exceed ${megabytes(MAX_DOCUMENT_BYTES)}:`,
  );
  for (const { path, bytes } of oversized) {
    console.error(`  ${megabytes(bytes)}  ${relative(OUT_DIR, path)}`);
  }
  console.error(
    "\nRoute data is unbounded. Paginate it across static routes rather than raising the limit.",
  );
  process.exit(1);
}
