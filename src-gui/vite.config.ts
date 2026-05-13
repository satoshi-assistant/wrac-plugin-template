import { defineConfig } from "vite";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

function readCargoVersion(): string {
  const cargoToml = readFileSync(resolve(__dirname, "../src-plugin/Cargo.toml"), "utf8");
  const versionMatch = cargoToml.match(/^\s*version\s*=\s*"([^"]+)"\s*$/m);
  if (!versionMatch) {
    throw new Error("src-plugin/Cargo.toml から version を取得できませんでした");
  }
  return versionMatch[1];
}

export default defineConfig({
  define: {
    __WRAC_GAIN_VERSION__: JSON.stringify(readCargoVersion()),
  },
  server: {
    // Debug plugin は WebView から 127.0.0.1 を読む。Vite の default `localhost`
    // だと環境によって IPv6 loopback だけに bind され、DAW 内 WebView の解決先と
    // ずれて黒画面になり得る。
    host: "127.0.0.1",
    port: 5173,
    strictPort: true,
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
});
