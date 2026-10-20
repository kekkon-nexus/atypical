"use client";

import { type ReactNode, useEffect, useMemo, useState } from "react";
import { Link } from "waku";

import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import type { Bench } from "@/lib/bench";
import { cn } from "@/lib/utils";

/** Toggles between panels rendered on the server; hidden panels stay in the markup. */
export function Switcher({
  options,
  className,
}: {
  options: { value: string; label: ReactNode; panel: ReactNode }[];
  className?: string;
}) {
  const [active, setActive] = useState(options[0]?.value);

  return (
    <div className={className}>
      <ToggleGroup
        variant="outline"
        size="sm"
        value={active ? [active] : []}
        onValueChange={(value: string[]) => {
          if (value[0]) setActive(value[0]);
        }}
      >
        {options.map(({ value, label }) => (
          <ToggleGroupItem
            key={value}
            value={value}
          >
            {label}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
      {options.map(({ value, panel }) => (
        <div
          key={value}
          hidden={value !== active}
        >
          {panel}
        </div>
      ))}
    </div>
  );
}

export type TerminalLine = { input: true; text: string } | { input: false; text: ReactNode };

const prompt = <span className="text-fd-primary">$ </span>;

const typing = 35;
const beforeTyping = 500;
const beforeOutput = 300;

/** Shows the finished session; on mouse enter, types each input line and prints each output line whole. */
export function Terminal({ lines }: { lines: TerminalLine[] }) {
  const steps = useMemo(
    () => [
      ...lines.flatMap((line, index) =>
        line.input
          ? Array.from({ length: line.text.length + 1 }, (_, chars) => ({
              line: index,
              chars,
              delay: chars === 0 ? beforeTyping : typing,
            }))
          : [{ line: index, chars: 0, delay: beforeOutput }],
      ),
      { line: lines.length, chars: 0, delay: beforeOutput },
    ],
    [lines],
  );
  const last = steps.length - 1;
  const [step, setStep] = useState(last);
  const [playing, setPlaying] = useState(false);

  useEffect(() => {
    if (!playing) return;

    if (step >= last) {
      setPlaying(false);
      return;
    }

    const timer = setTimeout(() => setStep(step + 1), steps[step + 1]?.delay ?? 0);

    return () => clearTimeout(timer);
  }, [playing, step, steps, last]);

  const replay = () => {
    if (playing || matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    setStep(0);
    setPlaying(true);
  };

  const current = steps[step] ?? { line: 0, chars: 0 };
  const typingLine = lines[current.line];

  return (
    <pre
      onMouseEnter={replay}
      className="min-h-64 overflow-x-auto p-3 text-sm font-mono text-fd-secondary-foreground/80"
    >
      {lines.slice(0, current.line).map((line, index) => (
        <div key={index}>
          {line.input && prompt}
          {line.text}
        </div>
      ))}
      {typingLine && (
        <div>
          {typingLine.input && prompt}
          {typingLine.input ? typingLine.text.slice(0, current.chars) : typingLine.text}
          {typingLine.input && <span className="ms-px inline-block h-4 w-2 translate-y-0.5 bg-fd-primary" />}
        </div>
      )}
      {!typingLine && (
        <div>
          {prompt}
          <span className="inline-block h-4 w-2 translate-y-0.5 bg-fd-primary" />
        </div>
      )}
    </pre>
  );
}

type Metric = "latency" | "size" | "packages";

interface Bar {
  name: string;
  value: string;
  segments: { amount: number; className: string }[];
}

interface Chart {
  title: string;
  unit: string;
  tick: (amount: number) => string;
  bars: Bar[];
}

const metrics: { value: Metric; label: string }[] = [
  { value: "latency", label: "Latency" },
  { value: "size", label: "Install size" },
  { value: "packages", label: "Packages" },
];

const ours = "bg-fd-primary";
const theirs = "bg-fd-muted-foreground/60";

const mb = (bytes: number) => bytes / 1e6;

const slowdown = 10;

function charts({ latency, size, packages }: Omit<Bench, "lanes" | "file">): Record<Metric, Chart> {
  return {
    latency: {
      title: "Linting a valid header",
      unit: "mean wall time · milliseconds (lower is better)",
      tick: (ms) => `${ms}ms`,
      bars: [
        {
          name: "atypical",
          value: `${latency.atypical}ms`,
          segments: [{ amount: latency.atypical, className: ours }],
        },
        {
          name: "commitlint",
          value: `${latency.commitlint}ms`,
          segments: [{ amount: latency.commitlint, className: theirs }],
        },
      ],
    },
    size: {
      title: "Install footprint",
      unit: "megabytes on disk (lower is better)",
      tick: (amount) => `${amount}MB`,
      bars: [
        {
          name: "atypical",
          value: `${mb(size.atypical).toFixed(1)}MB`,
          segments: [{ amount: mb(size.atypical), className: ours }],
        },
        {
          name: "commitlint",
          value: `${mb(size.commitlint).toFixed(1)}MB`,
          segments: [{ amount: mb(size.commitlint), className: theirs }],
        },
      ],
    },
    packages: {
      title: "Packages installed",
      unit: "count (lower is better)",
      tick: (amount) => `${amount}`,
      bars: [
        {
          name: "atypical",
          value: `${packages.atypical}`,
          segments: [{ amount: packages.atypical, className: ours }],
        },
        {
          name: "commitlint",
          value: `${packages.commitlint}`,
          segments: [{ amount: packages.commitlint, className: theirs }],
        },
      ],
    },
  };
}

/** A 1-2-5 step that splits `max` into at most five intervals. */
function axis(max: number) {
  const magnitude = 10 ** Math.floor(Math.log10(max / 5));
  const step = [1, 2, 5, 10].map((m) => m * magnitude).find((s) => max / s <= 5) ?? magnitude * 10;
  const end = Math.ceil(max / step) * step;

  return { end, ticks: Array.from({ length: end / step + 1 }, (_, i) => i * step) };
}

/**
 * The race clock in measured milliseconds, running `slowdown` times slower than real time from each change of `run` until
 * `finish`. A `run` of 0 rests at `finish`, showing the result. Reduced motion jumps straight to `finish`.
 *
 * @returns The milliseconds elapsed on the clock.
 */
function useRaceClock(run: number, finish: number) {
  const [elapsed, setElapsed] = useState(finish);

  useEffect(() => {
    if (run === 0) {
      setElapsed(finish);
      return;
    }

    if (matchMedia("(prefers-reduced-motion: reduce)").matches) {
      setElapsed(finish);
      return;
    }

    setElapsed(0);

    const start = performance.now();
    let frame = 0;
    const tick = (now: number) => {
      const ms = (now - start) / slowdown;

      setElapsed(Math.min(ms, finish));
      if (ms < finish) frame = requestAnimationFrame(tick);
    };

    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [run, finish]);

  return elapsed;
}

export function BenchRace({ bench }: { bench: Omit<Bench, "lanes" | "file"> }) {
  const [metric, setMetric] = useState<Metric>("latency");
  const [run, setRun] = useState(0);

  const finish = Math.max(bench.latency.atypical, bench.latency.commitlint);
  const elapsed = useRaceClock(run, finish);
  const racing = metric === "latency" && run > 0 && elapsed < finish;
  const chart = charts(bench)[metric];
  const totals = chart.bars.map((bar) => bar.segments.reduce((sum, { amount }) => sum + amount, 0));
  const { end, ticks } = axis(Math.max(...totals));

  return (
    <div
      className="flex flex-col gap-6"
      onMouseEnter={() => {
        if (metric === "latency" && !racing) setRun((n) => n + 1);
      }}
    >
      <ToggleGroup
        variant="outline"
        value={[metric]}
        onValueChange={(value: string[]) => {
          const next = value[0] as Metric | undefined;
          if (!next) return;
          setMetric(next);
          setRun(0);
        }}
      >
        {metrics.map(({ value, label }) => (
          <ToggleGroupItem
            key={value}
            value={value}
          >
            {label}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>

      <div className="flex flex-row items-start justify-between gap-4">
        <div>
          <h3 className="font-medium">{chart.title}</h3>
          <p className="text-sm text-fd-muted-foreground">{chart.unit}</p>
        </div>
        {metric === "latency" && (
          <button
            type="button"
            className="shrink-0 text-sm tabular-nums text-fd-muted-foreground hover:text-fd-foreground"
            onClick={() => setRun((n) => (racing ? 0 : n + 1))}
          >
            {racing ? (
              <span className="font-mono">
                <span className="me-1.5 inline-block size-2 bg-fd-primary" />
                {elapsed.toFixed(1)}ms
              </span>
            ) : (
              <>
                <span className="text-fd-primary">▶</span> replay at 1/{slowdown} speed
              </>
            )}
          </button>
        )}
      </div>

      <div className="flex flex-col gap-3">
        {chart.bars.map((bar, index) => {
          const total = totals[index] ?? 0;
          const shown = metric === "latency" ? Math.min(elapsed, total) : total;

          return (
            <Row
              key={bar.name}
              name={bar.name}
              value={metric === "latency" ? `${shown.toFixed(1)}ms` : bar.value}
              highlight={index === 0}
            >
              {bar.segments.map((segment) => (
                <div
                  key={segment.className}
                  className={cn("h-full min-w-1", segment.className)}
                  style={{ width: `${((metric === "latency" ? shown : segment.amount) / end) * 100}%` }}
                />
              ))}
              {racing && shown < total && (
                <span className="self-center px-2 text-xs whitespace-nowrap text-fd-muted-foreground">still going…</span>
              )}
            </Row>
          );
        })}
        <Row>
          <div className="relative h-5 w-full border-t border-fd-border text-[0.65rem] text-fd-muted-foreground font-mono">
            {ticks.map((tick) => (
              <span
                key={tick}
                className="absolute top-1 -translate-x-1/2"
                style={{ left: `${(tick / end) * 100}%` }}
              >
                {chart.tick(tick)}
              </span>
            ))}
          </div>
        </Row>
      </div>

      <div className="flex flex-row items-start justify-between gap-4 border-t pt-4 text-xs text-fd-muted-foreground">
        <p>
          hyperfine, 100 runs after 10 warmup · {bench.cpu} · {bench.date}
        </p>
        <Link
          to="/docs/benchmarks"
          className="shrink-0 underline underline-offset-2"
        >
          reproduce
        </Link>
      </div>
    </div>
  );
}

function Row({
  name,
  value,
  highlight,
  children,
}: {
  name?: string;
  value?: string;
  highlight?: boolean;
  children: ReactNode;
}) {
  return (
    <div className="grid grid-cols-[5.5rem_1fr_4.5rem] items-center gap-3 text-sm">
      <span className={cn(highlight && "font-medium")}>{name}</span>
      <div className={cn("flex", name && "h-6 overflow-hidden rounded-sm bg-fd-muted")}>{children}</div>
      <span
        className={cn("text-end font-mono tabular-nums", highlight ? "font-bold text-base" : "text-fd-muted-foreground")}
      >
        {value}
      </span>
    </div>
  );
}
