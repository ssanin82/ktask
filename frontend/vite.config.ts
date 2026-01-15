import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    host: '0.0.0.0', // Allow access from network
    port: 3000,
    strictPort: false, // Try next available port if 3000 is taken
    open: false, // Don't auto-open browser
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:50051',
        changeOrigin: true,
      },
      '/ws': {
        target: 'ws://127.0.0.1:50051',
        ws: true,
      },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
  },
})
