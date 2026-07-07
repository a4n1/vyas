import { CursorMode } from "~/pkg/forge";
import { store } from "~/store";
import { createSignal } from "solid-js";
import styles from "./cursor.module.css";

export function Cursor() {
  const [isSelectVisible, setIsSelectVisible] = createSignal(false);
  const { cursorMode, setCursorMode } = store;

  const handleSelectCursorMode = (mode: CursorMode) => {
    setIsSelectVisible(false);
    setCursorMode(mode);
  };

  return (
    <div>
      <div class={styles.select} hidden={!isSelectVisible()}>
        <button
          class={styles.inputGroup}
          onclick={() => handleSelectCursorMode(CursorMode.Insert)}
        >
          <span
            class={
              cursorMode() === CursorMode.Insert
                ? styles.modeSelected
                : styles.modeUnselected
            }
          >
            ✓
          </span>
          <span>Insert</span>
        </button>
        <button
          class={styles.inputGroup}
          onclick={() => handleSelectCursorMode(CursorMode.Remove)}
        >
          <span
            class={
              cursorMode() === CursorMode.Remove
                ? styles.modeSelected
                : styles.modeUnselected
            }
          >
            ✓
          </span>
          <span>Remove</span>
        </button>
      </div>

      <div class={styles.content}>
        <button class={styles.cursor}>
          {cursorMode() === CursorMode.Insert ? (
            <img
              class={styles.icon}
              src="/cursor.svg"
              alt=""
              aria-hidden="true"
            />
          ) : (
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path d="M1 1L23 23M1 23L23 1" stroke="white" stroke-width="3" />
            </svg>
          )}
        </button>

        <button
          class={`${styles.dropdown} ${isSelectVisible() ? styles.dropdownSelected : ""}`}
          onclick={() => setIsSelectVisible((s) => !s)}
        >
          <svg width="16" height="16" fill="none" viewBox="4 4 16 16">
            <path
              fill="#fff"
              fill-rule="evenodd"
              d="M9.646 11.146a.5.5 0 0 1 .708 0L12 12.793l1.646-1.647a.5.5 0 0 1 .708.708l-2 2a.5.5 0 0 1-.708 0l-2-2a.5.5 0 0 1 0-.708"
              clip-rule="evenodd"
            ></path>
          </svg>
        </button>
      </div>
    </div>
  );
}
