import { defineConfig, type Plugin } from 'vite'
import react from '@vitejs/plugin-react'

/** Strip data-testid attributes from JSX in production builds. */
function stripTestIds(): Plugin {
  return {
    name: 'strip-test-ids',
    enforce: 'pre',
    transform(code, id) {
      if (!/\.(tsx?|jsx?)$/.test(id)) return;
      // Remove data-testid="..." (simple quoted strings)
      let out = code.replace(/\s+data-testid="[^"]*"/g, '');
      // Remove data-testid={...} — use depth tracking so nested ${...}
      // inside template literals are handled correctly.
      const attr = 'data-testid={';
      let idx = out.indexOf(attr);
      while (idx !== -1) {
        // include any leading whitespace
        let start = idx;
        while (start > 0 && (out[start - 1] === ' ' || out[start - 1] === '\n')) start--;
        // find the matching closing brace
        let depth = 1;
        let end = idx + attr.length;
        while (end < out.length && depth > 0) {
          if (out[end] === '{') depth++;
          else if (out[end] === '}') depth--;
          end++;
        }
        out = out.slice(0, start) + out.slice(end);
        idx = out.indexOf(attr);
      }
      return out;
    },
  };
}


// https://vite.dev/config/
export default defineConfig(({ mode }) => ({
  plugins: [
    react(),
    ...(mode === 'production' ? [stripTestIds()] : []),
  ],
  // Base path must match the Axum mount point so asset URLs resolve correctly
  base: '/',
  build: {
    outDir: 'dist',
  },
  server: {
    // Proxy API + emulator routes to the running emulator process in dev.
    // Set VITE_EMULATOR_PORT if your emulator uses a non-default port.
    proxy: {
      '/v1': { target: `http://localhost:${process.env.VITE_EMULATOR_PORT ?? 4600}`, changeOrigin: true },
      '/v2': { target: `http://localhost:${process.env.VITE_EMULATOR_PORT ?? 4600}`, changeOrigin: true },
      '/emulator': { target: `http://localhost:${process.env.VITE_EMULATOR_PORT ?? 4600}`, changeOrigin: true },
      '/health': { target: `http://localhost:${process.env.VITE_EMULATOR_PORT ?? 4600}`, changeOrigin: true },
    },
  },
}))
