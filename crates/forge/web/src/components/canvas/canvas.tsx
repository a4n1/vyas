import { hsvToRgb } from "~/components/toolbar/color-picker";
import { Grid, serializeGrid } from "~/components/toolbar/save";
import * as vyas from "~/pkg/forge.js";
import { store } from "~/store";
import { createEffect, onCleanup, onMount } from "solid-js";
import styles from "./canvas.module.css";

export function Canvas() {
  const { color, cursorMode, forge, setForge } = store;

  onMount(() => {
    setForge(new vyas.Forge());
  });

  onMount(() => {
    const handleGridUpdate = () => {
      const grid: Grid = forge()?.export_grid();
      const csv = serializeGrid(grid);
      localStorage.setItem("last_save", btoa(csv));
    };

    window.addEventListener("forge:grid-update", handleGridUpdate);

    onCleanup(() =>
      window.removeEventListener("forge:grid-update", handleGridUpdate),
    );
  });

  createEffect(async () => {
    const lastSave = localStorage.getItem("last_save");

    if (!lastSave) {
      const response = await fetch("/models/tree.csv");
      const bytes = new Uint8Array(await response.arrayBuffer());
      forge()?.load_grid(bytes);
      return;
    }

    const csv = atob(lastSave);
    forge()?.load_grid(new TextEncoder().encode(csv));
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
