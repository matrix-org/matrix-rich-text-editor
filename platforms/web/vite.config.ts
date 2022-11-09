/*
Copyright 2022 The Matrix.org Foundation C.I.C.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';
import dts from 'vite-plugin-dts';

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        react(),
        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        dts({
            include: ['lib/useWysiwyg.ts', 'lib/types.ts', 'lib/constants.ts'],
            rollupTypes: true,
        }),
    ],
    test: {
        globals: true,
        environment: 'jsdom',
        setupFiles: 'test.setup.ts',
        includeSource: ['lib/**/*.{ts,tsx}'],
        coverage: {
            all: true,
            include: ['lib/**/*.{ts,tsx}'],
            exclude: [
                'lib/testUtils/**/*.{ts,tsx}',
                'lib/**/*test.{ts,tsx}',
                'lib/**/*.d.ts',
            ],
        },
    },
    build: {
        lib: {
            entry: resolve(__dirname, 'lib/useWysiwyg.ts'),
            name: 'Matrix wysiwyg',
            // the proper extensions will be added
            fileName: 'matrix-wysiwyg',
        },
        rollupOptions: {
            // make sure to externalize deps that shouldn't be bundled
            // into your library
            external: ['react', 'react-dom'],
            output: {
                // Provide global variables to use in the UMD build
                // for externalized deps
                globals: {
                    'react': 'React',
                    'react-dom': 'ReactDOM',
                },
            },
        },
    },
});
