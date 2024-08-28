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
// eslint-disable-next-line camelcase
import init, {
    new_composer_model,
    SuggestionPattern,
} from '../generated/wysiwyg';
import { SUGGESTIONS } from './constants';
import {
    getSuggestionChar,
    getSuggestionType,
    mapSuggestion,
} from './suggestion';

beforeAll(async () => {
    await init();
});

describe('getSuggestionChar', () => {
    it('returns the expected character', () => {
        SUGGESTIONS.forEach((suggestionCharacter, index) => {
            const suggestion = {
                key: { key_type: index },
            } as unknown as SuggestionPattern;
            expect(getSuggestionChar(suggestion)).toBe(suggestionCharacter);
        });
    });

    it('returns empty string if given index is too high', () => {
        const suggestion = { key: 200 } as unknown as SuggestionPattern;
        expect(getSuggestionChar(suggestion)).toBe('');
    });
});

describe('getSuggestionType', () => {
    it('returns the expected type for a user or room mention', () => {
        const userSuggestion = {
            key: { key_type: 0 },
        } as unknown as SuggestionPattern;
        const roomSuggestion = {
            key: { key_type: 1 },
        } as unknown as SuggestionPattern;

        expect(getSuggestionType(userSuggestion)).toBe('mention');
        expect(getSuggestionType(roomSuggestion)).toBe('mention');
    });

    it('returns the expected type for a slash command', () => {
        const slashSuggestion = {
            key: { key_type: 2 },
        } as unknown as SuggestionPattern;

        expect(getSuggestionType(slashSuggestion)).toBe('command');
    });

    it('returns unknown for any other implementations', () => {
        const slashSuggestion = { key: 200 } as unknown as SuggestionPattern;

        expect(getSuggestionType(slashSuggestion)).toBe('unknown');
    });
});

describe('mapSuggestion', () => {
    it('returns null when passed null', () => {
        expect(mapSuggestion(null)).toBe(null);
    });

    it('returns the input with additional keys keyChar and type', () => {
        const suggestion: SuggestionPattern = {
            free: () => {},
            start: 1,
            end: 2,
            key: {
                free: () => {},
                key_type: 0,
                custom_key_value: undefined,
            },
            text: 'some text',
        };

        const mappedSuggestion = mapSuggestion(suggestion);
        expect(mappedSuggestion).toMatchObject({
            keyChar: '@',
            type: 'mention',
            text: suggestion.text,
        });
    });
});

describe('suggestionPattern', () => {
    it('Content should be encoded', () => {
        // Given
        const model = new_composer_model();
        model.replace_text('hello ');
        const update = model.replace_text('@alic');
        const suggestion = update.menu_action().suggestion();

        // When
        if (!suggestion) {
            fail('There should be an suggestion!');
        }

        model.insert_mention_at_suggestion(
            'https://matrix.to/#/@alice:matrix.org',
            ':D</a> a broken mention!',
            suggestion.suggestion_pattern,
            new Map(),
        );

        // Then
        expect(model.get_content_as_html()).toBe(
            'hello <a data-mention-type="user" href="https://matrix.to/#/@alice:matrix.org" contenteditable="false">:D&lt;&#x2F;a&gt; a broken mention!</a>\u{a0}',
        );
    });
});
