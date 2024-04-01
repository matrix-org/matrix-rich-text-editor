package io.element.android.wysiwyg

import android.text.Editable
import android.text.TextWatcher
import timber.log.Timber
import java.util.concurrent.atomic.AtomicBoolean

/**
 * A [TextWatcher] that intercepts changes in the underlying text and can have child watchers.
 */
internal class EditorTextWatcher: TextWatcher {
    private val nestedWatchers: MutableList<TextWatcher> = mutableListOf()
    private val updateIsFromEditor: AtomicBoolean = AtomicBoolean(false)

    val isInEditorChange get() = updateIsFromEditor.get()

    var enableDebugLogs = false

    private var beforeText: CharSequence? = null

    /**
     * The callback to be called when the text changes unexpectedly.
     * These changes don't come from the editor, they might come from the OS or other sources.
     */
    var updateCallback: (CharSequence, Int, Int, CharSequence?) -> Unit = { _, _, _, _ -> }

    /**
     * Add a child watcher to be notified of text changes.
     */
    fun addChild(watcher: TextWatcher) {
        nestedWatchers.add(watcher)
    }

    /**
     * Remove a child watcher from being notified of text changes.
     */
    fun removeChild(watcher: TextWatcher) {
        nestedWatchers.remove(watcher)
    }

    /**
     * Run a block of code that comes from the editor.
     *
     * This is used to prevent the editor from updating itself when it's already updating itself and
     * entering an endless loop.
     *
     * @param block The block of code to run.
     */
    fun runInEditor(block: EditorTextWatcher.() -> Unit) {
        updateIsFromEditor.set(true)
        this.block()
        updateIsFromEditor.set(false)
    }

    override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) {
        if (enableDebugLogs) {
            Timber.v("beforeTextChanged | text: \"$s\", start: $start, count: $count, after: $after")
        }
        if (!updateIsFromEditor.get()) {
            beforeText = s?.subSequence(start, start + count)
        }
    }

    override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) {
        if (enableDebugLogs) {
            Timber.v("onTextChanged | text: \"$s\", start: $start, before: $before, count: $count")
        }
        if (!updateIsFromEditor.get()) {
            val newText = s?.subSequence(start, start + count) ?: ""
            updateCallback(newText, start, start + before, beforeText)
        }
    }

    override fun afterTextChanged(s: Editable?) {
        if (enableDebugLogs) {
            Timber.v("afterTextChanged")
        }
        if (!updateIsFromEditor.get()) {
            beforeText = null
        }
    }

    /**
     * Notify the nested watchers that the text is about to change.
     * @param text The text that will be changed.
     * @param start The start index of the change.
     * @param count The number of characters that will be removed.
     * @param after The number of characters that will be added.
     */
    fun notifyBeforeTextChanged(text: CharSequence, start: Int, count: Int, after: Int) {
        nestedWatchers.forEach { it.beforeTextChanged(text, start, count, after) }
    }

    /**
     * Notify the nested watchers that the text is changing.
     * @param text The new text.
     * @param start The start index of the change.
     * @param before The number of characters that were removed.
     * @param count The number of characters that were added.
     */
    fun notifyOnTextChanged(text: CharSequence, start: Int, before: Int, count: Int) {
        nestedWatchers.forEach { it.onTextChanged(text, start, before, count) }
    }

    /**
     * Notify the nested watchers that the text changed.
     * @param editable The updated text.
     */
    fun notifyAfterTextChanged(editable: Editable) {
        nestedWatchers.forEach { it.afterTextChanged(editable) }
    }
}