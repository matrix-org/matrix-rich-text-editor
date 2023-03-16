/*
Copyright 2023 The Matrix.org Foundation C.I.C.

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

import { render, screen, waitFor } from '@testing-library/react';

import { Editor } from './testUtils/Editor';

// TODO move this into useListeners folder - this doesn't seem to understand
// that we can use the vite globals
// incidentally, the includeSource part of config may be what is causing hanging

describe('mention behaviour', () => {
    beforeEach(async () => {
        render(<Editor />);
        await waitFor(() =>
            expect(screen.getByRole('textbox')).toHaveAttribute(
                'contentEditable',
                'true',
            ),
        );
    });
    it('can move selection around single mention', () => {
        screen.debug();
    });
});
