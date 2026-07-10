import { Hsv, store } from "~/store";
import {
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  type JSX,
} from "solid-js";
import styles from "./color-picker.module.css";

export function ColorPicker() {
  const { color, setColor } = store;

  const [open, setOpen] = createSignal(false);

  let root: HTMLDivElement | undefined;

  const hex = createMemo(() => hsvToHex(color()));
  const rgb = createMemo(() => hsvToRgb(color()));
  const solidColor = createMemo(
    () => `rgb(${rgb().red}, ${rgb().green}, ${rgb().blue})`,
  );

  const updateFromAreaPointer = (element: HTMLElement, event: PointerEvent) => {
    const rect = element.getBoundingClientRect();
    const s = clamp((event.clientX - rect.left) / rect.width);
    const v = 1 - clamp((event.clientY - rect.top) / rect.height);
    setColor((current) => ({ ...current, s, v }));
  };

  const updateFromHuePointer = (element: HTMLElement, event: PointerEvent) => {
    const rect = element.getBoundingClientRect();
    setColor((current) => ({
      ...current,
      h: clamp((event.clientX - rect.left) / rect.width) * 360,
    }));
  };

  const drag = (update: (element: HTMLElement, event: PointerEvent) => void) =>
    ((event) => {
      const element = event.currentTarget;

      event.preventDefault();
      element.setPointerCapture(event.pointerId);
      update(element, event);

      const move = (moveEvent: PointerEvent) => update(element, moveEvent);
      const stop = () => {
        if (element.hasPointerCapture(event.pointerId)) {
          element.releasePointerCapture(event.pointerId);
        }
        window.removeEventListener("pointermove", move);
        window.removeEventListener("pointerup", stop);
      };

      window.addEventListener("pointermove", move);
      window.addEventListener("pointerup", stop);
    }) satisfies JSX.EventHandler<HTMLElement, PointerEvent>;

  const handleHexInput: JSX.EventHandler<HTMLInputElement, InputEvent> = (
    event,
  ) => {
    const next = hexToHsv(event.currentTarget.value);
    if (next) setColor(next);
  };

  createEffect(() => {
    if (!open()) return;

    const closeOnOutsidePointer = (event: PointerEvent) => {
      if (!root?.contains(event.target as Node)) setOpen(false);
    };

    document.addEventListener("pointerdown", closeOnOutsidePointer);
    onCleanup(() =>
      document.removeEventListener("pointerdown", closeOnOutsidePointer),
    );
  });

  return (
    <div ref={root} class={styles.root}>
      <button
        type="button"
        class={`${styles.button} ${open() ? styles.buttonOpen : ""}`}
        onClick={() => setOpen((current) => !current)}
      >
        <span
          class={styles.buttonSwatch}
          style={{ "background-color": solidColor() }}
        />
      </button>

      <div class={styles.positioner} hidden={!open()}>
        <div class={styles.content}>
          <div
            class={styles.area}
            style={{ "--hue-color": `hsl(${color().h}deg 100% 50%)` }}
            onPointerDown={drag(updateFromAreaPointer)}
          >
            <div
              class={styles.areaThumb}
              style={{
                left: `${color().s * 100}%`,
                top: `${(1 - color().v) * 100}%`,
                "background-color": solidColor(),
              }}
            />
          </div>

          <div class={styles.slider} onPointerDown={drag(updateFromHuePointer)}>
            <div class={styles.hueTrack} />
            <div
              class={styles.sliderThumb}
              style={{ left: `${color().h / 3.6}%` }}
            />
          </div>

          <div class={styles.inputRow}>
            <input
              class={styles.hexInput}
              value={hex()}
              onInput={handleHexInput}
            />
          </div>
        </div>
      </div>
    </div>
  );
}

function clamp(value: number, min = 0, max = 1) {
  return Math.min(max, Math.max(min, value));
}

function componentToHex(value: number) {
  return Math.round(value).toString(16).padStart(2, "0").toUpperCase();
}

export function hsvToRgb({ h, s, v }: Hsv) {
  const chroma = v * s;
  const hue = h / 60;
  const x = chroma * (1 - Math.abs((hue % 2) - 1));
  const match = v - chroma;

  let red = 0;
  let green = 0;
  let blue = 0;

  if (hue >= 0 && hue < 1) [red, green, blue] = [chroma, x, 0];
  else if (hue < 2) [red, green, blue] = [x, chroma, 0];
  else if (hue < 3) [red, green, blue] = [0, chroma, x];
  else if (hue < 4) [red, green, blue] = [0, x, chroma];
  else if (hue < 5) [red, green, blue] = [x, 0, chroma];
  else [red, green, blue] = [chroma, 0, x];

  return {
    red: Math.round((red + match) * 255),
    green: Math.round((green + match) * 255),
    blue: Math.round((blue + match) * 255),
  };
}

function rgbToHsv(red: number, green: number, blue: number): Hsv {
  const r = red / 255;
  const g = green / 255;
  const b = blue / 255;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const delta = max - min;

  let h = 0;

  if (delta !== 0) {
    if (max === r) h = 60 * (((g - b) / delta) % 6);
    else if (max === g) h = 60 * ((b - r) / delta + 2);
    else h = 60 * ((r - g) / delta + 4);
  }

  return {
    h: h < 0 ? h + 360 : h,
    s: max === 0 ? 0 : delta / max,
    v: max,
  };
}

function hsvToHex(hsv: Hsv) {
  const { red, green, blue } = hsvToRgb(hsv);
  return `#${componentToHex(red)}${componentToHex(green)}${componentToHex(blue)}`;
}

function hexToHsv(value: string) {
  const match = value.match(/^#?([0-9a-f]{6})$/i);
  if (!match) return null;

  const color = match[1];
  return rgbToHsv(
    Number.parseInt(color.slice(0, 2), 16),
    Number.parseInt(color.slice(2, 4), 16),
    Number.parseInt(color.slice(4, 6), 16),
  );
}
