package io.element.android.wysiwyg.compose.testutils

import io.element.android.wysiwyg.compose.testutils.ComposerActions.DEFAULT_ACTIONS
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction

object ComposerActions {
    val DEFAULT_ACTIONS =
        mapOf(
            ComposerAction.INDENT to ActionState.DISABLED,
            ComposerAction.STRIKE_THROUGH to ActionState.ENABLED,
            ComposerAction.UNORDERED_LIST to ActionState.ENABLED,
            ComposerAction.ORDERED_LIST to ActionState.ENABLED,
            ComposerAction.ITALIC to ActionState.ENABLED,
            ComposerAction.UNDO to ActionState.DISABLED,
            ComposerAction.QUOTE to ActionState.ENABLED,
            ComposerAction.UNDERLINE to ActionState.ENABLED,
            ComposerAction.REDO to ActionState.DISABLED,
            ComposerAction.BOLD to ActionState.ENABLED,
            ComposerAction.LINK to ActionState.ENABLED,
            ComposerAction.INLINE_CODE to ActionState.ENABLED,
            ComposerAction.CODE_BLOCK to ActionState.ENABLED,
            ComposerAction.UNINDENT to ActionState.DISABLED
        )
}

fun Map<ComposerAction, ActionState>.copy(
    newEntries: Map<ComposerAction, ActionState>
) = DEFAULT_ACTIONS.mapValues {
    newEntries[it.key] ?: it.value
}
