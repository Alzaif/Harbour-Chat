import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

const apiTarget = process.env.VITE_API_PROXY_TARGET ?? 'http://localhost:3004';

export default defineConfig({
  plugins: [react()],
  envPrefix: 'VITE_',
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
  server: {
    port: 5177,
    proxy: {
      '/api': { target: apiTarget, changeOrigin: true, ws: true },
      '/health': { target: apiTarget, changeOrigin: true },
      '/version': { target: apiTarget, changeOrigin: true },
    },
  },
});
