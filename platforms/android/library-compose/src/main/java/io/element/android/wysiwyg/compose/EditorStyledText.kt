package io.element.android.wysiwyg.compose

import android.text.Layout
import android.text.Spanned
import android.widget.TextView
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalInspectionMode
import androidx.compose.ui.viewinterop.AndroidView
import io.element.android.wysiwyg.EditorStyledTextView
import io.element.android.wysiwyg.compose.internal.applyStyleInCompose
import io.element.android.wysiwyg.compose.internal.rememberTypeface
import io.element.android.wysiwyg.compose.internal.toStyleConfig
import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.display.TextDisplay

/**
 * A composable EditorStyledText.
 * This composable is a wrapper around the [EditorStyledTextView] view.
 *
 * @param text The text to render.
 * If it's spanned it will be rendered as is, otherwise it will go first to HtmlConverter.
 * Your might want to use HtmlConverter before the rendering to avoid the conversion at each recomposition.
 * @param modifier The modifier for the layout.
 * @param resolveMentionDisplay A function to resolve the [TextDisplay] of a mention.
 * @param resolveRoomMentionDisplay A function to resolve the [TextDisplay] of an `@room` mention.
 * @param style The styles to use for any customisable elements.
 * @param releaseOnDetach Whether to release the view when the composable is detached from the composition or not.
 * Setting this to `false` is specially useful in Lazy composables that need to reuse these views. Defaults to `true`.
 */
@Composable
fun EditorStyledText(
    text: CharSequence,
    modifier: Modifier = Modifier,
    resolveMentionDisplay: (text: String, url: String) -> TextDisplay = RichTextEditorDefaults.MentionDisplay,
    resolveRoomMentionDisplay: () -> TextDisplay = RichTextEditorDefaults.RoomMentionDisplay,
    onLinkClickedListener: ((String) -> Unit) = {},
    onTextLayout: (Layout) -> Unit = {},
    style: RichTextEditorStyle = RichTextEditorDefaults.style(),
    releaseOnDetach: Boolean = true,
) {
    val typeface by style.text.rememberTypeface()
    val mentionDisplayHandler = remember(resolveMentionDisplay, resolveRoomMentionDisplay) {
        object : MentionDisplayHandler {
            override fun resolveMentionDisplay(text: String, url: String): TextDisplay {
                return resolveMentionDisplay(text, url)
            }

            override fun resolveAtRoomMentionDisplay(): TextDisplay {
                return resolveRoomMentionDisplay()
            }
        }
    }

    val isInEditMode = LocalInspectionMode.current
    AndroidView(
        modifier = modifier,
        factory = { context ->
            EditorStyledTextView(context).apply {
                isNativeCodeEnabled = !isInEditMode
            }
        },
        update = { view ->
            view.applyStyleInCompose(style)
            view.updateStyle(style.toStyleConfig(view.context), mentionDisplayHandler)
            view.typeface = typeface
            if (text is Spanned) {
                view.setText(text, TextView.BufferType.SPANNABLE)
            } else {
                view.setHtml(text.toString())
            }
            view.onLinkClickedListener = onLinkClickedListener
            view.onTextLayout = onTextLayout
        },
        onReset = { view: EditorStyledTextView ->
            view.setText("", TextView.BufferType.SPANNABLE)
        }.takeIf { releaseOnDetach },
    )
}
