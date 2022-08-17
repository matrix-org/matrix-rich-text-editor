package io.element.android.wysiwyg

import android.content.Context
import android.graphics.Rect
import android.util.AttributeSet
import android.view.LayoutInflater
import com.google.android.material.textfield.TextInputLayout
import io.element.android.wysiwyg.databinding.ViewRichTextEditorBinding

class RichTextEditor : TextInputLayout {

    private val binding = ViewRichTextEditorBinding.inflate(LayoutInflater.from(context), this, true)

    constructor(context: Context): super(context)

    constructor(context: Context, attrs: AttributeSet?): super(context, attrs)

    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int):
            super(context, attrs, defStyleAttr)

    var onSetLinkListener: OnSetLinkListener? = null

    override fun onFinishInflate() {
        super.onFinishInflate()

        with (binding) {
            formatBoldButton.setOnClickListener {
                editText.toggleInlineFormat(InlineFormat.Bold)
            }
            formatItalicButton.setOnClickListener {
                editText.toggleInlineFormat(InlineFormat.Italic)
            }
            formatUnderlineButton.setOnClickListener {
                editText.toggleInlineFormat(InlineFormat.Underline)
            }
            formatStrikeThroughButton.setOnClickListener {
                editText.toggleInlineFormat(InlineFormat.StrikeThrough)
            }
            addLinkButton.setOnClickListener {
                onSetLinkListener?.openLinkDialog(null) { link ->
                    editText.setLink(link)
                }
            }
            undoButton.setOnClickListener {
                editText.undo()
            }
            redoButton.setOnClickListener {
                editText.redo()
            }
        }
    }

    override fun requestFocus(direction: Int, previouslyFocusedRect: Rect?): Boolean {
        return binding.editText.requestFocus(direction, previouslyFocusedRect)
    }

}

interface OnSetLinkListener {
    fun openLinkDialog(link: String?, callback: (String) -> Unit)
}
