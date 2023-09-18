package io.element.wysiwyg.compose.ui.components

import androidx.annotation.DrawableRes
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import io.element.wysiwyg.compose.R
import kotlinx.collections.immutable.ImmutableMap
import kotlinx.collections.immutable.persistentMapOf
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction

@Composable
fun FormattingButtons(
    actionStates: ImmutableMap<ComposerAction, ActionState>,
    modifier: Modifier = Modifier,
    onResetText: () -> Unit = {},
    onActionClick: (ComposerAction) -> Unit = {},
) {
    val rowArrangement = Arrangement.spacedBy(2.dp)
    val buttonArrangement = Arrangement.spacedBy(4.dp)

    Column(
        modifier = modifier,
        verticalArrangement = rowArrangement
    ) {
        val scrollState = rememberScrollState()

        Row(
            horizontalArrangement = buttonArrangement,
            modifier = Modifier
                .horizontalScroll(scrollState)
        ) {
            FormattingButton(
                contentDescription = "Bold",
                icon = R.drawable.ic_format_bold,
                onClick = { onActionClick(ComposerAction.BOLD) },
                actionState = actionStates.getOrDefault(
                    ComposerAction.BOLD,
                    ActionState.DISABLED
                )
            )
            FormattingButton(
                contentDescription = "Italic",
                icon = R.drawable.ic_format_italic,
                onClick = { onActionClick(ComposerAction.ITALIC) },
                actionState = actionStates.getOrDefault(
                    ComposerAction.ITALIC,
                    ActionState.DISABLED
                )
            )
            FormattingButton(
                contentDescription = "Underline",
                icon = R.drawable.ic_format_underline,
                onClick = { onActionClick(ComposerAction.UNDERLINE) },
                actionState = actionStates.getOrDefault(
                    ComposerAction.UNDERLINE,
                    ActionState.DISABLED
                )
            )
            FormattingButton(
                contentDescription = "Strike through",
                icon = R.drawable.ic_format_strikethrough,
                onClick = { onActionClick(ComposerAction.STRIKE_THROUGH) },
                actionState = actionStates.getOrDefault(
                    ComposerAction.STRIKE_THROUGH,
                    ActionState.DISABLED
                )
            )
            FormattingButton(
                contentDescription = "Inline code",
                icon = R.drawable.ic_code,
                onClick = { onActionClick(ComposerAction.INLINE_CODE) },
                actionState = actionStates.getOrDefault(
                    ComposerAction.INLINE_CODE,
                    ActionState.DISABLED
                )
            )
            FormattingButton(
                contentDescription = "Code block",
                icon = R.drawable.ic_code_block,
                onClick = { onActionClick(ComposerAction.CODE_BLOCK) },
                actionState = actionStates.getOrDefault(
                    ComposerAction.CODE_BLOCK,
                    ActionState.DISABLED
                )
            )
            FormattingButton(
                contentDescription = "Quote",
                icon = R.drawable.ic_quote,
                onClick = { onActionClick(ComposerAction.QUOTE) },
                actionState = actionStates.getOrDefault(
                    ComposerAction.QUOTE,
                    ActionState.DISABLED
                )
            )
            FormattingButton(
                contentDescription = "Link",
                icon = R.drawable.ic_link,
                onClick = { onActionClick(ComposerAction.LINK) },
                actionState = actionStates.getOrDefault(
                    ComposerAction.LINK,
                    ActionState.DISABLED
                )
            )
            FormattingButton(
                contentDescription = "Ordered list",
                icon = R.drawable.ic_ordered_list,
                onClick = { onActionClick(ComposerAction.ORDERED_LIST) },
                actionState = actionStates.getOrDefault(
                    ComposerAction.ORDERED_LIST,
                    ActionState.DISABLED
                )
            )
            FormattingButton(
                contentDescription = "Unordered list",
                icon = R.drawable.ic_unordered_list,
                onClick = { onActionClick(ComposerAction.UNORDERED_LIST) },
                actionState = actionStates.getOrDefault(
                    ComposerAction.UNORDERED_LIST,
                    ActionState.DISABLED
                )
            )
            FormattingButton(
                contentDescription = "Indent",
                icon = R.drawable.ic_indent,
                onClick = { onActionClick(ComposerAction.INDENT) },
                actionState = actionStates.getOrDefault(
                    ComposerAction.INDENT,
                    ActionState.DISABLED
                )
            )
            FormattingButton(
                contentDescription = "Unindent",
                icon = R.drawable.ic_unindent,
                onClick = { onActionClick(ComposerAction.UNINDENT) },
                actionState = actionStates.getOrDefault(
                    ComposerAction.UNINDENT,
                    ActionState.DISABLED
                )
            )
        }
        Row(
            horizontalArrangement = buttonArrangement
        ) {
            FormattingButton(
                contentDescription = "Undo",
                icon = R.drawable.ic_undo,
                onClick = { onActionClick(ComposerAction.UNDO) },
                actionState = actionStates.getOrDefault(
                    ComposerAction.UNDO,
                    ActionState.DISABLED
                )
            )
            FormattingButton(
                contentDescription = "Redo",
                icon = R.drawable.ic_redo,
                onClick = { onActionClick(ComposerAction.REDO) },
                actionState = actionStates.getOrDefault(
                    ComposerAction.REDO,
                    ActionState.DISABLED
                )
            )
            TextButton(
                onClick = onResetText
            ) {
                Text("Reset")
            }
        }
    }
}


@Composable
private fun FormattingButton(
    contentDescription: String,
    @DrawableRes
    icon: Int,
    onClick: () -> Unit,
    actionState: ActionState,
) {
    val selectedBgColor = MaterialTheme.colorScheme.primary
    val selectedTextColor = MaterialTheme.colorScheme.onPrimary
    IconButton(
        onClick = onClick,
        enabled = actionState != ActionState.DISABLED,
        colors = if (actionState == ActionState.REVERSED) {
            IconButtonDefaults.iconButtonColors(
                containerColor = selectedBgColor,
                contentColor = selectedTextColor,
            )
        } else IconButtonDefaults.iconButtonColors(),
    ) {
        Icon(
            painter = painterResource(id = icon),
            contentDescription = contentDescription,
        )
    }
}

@Preview
@Composable
private fun FormattingButtonsPreview() =
    FormattingButtons(
        actionStates = persistentMapOf(
            ComposerAction.BOLD to ActionState.ENABLED,
            ComposerAction.ITALIC to ActionState.REVERSED,
            ComposerAction.UNDERLINE to ActionState.DISABLED,
        )
    )

@Preview
@Composable
private fun FormattingButtonsDefaultPreview() =
    FormattingButtons(
        actionStates = persistentMapOf()
    )

