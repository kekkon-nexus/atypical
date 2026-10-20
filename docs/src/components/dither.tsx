"use client";

import { type HTMLAttributes, useEffect, useRef } from "react";
import {
  type AttachmentOptions,
  bindFramebufferInfo,
  createFramebufferInfo,
  createProgramInfo,
  createTexture,
  createUniformBlockInfo,
  resizeFramebufferInfo,
  setBlockUniforms,
  setUniformBlock,
  setUniforms,
  type UniformBlockInfo,
} from "twgl.js";

import { cn } from "@/lib/utils";

import * as imageShader from "./dither-image.wgsl?glsl";
import * as patternShader from "./dither-pattern.wgsl?glsl";
import * as ditherShader from "./dither.wgsl?glsl";

const MODES = { random: 1, "2x2": 2, "4x4": 3, "8x8": 4, halftone: 5 } as const;
const LOAD_DURATION = 2000;

type Settings = {
  type: keyof typeof MODES;
  size: number;
  colorSteps: number;
  inverted: boolean;
  subdiv: number;
  gain: number;
  threshold: number;
  shading: number;
  shapeScale: number;
  noiseAmp: number;
  timeScale: number;
  seed?: number;
  /** An image URL to dither instead of the animated noise. */
  image?: string;
  pause: boolean;
  pageLoadAnimation: boolean;
};

export type DitherProps = HTMLAttributes<HTMLCanvasElement> & Partial<Settings>;

export default function Dither({
  type = "halftone",
  size = 32,
  colorSteps = 2,
  inverted = false,
  subdiv = 5,
  gain = 1,
  threshold = 0.1,
  shading = 1,
  shapeScale = 1.5,
  noiseAmp = 1,
  timeScale = 0.3,
  seed,
  image,
  pause = false,
  pageLoadAnimation = true,
  className,
  ...rest
}: DitherProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rendererRef = useRef<ReturnType<typeof createRenderer>>(null);

  useEffect(() => {
    if (!canvasRef.current) return;
    const renderer = createRenderer(canvasRef.current);
    rendererRef.current = renderer;
    return () => {
      renderer.dispose();
      rendererRef.current = null;
    };
  }, []);

  useEffect(() => {
    rendererRef.current?.update({
      type,
      size,
      colorSteps,
      inverted,
      subdiv,
      gain,
      threshold,
      shading,
      shapeScale,
      noiseAmp,
      timeScale,
      seed,
      image,
      pause,
      pageLoadAnimation,
    });
  }, [
    type,
    size,
    colorSteps,
    inverted,
    subdiv,
    gain,
    threshold,
    shading,
    shapeScale,
    noiseAmp,
    timeScale,
    seed,
    image,
    pause,
    pageLoadAnimation,
  ]);

  return <canvas ref={canvasRef} className={cn("block size-full", className)} {...rest} />;
}

