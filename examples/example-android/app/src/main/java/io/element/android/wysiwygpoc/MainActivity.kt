package io.element.android.wysiwygpoc

import android.os.Bundle
import android.text.Editable
import android.text.InputFilter
import android.text.Spanned
import android.text.TextWatcher
import androidx.appcompat.app.AppCompatActivity
import androidx.core.text.HtmlCompat
import io.element.android.wysiwygpoc.databinding.ActivityMainBinding
import uniffi.wysiwyg_composer.ComposerModel
import uniffi.wysiwyg_composer.TextUpdate

class MainActivity : AppCompatActivity() {

    private val composer: ComposerModel = uniffi.wysiwyg_composer.newComposerModel()
    private val inputProcessor = InputProcessor(composer)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        with (binding.editor) {
            requestFocus()
            selectionChangeListener = EditorEditText.OnSelectionChangeListener { start, end ->
                composer.select(start.toUInt(), end.toUInt())
            }
            addTextChangedListener(EditorTextWatcher(inputProcessor))
        }

        binding.buttonBold.setOnClickListener {
            val result = inputProcessor.processInput(
                EditorInputAction.ApplyInlineFormat(InlineFormat.Bold)
            )
            result?.let { binding.editor.setText(it) }
        }
    }

    class InputProcessor(
        private val composer: ComposerModel,
    ) {
        fun processInput(action: EditorInputAction): CharSequence? {
            val update = when (action) {
                is EditorInputAction.InsertText -> {
                    // This conversion to a plain String might be too simple
                    composer.replaceText(action.value.toString())
                }
                is EditorInputAction.InsertParagraph -> {
                    composer.enter()
                }
                is EditorInputAction.BackPress -> {
                    composer.backspace()
                }
                is EditorInputAction.ApplyInlineFormat -> {
                    when (action.format) {
                        is InlineFormat.Bold -> composer.bold()
                    }
                }
                is EditorInputAction.ReplaceAll -> return null
            }
            return when (val textUpdate = update.textUpdate()) {
                is TextUpdate.Keep -> null
                is TextUpdate.ReplaceAll -> {
                    stringToSpans(textUpdate.htmlString())
                }
            }
        }

        private fun stringToSpans(string: String): Spanned {
            // TODO: Check parsing flags
            return HtmlCompat.fromHtml(string, 0)
        }
    }

    class EditorTextWatcher(
        private val inputProcessor: InputProcessor,
    ) : TextWatcher {
        private var replacement: CharSequence? = null

        override fun beforeTextChanged(source: CharSequence?, start: Int, count: Int, after: Int) {}

        override fun onTextChanged(source: CharSequence?, start: Int, before: Int, count: Int) {
            // When we make any changes to the editor's text using `replacement` the TextWatcher
            // will be called again. When this happens, clean `replacement` and just return.
            if (replacement != null) {
                replacement = null
                return
            }
            // When all text is deleted, clean `replacement` and early return.
            if (source == null) {
                replacement = null
                return
            }

            // TODO: instead of using `replaced` + `ReplaceAll`, add a new replace operation with
            //  indexes in Rust to modify the underlying buffer. Otherwise, we're going to have to
            //  fight the IME's autocorrect feature.
            val replaced = if (before <= count) source.substring(start+before, start+count) else ""
            replacement = when {
                start == 0 && count == before -> {
                    inputProcessor.processInput(EditorInputAction.ReplaceAll(replaced))
                }
                before > count -> {
                    inputProcessor.processInput(EditorInputAction.BackPress)
                }
                count != 0 && replaced != "\n" -> {
                    inputProcessor.processInput(EditorInputAction.InsertText(replaced))
                }
                replaced == "\n" -> {
                    inputProcessor.processInput(EditorInputAction.InsertParagraph)
                }
                else -> null
            }
        }

        override fun afterTextChanged(s: Editable?) {
            replacement?.let {
                // Note: this is reentrant, it will call the TextWatcher again
                s?.replace(0, s.length, it, 0, it.length)
                if (s?.length == 0) {
                    replacement = null
                }
            }
        }
    }

    // InputFilter would be a lot cleaner to use, but it only allows modification of the current
    // word being written, not the whole text.
    private val inputFilter = InputFilter { source, start, end, dest, dstart, dend ->
        when {
            source.isNotEmpty() && source != "\n" -> {
                inputProcessor.processInput(EditorInputAction.InsertText(source))
            }
            source == "\n" -> {
                inputProcessor.processInput(EditorInputAction.InsertParagraph)
            }
            source.isEmpty() && dend > dstart -> {
                inputProcessor.processInput(EditorInputAction.BackPress)
            }
            else -> null
        }
    }
}

sealed interface EditorInputAction {
    data class InsertText(val value: CharSequence): EditorInputAction
    data class ReplaceAll(val value: CharSequence): EditorInputAction
    object InsertParagraph: EditorInputAction
    object BackPress: EditorInputAction
    data class ApplyInlineFormat(val format: InlineFormat): EditorInputAction
}

sealed interface InlineFormat {
    object Bold: InlineFormat
}

fun TextUpdate.ReplaceAll.htmlString(): String = with(StringBuffer()) {
    replacementHtml.forEach {
        appendCodePoint(it.toInt())
    }
    toString()
}
