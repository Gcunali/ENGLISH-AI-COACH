import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    strictPort: true,
    host: '127.0.0.1',
  },
  test: {
    environment: 'node',
    pool: 'threads',
    fileParallelism: false,
    exclude: [
      '**/node_modules/**',
      '**/dist/**',
      '**/.backup-phase-*/**',
      '**/.artifacts-phase-*/**',
      '**/local-ai/**',
      '**/src-tauri/**',
      '**/src-tauri/target/**',
    ],
  },
})
