import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

const apiTarget = process.env.VITE_API_PROXY_TARGET ?? 'http://localhost:3004';
const appBase = '/board/';

export default defineConfig({
  base: appBase,
  plugins: [react()],
  envPrefix: 'VITE_',
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
  server: {
    port: 5177,
    proxy: {
      [`${appBase}api`]: {
        target: apiTarget,
        changeOrigin: true,
        ws: true,
        rewrite: (path) => path.replace(/^\/board/, ''),
      },
      [`${appBase}health`]: {
        target: apiTarget,
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/board/, ''),
      },
      [`${appBase}version`]: {
        target: apiTarget,
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/board/, ''),
      },
    },
  },
});
