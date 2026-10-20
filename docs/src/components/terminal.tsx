"use client";

import { useEffect, useMemo, useRef, useState } from "react";

export interface TerminalLine {
  input: boolean;
  text: string;
}

const typing = 35;
const beforeTyping = 500;
const beforeOutput = 300;
const beforeReplay = 5000;

/** Types each input line, prints each output line whole, then replays. Starts once scrolled into view. */
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
          : [{ line: index, chars: line.text.length, delay: beforeOutput }],
      ),
      { line: lines.length, chars: 0, delay: beforeOutput },
    ],
    [lines],
  );
  const [step, setStep] = useState(0);
  const [playing, setPlaying] = useState(false);
  const ref = useRef<HTMLPreElement>(null);

  useEffect(() => {
    if (matchMedia("(prefers-reduced-motion: reduce)").matches) {
      setStep(steps.length - 1);
      return;
    }

    const node = ref.current;
    if (!node) return;

    const observer = new IntersectionObserver(([entry]) => {
      if (entry?.isIntersecting) {
        setPlaying(true);
        observer.disconnect();
      }
    });

    observer.observe(node);
    return () => observer.disconnect();
  }, [steps]);

  useEffect(() => {
    if (!playing) return;

    const last = step === steps.length - 1;
    const timer = setTimeout(
      () => setStep(last ? 0 : step + 1),
      last ? beforeReplay : (steps[step + 1]?.delay ?? 0),
    );

    return () => clearTimeout(timer);
  }, [playing, step, steps]);

  const current = steps[step] ?? { line: 0, chars: 0 };
  const typingLine = lines[current.line];

  return (
    <pre
      ref={ref}
      className="min-h-64 overflow-x-auto p-3 text-sm font-mono text-fd-secondary-foreground/80"
    >
      {lines.slice(0, current.line).map((line, index) => (
        <div key={index}>{line.input ? `$ ${line.text}` : line.text}</div>
      ))}
      {typingLine && (
        <div>
          {typingLine.input ? `$ ${typingLine.text.slice(0, current.chars)}` : typingLine.text}
          {typingLine.input && <span className="ms-px inline-block h-4 w-2 translate-y-0.5 bg-fd-primary" />}
        </div>
      )}
      {!typingLine && (
        <div>
          {"$ "}
          <span className="inline-block h-4 w-2 translate-y-0.5 bg-fd-primary" />
        </div>
      )}
    </pre>
  );
}
