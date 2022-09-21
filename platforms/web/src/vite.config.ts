import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

// https://vitejs.dev/config/
export default defineConfig({
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    plugins: [react()],
    test: {
        globals: true,
        environment: 'jsdom',
        setupFiles: 'test.setup.ts',
        includeSource: ['src/**/*.{ts,tsx}'],
        coverage: {
            all: true,
            include: ['src/**/*.{ts,tsx}'],
        },
    },
});
