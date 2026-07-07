import { CursorMode } from "~/pkg/forge";
import { createRoot, createSignal } from "solid-js";

export type Hsv = {
  h: number;
  s: number;
  v: number;
};

export const store = createRoot(() => {
  const [cursorMode, setCursorMode] = createSignal(CursorMode.Insert);
  const [color, setColor] = createSignal<Hsv>({ h: 0, s: 0, v: 0 });

  return {
    cursorMode,
    setCursorMode,
    color,
    setColor,
  };
});
