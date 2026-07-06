import styles from "./cursor.module.css";

export function Cursor() {
  return (
    <button class={styles.root}>
      <img class={styles.icon} src="/cursor.svg" alt="" aria-hidden="true" />
    </button>
  );
}
