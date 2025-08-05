import path from 'path';
import fs from 'fs';
import { defineConfig } from 'vite';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default {
  root: "src",
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        index: path.resolve(__dirname, 'src/index.html'),
        view: path.resolve(__dirname, 'src/view.html')
      }
    }
  }
}
