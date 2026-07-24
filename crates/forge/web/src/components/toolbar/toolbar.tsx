import type { JSX } from "solid-js";
import { ColorPicker } from "./color-picker";
import { Cursor } from "./cursor";
import { Save } from "./save";
import styles from "./toolbar.module.css";

export function Toolbar() {
  const focusCanvasAfterPointerClick: JSX.EventHandler<
    HTMLDivElement,
    MouseEvent
  > = (event) => {
    if (
      event.detail > 0 &&
      event.target instanceof Element &&
      !event.target.closest("input, textarea, select, [contenteditable='true']")
    ) {
      document.getElementById("vyas")?.focus();
    }
  };

  return (
    <div class={styles.root} onClick={focusCanvasAfterPointerClick}>
      <Cursor />
      <ColorPicker />
      <Save />
    </div>
  );
}
