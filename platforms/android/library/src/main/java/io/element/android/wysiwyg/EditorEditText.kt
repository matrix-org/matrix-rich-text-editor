package io.element.android.wysiwyg

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Context.CLIPBOARD_SERVICE
import android.os.Build
import android.text.Spannable
import android.text.SpannableStringBuilder
import android.util.AttributeSet
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import com.google.android.material.textfield.TextInputEditText
import io.element.android.wysiwyg.inputhandlers.models.EditorInputAction
import io.element.android.wysiwyg.inputhandlers.models.InlineFormat
import io.element.android.wysiwyg.inputhandlers.InputProcessor
import io.element.android.wysiwyg.inputhandlers.InterceptInputConnection
import io.element.android.wysiwyg.utils.EditorIndexMapper
import uniffi.wysiwyg_composer.MenuState
import uniffi.wysiwyg_composer.newComposerModel

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
        addHardwareKeyInterceptor()
    }

    fun interface OnSelectionChangeListener {
        fun selectionChanged(start: Int, end: Int)
    }

    fun interface OnMenuStateChangedListener {
        fun menuStateChanged(menuState: MenuState)
    }

    var selectionChangeListener: OnSelectionChangeListener? = null
    var menuStateChangedListener: OnMenuStateChangedListener? = null

    @Suppress("SAFE_CALL_WILL_CHANGE_NULLABILITY", "UNNECESSARY_SAFE_CALL")
    override fun onSelectionChanged(selStart: Int, selEnd: Int) {
        super.onSelectionChanged(selStart, selEnd)
        inputProcessor?.updateSelection(editableText, selStart, selEnd)
        selectionChangeListener?.selectionChanged(selStart, selEnd)
    }

    /**
     * We wrap the internal `com.android.internal.widget.EditableInputConnection` as it's not
     * public, but its internal behavior is needed to work properly with the EditText.
     */
    override fun onCreateInputConnection(outAttrs: EditorInfo): InputConnection {
        val baseInputConnection = requireNotNull(super.onCreateInputConnection(outAttrs))
        val inputConnection =
            InterceptInputConnection(baseInputConnection, this, inputProcessor)
        this.inputConnection = inputConnection
        return inputConnection
    }

    /**
     * Override cut & paste events so output is redirected to the [inputProcessor].
     */
    override fun onTextContextMenuItem(id: Int): Boolean {
        when (id) {
            android.R.id.cut -> {
                val clipboardManager = context.getSystemService(CLIPBOARD_SERVICE) as ClipboardManager
                val clpData = ClipData.newPlainText("newText", this.editableText.slice(this.selectionStart until this.selectionEnd))
                clipboardManager.setPrimaryClip(clpData)

                val update = inputProcessor.processInput(EditorInputAction.Delete(this.selectionStart, this.selectionEnd))
                val result = update?.let { inputProcessor.processUpdate(it) }

                if (result != null) {
                    setTextInternal(result.text)
                    setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
                }

                return false
            }
            android.R.id.paste, android.R.id.pasteAsPlainText -> {
                val clipBoardManager = context.getSystemService(CLIPBOARD_SERVICE) as ClipboardManager
                val copiedString = clipBoardManager.primaryClip?.getItemAt(0)?.text ?: return false
                val update = inputProcessor.processInput(EditorInputAction.ReplaceText(copiedString))
                val result = update?.let { inputProcessor.processUpdate(it) }

                if (result != null) {
                    setTextInternal(result.text)
                    setSelectionFromComposerUpdate(result.selection.first, result.selection.last)
                }

                return false
            }
            else -> { return super.onTextContextMenuItem(id) }
        }
    }

    private fun addHardwareKeyInterceptor() {
        // This seems to be the only way to prevent EditText from automatically handling key strokes
        setOnKeyListener { _, keyCode, event ->
            if (keyCode == KeyEvent.KEYCODE_ENTER) {
                if (event.action == MotionEvent.ACTION_DOWN) {
                    inputConnection?.sendHardwareKeyboardInput(event)
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
                inputConnection?.sendHardwareKeyboardInput(event)
                true
            }
        }
    }

    @Suppress("SENSELESS_COMPARISON")
    override fun setText(text: CharSequence?, type: BufferType?) {
        val currentText = this.text
        val end = currentText?.length ?: 0
        // Although inputProcessor is assured to be not null, that's not the case while inflating.
        // We have to add this here to prevent some NullPointerExceptions from being thrown.
        if (inputProcessor == null) {
            super.setText(text, type)
        } else {
            inputProcessor.updateSelection(editableText, 0, end)
            val update = inputProcessor.processInput(EditorInputAction.ReplaceText(text.toString()))
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
        val update = inputProcessor.processInput(EditorInputAction.ReplaceText(text.toString()))
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
        val (newStart, newEnd) = EditorIndexMapper.fromComposerToEditText(start, end, editableText)
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
