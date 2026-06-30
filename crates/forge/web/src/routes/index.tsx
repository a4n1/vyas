import { clientOnly } from "@solidjs/start";

export default function Home() {
  return <Canvas />;
}

const Canvas = clientOnly(() => import("~/components/canvas"), {
  lazy: false,
});
