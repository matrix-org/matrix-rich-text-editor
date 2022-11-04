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

// eslint-disable-next-line camelcase
import init, { new_composer_model } from '../../generated/wysiwyg';
import { getFormattingState } from './event';

beforeAll(async () => {
    await init();
});

describe('getFormattingState', () => {
    it('Should be a map of action to state', () => {
        // Given
        const model = new_composer_model();
        const menuStateUpdate = model.bold().menu_state().update();

        // When
        if (!menuStateUpdate) {
            fail('There should be an update!');
        }
        const states = getFormattingState(menuStateUpdate);

        // Then
        expect(states.italic).toBe('enabled');
        expect(states.bold).toBe('reversed');
        expect(states.redo).toBe('disabled');
    });
});