function createRenderer(canvas: HTMLCanvasElement) {
  const doc = canvas.ownerDocument;
  const gl = canvas.getContext("webgl2");
  if (!gl) return { update() {}, dispose() {} };

  const attachments: AttachmentOptions[] = [
    { internalFormat: gl.R8, minMag: gl.NEAREST, wrap: gl.CLAMP_TO_EDGE },
  ];
  const setup = () => {
    const pattern = createProgramInfo(gl, [patternShader.vertex, patternShader.fragment]);
    const picture = createProgramInfo(gl, [imageShader.vertex, imageShader.fragment]);
    const dither = createProgramInfo(gl, [ditherShader.vertex, ditherShader.fragment]);
    return {
      pattern,
      picture,
      dither,
      patternBlock: createUniformBlockInfo(gl, pattern, patternShader.uniformBlock),
      pictureBlock: createUniformBlockInfo(gl, picture, imageShader.uniformBlock),
      ditherBlock: createUniformBlockInfo(gl, dither, ditherShader.uniformBlock),
      grid: createFramebufferInfo(gl, attachments, 1, 1),
    };
  };
  let res: ReturnType<typeof setup> | null = setup();

  const set = (block: UniformBlockInfo, struct: string, values: Record<string, number | number[]>) =>
    setBlockUniforms(
      block,
      Object.fromEntries(Object.entries(values).map(([name, value]) => [`${struct}.${name}`, value])),
    );

  let settings: Settings | undefined;
  let time: number | undefined;
  let last = 0;
  let loadStart: number | undefined;
  let raf = 0;
  let dpr = 1;
  let gridSize = "";
  let visible = !doc.hidden;
  let inView = true;
  let color = [0, 0, 0, 1];
  let picture: { src: string; texture: WebGLTexture; aspect: number } | undefined;

  const dropPicture = () => {
    if (picture) gl.deleteTexture(picture.texture);
    picture = undefined;
  };

  const loadPicture = (src: string) => {
    createTexture(gl, { src, minMag: gl.NEAREST, wrap: gl.CLAMP_TO_EDGE }, (error, texture, source) => {
      if (error || settings?.image !== src) {
        gl.deleteTexture(texture);
        return;
      }
      const image = source as HTMLImageElement;
      dropPicture();
      picture = { src, texture, aspect: image.naturalWidth / image.naturalHeight };
      loadStart = undefined;
      draw();
      schedule();
    });
  };

  const progress = () => {
    if (!settings?.pageLoadAnimation) return 1;
    if (loadStart === undefined) return 0;
    return Math.min((performance.now() - loadStart) / LOAD_DURATION, 1);
  };

  const draw = () => {
    if (!res || !settings || time === undefined || !canvas.width || !canvas.height) return;
    loadStart ??= performance.now();

    const cellSize = settings.size * dpr;
    const gridW = 2 * Math.ceil(canvas.width / (2 * cellSize));
    const gridH = 2 * Math.ceil(canvas.height / (2 * cellSize));
    if (gridSize !== `${gridW}x${gridH}`) {
      gridSize = `${gridW}x${gridH}`;
      resizeFramebufferInfo(gl, res.grid, attachments, gridW, gridH);
    }

    bindFramebufferInfo(gl, res.grid);
    if (settings.image) {
      if (picture?.src !== settings.image) return;
      set(res.pictureBlock, imageShader.uniformStruct, {
        gridSize: [gridW, gridH],
        imageAspect: picture.aspect,
        loadProgress: progress(),
      });
      gl.useProgram(res.picture.program);
      setUniformBlock(gl, res.picture, res.pictureBlock);
      setUniforms(res.picture, { [imageShader.samplers[0] as string]: picture.texture });
    } else {
      set(res.patternBlock, patternShader.uniformStruct, {
        time,
        noiseAmp: settings.noiseAmp,
        loadProgress: progress(),
      });
      gl.useProgram(res.pattern.program);
      setUniformBlock(gl, res.pattern, res.patternBlock);
    }
    gl.drawArrays(gl.TRIANGLES, 0, 3);

    set(res.ditherBlock, ditherShader.uniformStruct, {
      color,
      resolution: [canvas.width, canvas.height],
      cellSize,
      mode: MODES[settings.type],
      colorSteps: settings.colorSteps,
      inverted: Number(settings.inverted),
      subdiv: settings.subdiv,
      gain: settings.gain,
      threshold: settings.threshold,
      shading: settings.shading,
      shapeScale: settings.shapeScale,
    });
    bindFramebufferInfo(gl, null);
    gl.useProgram(res.dither.program);
    setUniformBlock(gl, res.dither, res.ditherBlock);
    setUniforms(res.dither, { [ditherShader.samplers[0] as string]: res.grid.attachments[0] });
    gl.drawArrays(gl.TRIANGLES, 0, 3);
  };

  // A still image only needs frames for its load fade.
  const running = () =>
    !!res && visible && inView && ((!settings?.pause && !settings?.image) || progress() < 1);

  const frame = (now: number) => {
    raf = 0;
    if (last && time !== undefined && !settings?.pause) {
      time += (now - last) * 0.001 * (settings?.timeScale ?? 0);
    }
    last = now;
    draw();
    schedule();
  };

  const schedule = () => {
    if (running()) {
      raf ||= requestAnimationFrame(frame);
    } else {
      cancelAnimationFrame(raf);
      raf = 0;
      last = 0;
    }
  };

  const probe = Object.assign(doc.createElement("canvas"), { width: 1, height: 1 }).getContext(
    "2d",
    { willReadFrequently: true },
  );
  const readColor = () => {
    if (!probe) return;
    probe.clearRect(0, 0, 1, 1);
    probe.fillStyle = getComputedStyle(canvas).color;
    probe.fillRect(0, 0, 1, 1);
    color = Array.from(probe.getImageData(0, 0, 1, 1).data, (channel) => channel / 255);
    draw();
  };
  readColor();
  const themeObserver = new MutationObserver(readColor);
  themeObserver.observe(doc.documentElement, { attributes: true, attributeFilter: ["class", "style"] });

  const resizeObserver = new ResizeObserver(([entry]) => {
    const css = entry?.contentBoxSize[0];
    if (!entry || !css?.inlineSize) return;
    const device = entry.devicePixelContentBoxSize?.[0];
    const width = device?.inlineSize ?? Math.round(css.inlineSize * devicePixelRatio);
    const height = device?.blockSize ?? Math.round(css.blockSize * devicePixelRatio);
    dpr = width / css.inlineSize;
    canvas.width = width;
    canvas.height = height;
    draw();
  });
  resizeObserver.observe(canvas);

  const intersectionObserver = new (doc.defaultView ?? window).IntersectionObserver(([entry]) => {
    inView = entry?.isIntersecting ?? true;
    schedule();
  });
  intersectionObserver.observe(canvas);

  const onVisibility = () => {
    visible = !doc.hidden;
    schedule();
  };
  doc.addEventListener("visibilitychange", onVisibility);

  const onLost = (event: Event) => {
    event.preventDefault();
    res = null;
    schedule();
  };
  const onRestored = () => {
    res = setup();
    gridSize = "";
    picture = undefined;
    if (settings?.image) loadPicture(settings.image);
    draw();
    schedule();
  };
  canvas.addEventListener("webglcontextlost", onLost);
  canvas.addEventListener("webglcontextrestored", onRestored);

  return {
    update(next: Settings) {
      const imageChanged = next.image !== settings?.image;
      settings = next;
      if (imageChanged) {
        dropPicture();
        if (next.image) loadPicture(next.image);
      }
      time ??= (next.seed ?? Math.random() * 100) * next.timeScale;
      draw();
      schedule();
    },
    dispose() {
      cancelAnimationFrame(raf);
      themeObserver.disconnect();
      resizeObserver.disconnect();
      intersectionObserver.disconnect();
      doc.removeEventListener("visibilitychange", onVisibility);
      canvas.removeEventListener("webglcontextlost", onLost);
      canvas.removeEventListener("webglcontextrestored", onRestored);
      if (!res) return;
      gl.deleteProgram(res.pattern.program);
      gl.deleteProgram(res.picture.program);
      gl.deleteProgram(res.dither.program);
      dropPicture();
      gl.deleteFramebuffer(res.grid.framebuffer);
      for (const attachment of res.grid.attachments) gl.deleteTexture(attachment as WebGLTexture);
      res = null;
    },
  };
}
