import { ColorPicker } from "./color-picker";
import { Cursor } from "./cursor";
import styles from "./toolbar.module.css";

export function Toolbar() {
  return (
    <div class={styles.root}>
      <Cursor />
      <ColorPicker />
    </div>
  );
}
