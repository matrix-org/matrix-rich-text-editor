package io.element.android.wysiwyg

import android.content.Context
import android.graphics.Rect
import android.util.AttributeSet
import android.view.LayoutInflater
import android.view.ViewGroup
import android.widget.Button
import android.widget.LinearLayout
import com.google.android.material.textfield.TextInputLayout
import io.element.android.wysiwyg.databinding.ViewRichTextEditorBinding

class RichTextEditor : TextInputLayout {

    private val binding = ViewRichTextEditorBinding.inflate(LayoutInflater.from(context), this, true)

    constructor(context: Context): super(context)

    constructor(context: Context, attrs: AttributeSet?): super(context, attrs)

    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int):
            super(context, attrs, defStyleAttr)

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
