export function onGridUpdate() {
  window.dispatchEvent(new CustomEvent("forge:grid-update"));
}
