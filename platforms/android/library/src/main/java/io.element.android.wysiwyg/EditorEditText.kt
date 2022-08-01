package io.element.android.wysiwyg

import android.content.Context
import android.os.Build
import android.text.Spannable
import android.text.SpannableStringBuilder
import android.util.AttributeSet
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import androidx.appcompat.widget.AppCompatEditText

class EditorEditText : AppCompatEditText {

    lateinit var inputConnection: InterceptInputConnection
    val inputProcessor = InputProcessor(uniffi.wysiwyg_composer.newComposerModel())

    private val spannableFactory = object : Spannable.Factory() {
        override fun newSpannable(source: CharSequence?): Spannable {
            // Try to reuse current source if possible to improve performance
            return source as? Spannable ?: SpannableStringBuilder(source)
        }
    }

    constructor(context: Context): super(context)

    constructor(context: Context, attrs: AttributeSet?): super(context, attrs)

    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int):
            super(context, attrs, defStyleAttr)

    init {
        setSpannableFactory(spannableFactory)
        addHardwareKeyInterceptor()
    }

    fun interface OnSelectionChangeListener {
        fun selectionChanged(start: Int, end: Int)
    }

    var selectionChangeListener: OnSelectionChangeListener? = null

    override fun onSelectionChanged(selStart: Int, selEnd: Int) {
        super.onSelectionChanged(selStart, selEnd)
        selectionChangeListener?.selectionChanged(selStart, selEnd)
    }

    override fun onCreateInputConnection(outAttrs: EditorInfo): InputConnection {
        return ensureInputConnection(outAttrs)
    }

    private fun addHardwareKeyInterceptor() {
        // This seems to be the only way to prevent EditText from automatically handling key strokes
        setOnKeyListener { v, keyCode, event ->
            if (event.keyCode == KeyEvent.KEYCODE_ENTER) {
                if (event.action == MotionEvent.ACTION_DOWN) {
                    inputConnection.sendHardwareKeyboardInput(event)
                }
                true
            } else if (event.action != MotionEvent.ACTION_DOWN) {
                false
            } else if (event.isMovementKey()) {
                false
            } else if (event.metaState != 0 && event.unicodeChar == 0) {
                // Is a modifier key
                false
            } else {
                inputConnection.sendHardwareKeyboardInput(event)
                true
            }
        }
    }

    /**
     * We wrap the internal [EditableInputConnection] as it's not public, but its internal behavior
     * is probably needed to work properly with the EditText.
     */
    private fun ensureInputConnection(outAttrs: EditorInfo): InterceptInputConnection {
        if (!this::inputConnection.isInitialized) {
            val baseInputConnection = requireNotNull(super.onCreateInputConnection(outAttrs))
            this.inputConnection =
                InterceptInputConnection(baseInputConnection, this, inputProcessor)
        }
        return this.inputConnection
    }

    override fun setText(text: CharSequence?, type: BufferType?) {
        val currentText = this.text
        val end = currentText?.length ?: 0
        // Although inputProcessor is assured to be not null, that's not the case while inflating.
        // We have to add this here to prevent some NullPointerExceptions from being thrown.
        if (inputProcessor == null) {
            super.setText(text, type)
        } else {
            inputProcessor.updateSelection(0, end)
            val update = inputProcessor.processInput(EditorInputAction.InsertText(text.toString()))
            val result = update?.let { inputProcessor.processUpdate(it) }

            if (result != null) {
                editableText.replace(0, end, result)
            } else {
                super.setText(text, type)
            }
        }
    }
}

private fun KeyEvent.isMovementKey(): Boolean {
    val baseIsMovement = this.keyCode in KeyEvent.KEYCODE_DPAD_UP..KeyEvent.KEYCODE_DPAD_CENTER
    val api24IsMovement = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
        this.keyCode in KeyEvent.KEYCODE_DPAD_UP_LEFT..KeyEvent.KEYCODE_DPAD_DOWN_RIGHT
    } else false
    return baseIsMovement || api24IsMovement
}
