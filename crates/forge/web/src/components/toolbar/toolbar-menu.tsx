import { createSignal, onCleanup, onMount, type JSX } from "solid-js";
import styles from "./toolbar-menu.module.css";

interface ToolbarMenuProps {
  button: JSX.Element;
  menu: (close: () => void) => JSX.Element;
}

export function ToolbarMenu(props: ToolbarMenuProps) {
  let rootRef: HTMLDivElement | undefined;
  const [isOpen, setIsOpen] = createSignal(false);

  const close = () => setIsOpen(false);

  onMount(() => {
    const handleDocumentClick = (event: MouseEvent) => {
      if (!rootRef?.contains(event.target as Node)) {
        close();
      }
    };

    document.addEventListener("click", handleDocumentClick);
    onCleanup(() => document.removeEventListener("click", handleDocumentClick));
  });

  return (
    <div ref={rootRef}>
      <div class={styles.select} hidden={!isOpen()}>
        {props.menu(close)}
      </div>

      <div class={styles.content}>
        {props.button}

        <button
          type="button"
          class={`${styles.dropdown} ${isOpen() ? styles.dropdownSelected : ""}`}
          onClick={() => setIsOpen((open) => !open)}
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

type ToolbarMenuItemProps = JSX.ButtonHTMLAttributes<HTMLButtonElement>;

export function ToolbarMenuItem(props: ToolbarMenuItemProps) {
  return <button {...props} type="button" class={styles.inputGroup} />;
}

interface ToolbarSubmenuProps {
  button: JSX.Element;
  menu: JSX.Element;
}

export function ToolbarSubmenu(props: ToolbarSubmenuProps) {
  return (
    <div class={styles.submenu}>
      <button type="button" class={styles.inputGroup}>
        <span>{props.button}</span>
        <span class={styles.submenuArrow}>
          <svg width="16" height="16" fill="none" viewBox="4 4 16 16">
            <path
              fill="#fff"
              fill-rule="evenodd"
              d="M11.146 9.646a.5.5 0 0 1 .708 0l2 2a.5.5 0 0 1 0 .708l-2 2a.5.5 0 0 1-.708-.708L12.793 12l-1.647-1.646a.5.5 0 0 1 0-.708"
              clip-rule="evenodd"
            ></path>
          </svg>
        </span>
      </button>

      <div class={styles.submenuContent}>{props.menu}</div>
    </div>
  );
}
