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

import { ACTION_TYPES } from './constants';

export type BlockType = InputEvent['inputType'] | 'formatInlineCode' | 'clear';

export type WysiwygInputEvent =
    | ClipboardEvent
    | (InputEvent & { inputType: BlockType });

export type ActionTypes = typeof ACTION_TYPES[number];

export type ActionState = 'enabled' | 'reversed' | 'disabled';

export type AllActionStates = Record<ActionTypes, ActionState>;

export type FormattingFunctions = Record<ActionTypes, () => void>;

export type Wysiwyg = {
    actions: FormattingFunctions;
    content: () => string;
};

export type InputEventProcessor = (
    event: WysiwygInputEvent,
    wysiwyg: Wysiwyg,
) => WysiwygInputEvent | null;
