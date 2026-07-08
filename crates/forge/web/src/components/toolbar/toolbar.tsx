import { ColorPicker } from "./color-picker";
import { Cursor } from "./cursor";
import { Save } from "./save";
import styles from "./toolbar.module.css";

export function Toolbar() {
  return (
    <div class={styles.root}>
      <Cursor />
      <ColorPicker />
      <Save />
    </div>
  );
}
