package io.element.android.wysiwyg.compose.selection

import android.view.ActionMode
import android.widget.EditText
import androidx.compose.runtime.Immutable

/**
 * An action to be added to the selection context menu.
 *
 * @see [EditText.setCustomSelectionActionModeCallback] and [ActionMode.Callback]
 *
 * @param id A unique ID for the action.
 * @param title The title of the action which should be short.
 */
@Immutable
data class SelectionAction(
    val id: Int,
    val title: String,
)
