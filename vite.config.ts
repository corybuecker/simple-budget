import { defineConfig } from 'vite'
import { resolve } from 'path'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
    plugins: [tailwindcss()],
    build: {
        outDir: 'static',
        lib: {
            entry: resolve(__dirname, 'assets/src/index.ts'),
            name: 'SimpleBudget',
            fileName: 'index',
            formats: ['es'],
        },
    },
})