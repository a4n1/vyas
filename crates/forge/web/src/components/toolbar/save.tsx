import { store } from "~/store";
import styles from "./save.module.css";
import { ToolbarMenu, ToolbarMenuItem, ToolbarSubmenu } from "./toolbar-menu";

export function Save() {
  const { forge } = store;

  let fileInputRef: HTMLInputElement | undefined;

  const handleSave = () => {
    const grid: Grid | undefined = forge()?.export_grid();
    if (!grid) {
      return;
    }

    saveGrid(grid);
  };

  const handleLoadClick = (close: () => void) => {
    close();
    fileInputRef?.click();
  };

  const handleClear = (close: () => void) => {
    close();
    forge()?.load_grid(new TextEncoder().encode("x,y,z,r,g,b"));
  };

  const handleLoadExample = async (path: string, close: () => void) => {
    close();
    const response = await fetch(path);
    const bytes = new Uint8Array(await response.arrayBuffer());
    forge()?.load_grid(bytes);
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

  return (
    <>
      <input
        ref={fileInputRef}
        type="file"
        accept=".csv,text/csv"
        hidden
        onchange={handleLoad}
      />

      <ToolbarMenu
        button={
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
        }
        menu={(close) => (
          <>
            <ToolbarMenuItem onClick={() => handleLoadClick(close)}>
              <span>Load</span>
            </ToolbarMenuItem>

            <ToolbarMenuItem onClick={() => handleClear(close)}>
              <span>Clear</span>
            </ToolbarMenuItem>

            <ToolbarSubmenu
              button="Examples"
              menu={
                <ToolbarMenuItem
                  onClick={() => handleLoadExample("/models/tree.csv", close)}
                >
                  <span>Tree</span>
                </ToolbarMenuItem>
              }
            />
          </>
        )}
      />
    </>
  );
}

export type Grid = Map<Position, Voxel>;

export interface Position {
  x: number;
  y: number;
  z: number;
}

export interface Voxel {
  color: Color;
}

export interface Color {
  Srgb: Srgb;
}

export interface Srgb {
  r: number;
  g: number;
  b: number;
}

export function serializeGrid(grid: Grid) {
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

  return csv;
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
