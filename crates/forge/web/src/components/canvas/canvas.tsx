import { hsvToRgb } from "~/components/toolbar/color-picker";
import * as vyas from "~/pkg/forge.js";
import { store } from "~/store";
import { createEffect, onMount } from "solid-js";
import styles from "./canvas.module.css";

export function Canvas() {
  const { color, cursorMode, forge, setForge } = store;

  onMount(async () => {
    setForge(new vyas.Forge());
  });

  createEffect(() => {
    const { red, green, blue } = hsvToRgb(color());
    forge()?.set_color(red, green, blue);
  });

  createEffect(() => {
    forge()?.set_cursor_mode(cursorMode());
  });

  return <canvas id="vyas" class={styles.root} />;
}
