import { CursorMode } from "~/pkg/forge";
import { store } from "~/store";
import styles from "./cursor.module.css";
import { ToolbarMenu, ToolbarMenuItem } from "./toolbar-menu";

export function Cursor() {
  const { cursorMode, setCursorMode } = store;

  const handleSelectCursorMode = (mode: CursorMode, close: () => void) => {
    close();
    setCursorMode(mode);
  };

  return (
    <ToolbarMenu
      button={
        <button class={styles.cursor}>
          {cursorMode() === CursorMode.Insert ? (
            <img class={styles.icon} src="/cursor.svg" alt="" />
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
      }
      menu={(close) => (
        <>
          <ToolbarMenuItem
            onClick={() => handleSelectCursorMode(CursorMode.Insert, close)}
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
          </ToolbarMenuItem>
          <ToolbarMenuItem
            onClick={() => handleSelectCursorMode(CursorMode.Remove, close)}
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
          </ToolbarMenuItem>
        </>
      )}
    />
  );
}
