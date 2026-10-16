import { defineConfig } from "vite";

export default defineConfig({
	build: {
		manifest: true,
		outDir: "static",
		rollupOptions: {
			input: "assets/index.ts",
		},
	},
});
