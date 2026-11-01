import { defineConfig } from "vite";

export default defineConfig({
  // Vite's native esbuild JSX transform keeps production builds deterministic
  // and avoids a second Babel pass over the single-file React workflow UI.
  esbuild: {
    jsx: "automatic",
  },
});
