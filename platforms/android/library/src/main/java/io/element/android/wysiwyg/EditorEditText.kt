package io.element.android.wysiwyg

import android.content.Context
import android.os.Build
import android.text.Spannable
import android.text.SpannableStringBuilder
import android.util.AttributeSet
import android.view.KeyEvent
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import androidx.core.text.getSpans
import com.google.android.material.textfield.TextInputEditText
import io.element.android.wysiwyg.spans.HtmlToSpansParser
import uniffi.wysiwyg_composer.MenuState
import uniffi.wysiwyg_composer.newComposerModel
import kotlin.math.absoluteValue
import kotlin.math.min

class EditorEditText : TextInputEditText {

    private var inputConnection: InterceptInputConnection? = null
    private val inputProcessor = InputProcessor(
        context,
        menuStateCallback = { menuStateChangedListener?.menuStateChanged(it) },
        // Using the returned ComposerModel automatically loads the native libraries and will crash
        // layout preview and other tools. We're making it nullable as a workaround for that.
        composer = if (isInEditMode) null else newComposerModel()
    )

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
    }

    fun interface OnSelectionChangeListener {
        fun selectionChanged(start: Int, end: Int)
    }

    fun interface OnMenuStateChangedListener {
        fun menuStateChanged(menuState: MenuState)
    }

    var selectionChangeListener: OnSelectionChangeListener? = null
    var menuStateChangedListener: OnMenuStateChangedListener? = null

    override fun onSelectionChanged(selStart: Int, selEnd: Int) {
        super.onSelectionChanged(selStart, selEnd)
        if (inputProcessor != null) {
            inputProcessor.updateSelection(editableText, selStart, selEnd)
        }
        selectionChangeListener?.selectionChanged(selStart, selEnd)
    }

    override fun onCreateInputConnection(outAttrs: EditorInfo): InputConnection {
        val inputConnection =
            InterceptInputConnection(this, inputProcessor)
        this.inputConnection = inputConnection
        return inputConnection
    }


    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean =
        if (event.keyCode == KeyEvent.KEYCODE_ENTER) {
            inputConnection?.processKeyEvent(event)
            true
        } else if (event.isMovementKey()) {
            super.onKeyDown(keyCode, event)
        } else if (event.metaState != 0 && event.unicodeChar == 0) {
            // Is a modifier key
            false
        } else {
            inputConnection?.processKeyEvent(event)
            true
        }

    override fun setText(text: CharSequence?, type: BufferType?) {
        val currentText = this.text
        val end = currentText?.length ?: 0
        // Although inputProcessor is assured to be not null, that's not the case while inflating.
        // We have to add this here to prevent some NullPointerExceptions from being thrown.
        if (inputProcessor == null) {
            super.setText(text, type)
        } else {
            inputProcessor.updateSelection(editableText, 0, end)
            val update = inputProcessor.processInput(EditorInputAction.InsertText(text.toString()))
            val result = update?.let { inputProcessor.processUpdate(it) }

            if (result != null) {
                setTextInternal(result.text)
                setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
            } else {
                super.setText(text, type)
            }
        }
    }

    private fun setTextInternal(text: CharSequence?) {
        beginBatchEdit()
        editableText.clear()
        editableText.replace(0, editableText.length, text)
        endBatchEdit()
    }

    override fun append(text: CharSequence?, start: Int, end: Int) {
        val update = inputProcessor.processInput(EditorInputAction.InsertText(text.toString()))
        val result = update?.let { inputProcessor.processUpdate(it) }

        if (result != null) {
            setTextInternal(result.text)
            setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
        } else {
            super.append(text, start, end)
        }
    }

    fun toggleInlineFormat(inlineFormat: InlineFormat): Boolean {
        val update = inputProcessor.processInput(EditorInputAction.ApplyInlineFormat(inlineFormat))
        val result = update?.let { inputProcessor.processUpdate(it) }

        if (result != null) {
            setTextInternal(result.text)
            setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
        }
        return result != null
    }

    fun undo() {
        val update = inputProcessor.processInput(EditorInputAction.Undo)
        val result = update?.let { inputProcessor.processUpdate(it) }

        if (result != null) {
            setTextInternal(result.text)
            setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
        }
    }

    fun redo() {
        val update = inputProcessor.processInput(EditorInputAction.Redo)
        val result = update?.let { inputProcessor.processUpdate(it) }

        if (result != null) {
            setTextInternal(result.text)
            setSelectionFromComposerUpdate(result.selection.last)
        }
    }

    fun setLink(link: String) {
        val update = inputProcessor.processInput(EditorInputAction.SetLink(link))
        val result = update?.let { inputProcessor.processUpdate(it) }

        if (result != null) {
            setTextInternal(result.text)
            setSelectionFromComposerUpdate(result.selection.last)
        }
    }

    fun toggleList(ordered: Boolean) {
        val update = inputProcessor.processInput(EditorInputAction.ToggleList(ordered))
        val result = update?.let { inputProcessor.processUpdate(it) }

        if (result != null) {
            setTextInternal(result.text)
            setSelectionFromComposerUpdate(result.selection.last)
        }
    }

    fun setHtml(html: String) {
        val update = inputProcessor.processInput(EditorInputAction.ReplaceAllHtml(html))
        val result = update?.let { inputProcessor.processUpdate(it) }

        if (result != null) {
            setTextInternal(result.text)
            setSelectionFromComposerUpdate(result.selection.last)
        }
    }

    fun getHtmlOutput(): String {
        return inputProcessor.getHtml()
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
