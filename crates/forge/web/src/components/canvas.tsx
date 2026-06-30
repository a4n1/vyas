import { onMount } from "solid-js";

const init = await import("../pkg/forge.js");

export default function Canvas() {
  onMount(async () => {
    init.web();
  });

  return <canvas id="vyas" />;
}
