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
import androidx.core.text.getSpans
import com.google.android.material.textfield.TextInputEditText
import io.element.android.wysiwyg.spans.HtmlToSpansParser
import kotlin.math.absoluteValue
import kotlin.math.min

class EditorEditText : TextInputEditText {

    lateinit var inputConnection: InterceptInputConnection
    private val inputProcessor = InputProcessor(context, uniffi.wysiwyg_composer.newComposerModel())

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
        if (inputProcessor != null) {
            inputProcessor.updateSelection(editableText, selStart, selEnd)
        }
        selectionChangeListener?.selectionChanged(selStart, selEnd)
    }

    /**
     * We wrap the internal [EditableInputConnection] as it's not public, but its internal behavior
     * is probably needed to work properly with the EditText.
     */
    override fun onCreateInputConnection(outAttrs: EditorInfo): InputConnection {
        val baseInputConnection = requireNotNull(super.onCreateInputConnection(outAttrs))
        this.inputConnection =
            InterceptInputConnection(baseInputConnection, this, inputProcessor)
        return this.inputConnection
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

    override fun setText(text: CharSequence?, type: BufferType?) {
        val currentText = this.text
        val end = currentText?.length ?: 0
        // Although inputProcessor is assured to be not null, that's not the case while inflating.
        // We have to add this here to prevent some NullPointerExceptions from being thrown.
        if (inputProcessor == null) {
            super.setText(text, type)
        } else if (text.isNullOrEmpty().not()) {
            inputProcessor.updateSelection(editableText, 0, end)
            val update = inputProcessor.processInput(EditorInputAction.InsertText(text.toString()))
            val result = update?.let { inputProcessor.processUpdate(it) }

            if (result != null) {
                editableText.clear()
                editableText.replace(0, end, result.text)
                setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
            } else {
                super.setText(text, type)
            }
        }
    }

    override fun append(text: CharSequence?, start: Int, end: Int) {
        val update = inputProcessor.processInput(EditorInputAction.InsertText(text.toString()))
        val result = update?.let { inputProcessor.processUpdate(it) }

        if (result != null) {
            editableText.clear()
            editableText.replace(0, end, result.text)
            setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
        } else {
            super.append(text, start, end)
        }
    }

    fun toggleInlineFormat(inlineFormat: InlineFormat): Boolean {
        val (start, end) = inputConnection.getCurrentComposition()
        inputProcessor.updateSelection(editableText, start, end)
        val update = inputProcessor.processInput(EditorInputAction.ApplyInlineFormat(inlineFormat))
        val result = update?.let { inputProcessor.processUpdate(it) }

        if (result != null) {
            editableText.clear()
            editableText.replace(0, editableText.length, result.text)
            setSelectionFromComposerUpdate(result.selection.last)
        }
        return result != null
    }

    fun undo() {
        val update = inputProcessor.processInput(EditorInputAction.Undo)
        val result = update?.let { inputProcessor.processUpdate(it) }

        if (result != null) {
            editableText.clear()
            editableText.replace(0, editableText.length, result.text)
            setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
        }
    }

    fun redo() {
        val update = inputProcessor.processInput(EditorInputAction.Redo)
        val result = update?.let { inputProcessor.processUpdate(it) }

        if (result != null) {
            editableText.clear()
            editableText.replace(0, editableText.length, result.text)
            setSelectionFromComposerUpdate(result.selection.last)
        }
    }

    fun setLink(link: String) {
        val update = inputProcessor.processInput(EditorInputAction.SetLink(link))
        val result = update?.let { inputProcessor.processUpdate(it) }

        if (result != null) {
            editableText.clear()
            editableText.replace(0, editableText.length, result.text)
            setSelectionFromComposerUpdate(result.selection.last)
        }
    }

    fun toggleList(ordered: Boolean) {
        val update = inputProcessor.processInput(EditorInputAction.ToggleList(ordered))
        val result = update?.let { inputProcessor.processUpdate(it) }

        if (result != null) {
            editableText.clear()
            editableText.replace(0, editableText.length, result.text)
            setSelectionFromComposerUpdate(result.selection.last)
        }
    }

    private fun setSelectionFromComposerUpdate(start: Int, end: Int = start) {
        val zeroWidthLineBreaks = editableText.getSpans<HtmlToSpansParser.ZeroWidthLineBreak>()
        val before = zeroWidthLineBreaks.filter { editableText.getSpanStart(it) <= start }
            .sumOf { (editableText.getSpanEnd(it) - editableText.getSpanStart(it)).absoluteValue }
        val during = zeroWidthLineBreaks.filter { editableText.getSpanStart(it) <= end }
            .sumOf { (editableText.getSpanEnd(it) - editableText.getSpanStart(it)).absoluteValue }
        val newStart = min(start + before, editableText.length)
        val newEnd = min(end + during, editableText.length)
        setSelection(newStart, newEnd)
    }
}

private fun KeyEvent.isMovementKey(): Boolean {
    val baseIsMovement = this.keyCode in KeyEvent.KEYCODE_DPAD_UP..KeyEvent.KEYCODE_DPAD_CENTER
    val api24IsMovement = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
        this.keyCode in KeyEvent.KEYCODE_DPAD_UP_LEFT..KeyEvent.KEYCODE_DPAD_DOWN_RIGHT
    } else false
    return baseIsMovement || api24IsMovement
}
