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

import { SuggestionPattern } from '../generated/wysiwyg';
import { SUGGESTIONS } from './constants';
import { MappedSuggestion, SuggestionChar, SuggestionType } from './types';

export function getSuggestionChar(
    suggestion: SuggestionPattern,
): SuggestionChar {
    console.log('getSuggestionChar');
    console.log(suggestion);
    console.log(suggestion.key);
    console.log(suggestion.key.key_type);
    var a = SUGGESTIONS[suggestion.key.key_type] || '';
    console.log(`a: ${a}`);
    return a;
}

export function getSuggestionType(
    suggestion: SuggestionPattern,
): SuggestionType {
    switch (suggestion.key.key_type) {
        case 0:
        case 1:
            return 'mention';
        case 2:
            return 'command';
        case 3:
            return 'custom';
        default:
            return 'unknown';
    }
}

export function mapSuggestion(
    suggestion: SuggestionPattern | null,
): MappedSuggestion | null {
    if (suggestion === null) return suggestion;
    return {
        text: suggestion.text,
        keyChar: getSuggestionChar(suggestion),
        type: getSuggestionType(suggestion),
    };
}
