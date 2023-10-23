package io.element.android.wysiwyg.compose

import android.text.Spanned
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import io.element.android.wysiwyg.EditorStyledTextView
import io.element.android.wysiwyg.compose.internal.applyStyle
import io.element.android.wysiwyg.compose.internal.rememberTypeface
import io.element.android.wysiwyg.compose.internal.toStyleConfig
import io.element.android.wysiwyg.display.MentionDisplayHandler

/**
 * A composable EditorStyledText.
 * This composable is a wrapper around the [EditorStyledTextView] view.
 *
 * @param text The text to render.
 * If it's spanned it will be rendered as is, otherwise it will go first to HtmlConverter.
 * Your might want to use HtmlConverter before the rendering to avoid the conversion at each recomposition.
 * @param modifier The modifier for the layout.
 * @param style The styles to use for any customisable elements.
 */
@Composable
fun EditorStyledText(
    text: CharSequence,
    modifier: Modifier = Modifier,
    mentionDisplayHandler: (() -> MentionDisplayHandler)? = null,
    style: RichTextEditorStyle = RichTextEditorDefaults.style(),
) {
    val typeface = style.text.rememberTypeface()
    AndroidView(modifier = modifier, factory = { context ->
        EditorStyledTextView(context)
    }, update = { view ->
        view.setStyleConfig(style.toStyleConfig(view.context))
        view.applyStyle(style)
        view.typeface = typeface
        view.mentionDisplayHandler = mentionDisplayHandler?.invoke()
        if (text is Spanned) {
            view.text = text
        } else {
            view.setHtml(text.toString())
        }
    })
}
