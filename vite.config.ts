import { defineConfig } from "vite";

export default defineConfig({
	build: {
		manifest: true,
		outDir: "static",
		rollupOptions: {
			input: {
				main: "assets/index.ts",
				styles: "assets/input.css"
			},
		},
	},
});
