import { hexColorToU32, hsvToHex } from "~/components/toolbar/color-picker";
import * as vyas from "~/pkg/forge.js";
import { store } from "~/store";
import { createEffect, onMount } from "solid-js";
import styles from "./canvas.module.css";

export function Canvas() {
  const { color, cursorMode } = store;

  onMount(async () => {
    vyas.init();
  });

  createEffect(() => {
    const hex = hsvToHex(color());
    vyas.set_color(hexColorToU32(hex));
  });

  createEffect(() => {
    vyas.set_cursor_mode(cursorMode());
  });

  return <canvas id="vyas" class={styles.root} />;
}
