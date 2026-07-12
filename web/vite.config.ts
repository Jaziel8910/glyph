import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const base = process.env.GITHUB_PAGES_BASE || "/glyph/";

export default defineConfig({
  base,
  plugins: [react()],
});
