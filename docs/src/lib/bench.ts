import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";

const dir = path.resolve(process.cwd(), "../benches");

export interface Lane {
  name: string;
  mean: string;
}

export interface Bench {
  file: string;
  date: string;
  cpu: string;
  lanes: Lane[];
  latency: { atypical: number; commitlint: number };
  size: { atypical: number; commitlint: number; node: number };
  packages: { atypical: number; commitlint: number };
}

function field(text: string, name: string) {
  return new RegExp(`^${name}\\s*:\\s*(.*)$`, "m").exec(text)?.[1] ?? "";
}

function count(text: string, unit: string) {
  return Number(new RegExp(`(\\d+) ${unit}`).exec(text)?.[1] ?? Number.NaN);
}

/**
 * The newest committed bench snapshot.
 *
 * @returns The parsed `benches/result-YYYY-MM-DD.md`, or `undefined` without one that has the commitlint lanes.
 */
export function latestBench(): Bench | undefined {
  const file = readdirSync(dir)
    .filter((name) => /^result-\d{4}-\d{2}-\d{2}\.md$/.test(name))
    .toSorted()
    .at(-1);

  if (!file) return;

  const text = readFileSync(path.join(dir, file), "utf8");
  const lanes = text
    .split("\n")
    .filter((line) => line.startsWith("| `"))
    .map((line) => {
      const [, name = "", mean = ""] = line.split("|").map((cell) => cell.trim());

      return { name: name.replaceAll("`", ""), mean };
    });
  const mean = (name: string) =>
    Number(lanes.find((lane) => lane.name === name)?.mean.split(" ")[0] ?? "");

  const bench = {
    file,
    date: field(text, "date").slice(0, 10),
    cpu: field(text, "cpu"),
    lanes,
    latency: { atypical: mean("atypical (valid)"), commitlint: mean("commitlint (valid)") },
    size: {
      atypical: count(field(text, "binary"), "bytes"),
      commitlint: count(field(text, "deps"), "bytes"),
      node: count(field(text, "node"), "bytes"),
    },
    // @atypical/commit plus its one platform package, as npm installs it; the bench does not measure this
    packages: { atypical: 2, commitlint: count(field(text, "deps"), "packages") },
  };

  const numbers = [
    ...Object.values(bench.latency),
    ...Object.values(bench.size),
    ...Object.values(bench.packages),
  ];

  return numbers.every((number) => Number.isFinite(number)) ? bench : undefined;
}
