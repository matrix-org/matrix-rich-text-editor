package io.element.android.wysiwyg.view.spans

import io.element.android.wysiwyg.display.KeywordDisplayHandler

/**
 * Used to override any [KeywordDisplayHandler] and force text to be plain.
 * This can be used, for example, inside a code block where text must be displayed as-is.
 */
internal interface PlainKeywordDisplaySpan