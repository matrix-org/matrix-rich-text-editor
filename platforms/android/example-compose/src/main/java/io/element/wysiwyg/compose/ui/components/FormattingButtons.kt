package io.element.wysiwyg.compose.ui.components

import androidx.annotation.DrawableRes
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
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
import uniffi.wysiwyg_composer.ActionState
import uniffi.wysiwyg_composer.ComposerAction

@Composable
fun FormattingButtons(
    actionStates: Map<ComposerAction, ActionState>,
    modifier: Modifier = Modifier,
    onResetText: () -> Unit = {},
    onBoldClick: () -> Unit = {},
    onItalicClick: () -> Unit = {},
) {
    val rowArrangement = Arrangement.spacedBy(2.dp)
    val buttonArrangement = Arrangement.spacedBy(4.dp)

    Column(
        modifier = modifier,
        verticalArrangement = rowArrangement
    ) {
        Row(
            horizontalArrangement = buttonArrangement
        ) {
            FormattingButton(
                contentDescription = "Bold",
                icon = R.drawable.ic_format_bold,
                onClick = onBoldClick,
                actionState = actionStates.getOrDefault(ComposerAction.BOLD, ActionState.DISABLED)
            )
            FormattingButton(
                contentDescription = "Italic",
                icon = R.drawable.ic_format_italic,
                onClick = onItalicClick,
                actionState = actionStates.getOrDefault(ComposerAction.ITALIC, ActionState.DISABLED)
            )
        }
        Row(
            horizontalArrangement = buttonArrangement
        ) {
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
        actionStates = mapOf(
            ComposerAction.BOLD to ActionState.ENABLED,
            ComposerAction.ITALIC to ActionState.REVERSED,
            ComposerAction.UNDERLINE to ActionState.DISABLED,
        )
    )

@Preview
@Composable
private fun FormattingButtonsDefaultPreview() =
    FormattingButtons(
        actionStates = emptyMap()
    )

