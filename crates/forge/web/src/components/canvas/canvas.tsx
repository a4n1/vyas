import { init } from "~/pkg/forge.js";
import { onMount } from "solid-js";
import styles from "./canvas.module.css";

export function Canvas() {
  onMount(async () => {
    init();
  });

  return <canvas id="vyas" class={styles.root} />;
}
