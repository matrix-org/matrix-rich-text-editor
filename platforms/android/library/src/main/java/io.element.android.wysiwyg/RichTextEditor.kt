package io.element.android.wysiwyg

import android.content.Context
import android.graphics.Rect
import android.util.AttributeSet
import android.view.LayoutInflater
import android.view.ViewGroup
import android.widget.Button
import android.widget.LinearLayout
import com.google.android.material.textfield.TextInputLayout

class RichTextEditor : TextInputLayout {

    val editText by lazy { findViewById<EditorEditText>(R.id.editText) }
    val menu by lazy { findViewById<ViewGroup>(R.id.menu) }

    constructor(context: Context): super(context)

    constructor(context: Context, attrs: AttributeSet?): super(context, attrs)

    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int):
            super(context, attrs, defStyleAttr)

    init {
        LayoutInflater.from(context).inflate(R.layout.view_rich_text_editor, this)
    }

    override fun onFinishInflate() {
        super.onFinishInflate()

        menu.findViewById<Button>(R.id.formatBoldButton).setOnClickListener {
            editText.toggleInlineFormat(InlineFormat.Bold)
        }
    }

    override fun requestFocus(direction: Int, previouslyFocusedRect: Rect?): Boolean {
        return editText.requestFocus(direction, previouslyFocusedRect)
    }

}
