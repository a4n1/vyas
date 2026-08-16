import { solidStart } from "@solidjs/start/config";
import { nitroV2Plugin as nitro } from "@solidjs/vite-plugin-nitro-2";
import { defineConfig } from "vite";
import topLevelAwait from "vite-plugin-top-level-await";
import wasm from "vite-plugin-wasm";

export default defineConfig({
  plugins: [
    solidStart(),
    topLevelAwait(),
    wasm(),
    {
      name: "reload-wasm",
      handleHotUpdate({ file, server }) {
        if (file.startsWith("pkg")) {
          server.ws.send({
            type: "full-reload",
          });

          return [];
        }
      },
    },
    nitro({
      prerender: {
        routes: ["/"],
        crawlLinks: true,
      },
    }),
  ],
});
