import { set_color as setColor } from "~/pkg/forge.js";
import {
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  type JSX,
} from "solid-js";
import styles from "./color-picker.module.css";

type Hsv = {
  h: number;
  s: number;
  v: number;
};

const clamp = (value: number, min = 0, max = 1) =>
  Math.min(max, Math.max(min, value));

const componentToHex = (value: number) =>
  Math.round(value).toString(16).padStart(2, "0").toUpperCase();

const hsvToRgb = ({ h, s, v }: Hsv) => {
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
};

const rgbToHsv = (red: number, green: number, blue: number): Hsv => {
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
};

const hsvToHex = (hsv: Hsv) => {
  const { red, green, blue } = hsvToRgb(hsv);
  return `#${componentToHex(red)}${componentToHex(green)}${componentToHex(blue)}`;
};

const hexToHsv = (value: string) => {
  const match = value.match(/^#?([0-9a-f]{6})$/i);
  if (!match) return null;

  const color = match[1];
  return rgbToHsv(
    Number.parseInt(color.slice(0, 2), 16),
    Number.parseInt(color.slice(2, 4), 16),
    Number.parseInt(color.slice(4, 6), 16),
  );
};

const hexColorToU32 = (hex: string): number => {
  hex = hex.replace(/^#/, "");

  const r = parseInt(hex.slice(0, 2), 16);
  const g = parseInt(hex.slice(2, 4), 16);
  const b = parseInt(hex.slice(4, 6), 16);

  return ((0 << 24) | (r << 16) | (g << 8) | b) >>> 0;
};

export function ColorPicker() {
  const [open, setOpen] = createSignal(false);
  const [hsv, setHsv] = createSignal<Hsv>(hexToHsv("#000000")!);
  let root: HTMLDivElement | undefined;

  const hex = createMemo(() => hsvToHex(hsv()));
  const rgb = createMemo(() => hsvToRgb(hsv()));
  const solidColor = createMemo(
    () => `rgb(${rgb().red}, ${rgb().green}, ${rgb().blue})`,
  );

  const updateFromAreaPointer = (element: HTMLElement, event: PointerEvent) => {
    const rect = element.getBoundingClientRect();
    const s = clamp((event.clientX - rect.left) / rect.width);
    const v = 1 - clamp((event.clientY - rect.top) / rect.height);
    setHsv((current) => ({ ...current, s, v }));
  };

  const updateFromHuePointer = (element: HTMLElement, event: PointerEvent) => {
    const rect = element.getBoundingClientRect();
    setHsv((current) => ({
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
    if (next) setHsv(next);
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

  createEffect(() => {
    const color = hexColorToU32(hex());
    setColor(color);
  });

  return (
    <div ref={root} class={styles.root}>
      <button
        type="button"
        class={styles.trigger}
        aria-label={`Select color, currently ${hex()}`}
        aria-expanded={open()}
        onClick={() => setOpen((current) => !current)}
      >
        <span
          class={styles.triggerSwatch}
          style={{ "background-color": solidColor() }}
        />
      </button>

      <div class={styles.positioner} hidden={!open()}>
        <div class={styles.content} role="dialog" aria-label="Select color">
          <div
            class={styles.area}
            style={{ "--hue-color": `hsl(${hsv().h}deg 100% 50%)` }}
            onPointerDown={drag(updateFromAreaPointer)}
          >
            <div
              class={styles.areaThumb}
              style={{
                left: `${hsv().s * 100}%`,
                top: `${(1 - hsv().v) * 100}%`,
                "background-color": solidColor(),
              }}
            />
          </div>

          <div class={styles.slider} onPointerDown={drag(updateFromHuePointer)}>
            <div class={styles.hueTrack} />
            <div
              class={styles.sliderThumb}
              style={{ left: `${hsv().h / 3.6}%` }}
            />
          </div>

          <div class={styles.inputRow}>
            <input
              class={styles.hexInput}
              aria-label="Hex color"
              value={hex()}
              onInput={handleHexInput}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
