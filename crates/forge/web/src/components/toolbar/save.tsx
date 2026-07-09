import { store } from "~/store";
import { createSignal, onCleanup, onMount } from "solid-js";
import styles from "./save.module.css";

export function Save() {
  const { forge } = store;

  const [isSelectVisible, setIsSelectVisible] = createSignal(false);

  let rootRef: HTMLDivElement | undefined;
  let fileInputRef: HTMLInputElement | undefined;

  const handleSave = () => {
    const grid: Grid | undefined = forge()?.export_grid();
    if (!grid) {
      return;
    }

    saveGrid(grid);
  };

  const handleLoadClick = () => {
    setIsSelectVisible(false);
    fileInputRef?.click();
  };

  const handleLoad = async (event: Event) => {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];

    if (!file) {
      return;
    }

    const bytes = new Uint8Array(await file.arrayBuffer());
    forge()?.load_grid(bytes);
    input.value = "";
  };

  onMount(() => {
    const handleDocumentClick = (event: MouseEvent) => {
      if (!rootRef?.contains(event.target as Node)) {
        setIsSelectVisible(false);
      }
    };

    document.addEventListener("click", handleDocumentClick);
    onCleanup(() => document.removeEventListener("click", handleDocumentClick));
  });

  return (
    <div ref={rootRef}>
      <input
        ref={fileInputRef}
        type="file"
        accept=".csv,text/csv"
        hidden
        onchange={handleLoad}
      />

      <div class={styles.select} hidden={!isSelectVisible()}>
        <button class={styles.inputGroup} onclick={handleLoadClick}>
          <span>Load</span>
        </button>
      </div>

      <div class={styles.content}>
        <button class={styles.save} onclick={handleSave}>
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path d="M2 15V23H22V15" stroke="white" stroke-width="2" />
            <path
              d="M16.5 11.5L12.0021 15.75L7.5 11.5"
              stroke="white"
              stroke-width="1.5"
            />
            <path d="M12 15L12 3.5" stroke="white" stroke-width="1.5" />
          </svg>
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

type Grid = Map<Position, Voxel>;

interface Position {
  x: number;
  y: number;
  z: number;
}

interface Voxel {
  color: Color;
}

interface Color {
  Srgb: Srgb;
}

interface Srgb {
  r: number;
  g: number;
  b: number;
}

function saveGrid(grid: Grid) {
  const content: string[] = [];

  content.push("x,y,z,r,g,b");

  const entries = Array.from(grid).sort(
    ([a], [b]) => a.x - b.x || a.y - b.y || a.z - b.z,
  );

  for (const [position, voxel] of entries) {
    const color = voxel.color.Srgb;

    content.push(
      `${position.x},${position.y},${position.z},${color.r},${color.g},${color.b}`,
    );
  }

  const csv = content.join("\n");

  const blob = new Blob([csv], {
    type: "text/csv;charset=utf-8",
  });

  const url = URL.createObjectURL(blob);

  const a = document.createElement("a");
  a.href = url;
  const filename = prompt("File name");

  if (filename === null || filename === "") {
    return;
  }

  a.download = `${filename}.csv`;
  a.click();
  a.remove();

  URL.revokeObjectURL(url);
}
