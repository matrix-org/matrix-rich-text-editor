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

import { LinkEvent, SuggestionEvent } from './types';

export function isInputEvent(e: Event): e is InputEvent {
    return 'inputType' in e;
}

export function isClipboardEvent(e: Event): e is ClipboardEvent {
    return 'clipboardData' in e;
}

export function isSuggestionEvent(e: Event): e is SuggestionEvent {
    return isInputEvent(e) && e.inputType === 'insertSuggestion';
}

export function isLinkEvent(e: Event): e is LinkEvent {
    return isInputEvent(e) && e.inputType == 'insertLink';
}
