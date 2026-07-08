import { CursorMode, Forge } from "~/pkg/forge";
import { createRoot, createSignal } from "solid-js";

export type Hsv = {
  h: number;
  s: number;
  v: number;
};

export const store = createRoot(() => {
  const [forge, setForge] = createSignal<Forge>();
  const [cursorMode, setCursorMode] = createSignal(CursorMode.Insert);
  const [color, setColor] = createSignal<Hsv>({ h: 0, s: 0, v: 0 });

  return {
    forge,
    setForge,
    cursorMode,
    setCursorMode,
    color,
    setColor,
  };
});
