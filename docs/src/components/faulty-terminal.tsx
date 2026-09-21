"use client";
import {
  type HTMLAttributes,
  type MouseEvent as ReactMouseEvent,
  useCallback,
  useEffect,
  useMemo,
  useRef,
} from "react";
import {
  createProgramInfo,
  createUniformBlockInfo,
  resizeCanvasToDisplaySize,
  setBlockUniforms,
  setUniformBlock,
} from "twgl.js";

import { fragment, uniformBlock, uniformStruct, vertex } from "./faulty-terminal.wgsl?glsl";

import "./faulty-terminal.css";

const DEFAULT_GRID_MUL: [number, number] = [2, 1];

export type FaultyTerminalProps = HTMLAttributes<HTMLDivElement> & {
  scale?: number;
  gridMul?: [number, number];
  digitSize?: number;
  timeScale?: number;
  pause?: boolean;
  scanlineIntensity?: number;
  glitchAmount?: number;
  flickerAmount?: number;
  noiseAmp?: number;
  chromaticAberration?: number;
  dither?: number | boolean;
  curvature?: number;
  mouseReact?: boolean;
  mouseStrength?: number;
  dpr?: number;
  pageLoadAnimation?: boolean;
  brightness?: number;
};

export default function FaultyTerminal({
  scale = 1,
  gridMul = DEFAULT_GRID_MUL,
  digitSize = 1.5,
  timeScale = 0.3,
  pause = false,
  scanlineIntensity = 0.3,
  glitchAmount = 1,
  flickerAmount = 1,
  noiseAmp = 1,
  chromaticAberration = 0,
  dither = 0,
  curvature = 0.2,
  mouseReact = true,
  mouseStrength = 0.2,
  dpr,
  pageLoadAnimation = true,
  brightness = 1,
  className = "",
  style,
  ...rest
}: FaultyTerminalProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const mouseRef = useRef({ x: 0.5, y: 0.5 });
  const smoothMouseRef = useRef({ x: 0.5, y: 0.5 });
  const frozenTimeRef = useRef(0);
  const rafRef = useRef(0);
  const loadAnimationStartRef = useRef(0);
  const timeOffsetRef = useRef(Math.random() * 100);

  const ditherValue = useMemo(
    () => (typeof dither === "boolean" ? (dither ? 1 : 0) : dither),
    [dither],
  );

  const handleMouseMove = useCallback((e: MouseEvent | ReactMouseEvent) => {
    const ctn = containerRef.current;
    if (!ctn) return;
    const rect = ctn.getBoundingClientRect();
    const x = (e.clientX - rect.left) / rect.width;
    const y = 1 - (e.clientY - rect.top) / rect.height;
    mouseRef.current = { x, y };
  }, []);

  useEffect(() => {
    const ctn = containerRef.current;
    if (!ctn) return;

    const canvas = document.createElement("canvas");
    const gl = canvas.getContext("webgl2");
    if (!gl) return;

    const programInfo = createProgramInfo(gl, [vertex, fragment]);
    if (!programInfo) return;
    const blockInfo = createUniformBlockInfo(gl, programInfo, uniformBlock);
    const set = (values: Record<string, number | number[]>) =>
      setBlockUniforms(
        blockInfo,
        Object.fromEntries(
          Object.entries(values).map(([name, value]) => [`${uniformStruct}.${name}`, value]),
        ),
      );

    set({
      iTime: 0,
      uScale: scale,
      uGridMul: gridMul,
      uDigitSize: digitSize,
      uScanlineIntensity: scanlineIntensity,
      uGlitchAmount: glitchAmount,
      uFlickerAmount: flickerAmount,
      uNoiseAmp: noiseAmp,
      uChromaticAberration: chromaticAberration,
      uDither: ditherValue,
      uCurvature: curvature,
      uMouse: [smoothMouseRef.current.x, smoothMouseRef.current.y],
      uMouseStrength: mouseStrength,
      uUseMouse: mouseReact ? 1 : 0,
      uPageLoadProgress: pageLoadAnimation ? 0 : 1,
      uUsePageLoadAnimation: pageLoadAnimation ? 1 : 0,
      uBrightness: brightness,
    });
    gl.useProgram(programInfo.program);

    const pixelRatio = dpr ?? Math.min(window.devicePixelRatio || 1, 2);
    const resize = () => {
      resizeCanvasToDisplaySize(canvas, pixelRatio);
      gl.viewport(0, 0, canvas.width, canvas.height);
      set({ iResolution: [canvas.width, canvas.height, canvas.width / canvas.height] });
    };

    const probe = Object.assign(document.createElement("canvas"), {
      width: 1,
      height: 1,
    }).getContext("2d", { willReadFrequently: true });
    if (!probe) return;
    const toRgb = (css: string) => {
      probe.clearRect(0, 0, 1, 1);
      probe.fillStyle = css;
      probe.fillRect(0, 0, 1, 1);
      const [r = 0, g = 0, b = 0] = probe.getImageData(0, 0, 1, 1).data;
      return [r / 255, g / 255, b / 255];
    };
    const readColors = () => set({ uFg: toRgb(getComputedStyle(ctn).color) });
    readColors();
    const themeObserver = new MutationObserver(readColors);
    themeObserver.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class", "style"],
    });

    ctn.append(canvas);
    const resizeObserver = new ResizeObserver(() => resize());
    resizeObserver.observe(ctn);
    resize();

    const update = (t: number) => {
      rafRef.current = requestAnimationFrame(update);

      if (pageLoadAnimation && loadAnimationStartRef.current === 0) {
        loadAnimationStartRef.current = t;
      }

      if (pause) {
        set({ iTime: frozenTimeRef.current });
      } else {
        const elapsed = (t * 0.001 + timeOffsetRef.current) * timeScale;
        set({ iTime: elapsed });
        frozenTimeRef.current = elapsed;
      }

      if (pageLoadAnimation && loadAnimationStartRef.current > 0) {
        const animationDuration = 2000;
        const animationElapsed = t - loadAnimationStartRef.current;
        set({ uPageLoadProgress: Math.min(animationElapsed / animationDuration, 1) });
      }

      if (mouseReact) {
        const dampingFactor = 0.08;
        const smoothMouse = smoothMouseRef.current;
        const mouse = mouseRef.current;
        smoothMouse.x += (mouse.x - smoothMouse.x) * dampingFactor;
        smoothMouse.y += (mouse.y - smoothMouse.y) * dampingFactor;
        set({ uMouse: [smoothMouse.x, smoothMouse.y] });
      }

      setUniformBlock(gl, programInfo, blockInfo);
      gl.drawArrays(gl.TRIANGLES, 0, 3);
    };
    rafRef.current = requestAnimationFrame(update);

    if (mouseReact) ctn.addEventListener("mousemove", handleMouseMove);

    return () => {
      cancelAnimationFrame(rafRef.current);
      resizeObserver.disconnect();
      themeObserver.disconnect();
      if (mouseReact) ctn.removeEventListener("mousemove", handleMouseMove);
      canvas.remove();
      gl.getExtension("WEBGL_lose_context")?.loseContext();
      loadAnimationStartRef.current = 0;
      timeOffsetRef.current = Math.random() * 100;
    };
  }, [
    dpr,
    pause,
    timeScale,
    scale,
    gridMul,
    digitSize,
    scanlineIntensity,
    glitchAmount,
    flickerAmount,
    noiseAmp,
    chromaticAberration,
    ditherValue,
    curvature,
    mouseReact,
    mouseStrength,
    pageLoadAnimation,
    brightness,
    handleMouseMove,
  ]);

  return (
    <div
      ref={containerRef}
      className={`faulty-terminal-container ${className}`}
      style={style}
      {...rest}
    />
  );
}
