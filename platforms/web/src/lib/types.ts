export type BlockType = 'formatInlineCode';
export type WysiwygInputEvent = InputEvent & { inputType: InputEvent['inputType'] | BlockType };
