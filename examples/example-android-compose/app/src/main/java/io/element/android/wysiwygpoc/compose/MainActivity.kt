package io.element.android.wysiwygpoc.compose

import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import com.google.accompanist.appcompattheme.AppCompatTheme
import io.element.android.wysiwygpoc.compose.databinding.ActivityMainBinding
import uniffi.wysiwyg_composer.ComposerModel
import uniffi.wysiwyg_composer.TextUpdate

class MainActivity : AppCompatActivity() {

    private lateinit var composer: ComposerModel

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        composer = uniffi.wysiwyg_composer.newComposerModel()

        binding.editor.setContent {
            AppCompatTheme {
                RichTextEditor(composer = composer)
            }
        }

        binding.buttonBold.setOnClickListener {
            when (val textUpdate = composer.bold().textUpdate()) {
                is TextUpdate.Keep -> {}
                is TextUpdate.ReplaceAll -> {
//                    binding.editor.setText(Html.fromHtml(textUpdate.replacementHtml, 0))
                }
            }
        }
    }

//    class EditorTextWatcher(
//        private val composer: ComposerModel,
//    ) : TextWatcher {
//        private var replacement: CharSequence? = null
//
//        override fun beforeTextChanged(source: CharSequence?, start: Int, count: Int, after: Int) {
//        }
//
//        override fun onTextChanged(source: CharSequence?, start: Int, before: Int, count: Int) {
//            if (replacement != null) {
//                replacement = null
//                return
//            }
//            if (source == null) {
//                replacement = null
//                return
//            }
//
//            val replaced = source.substring(start, start + count)
//            replacement = when {
//                start == 0 && count == before -> {
//                    processInput(EditorInputAction.ReplaceAll(replaced))
//                }
//                before > count -> {
//                    processInput(EditorInputAction.BackPress)
//                }
//                count != 0 && replaced != "\n" -> {
//                    processInput(EditorInputAction.InsertText(replaced))
//                }
//                replaced == "\n" -> {
//                    processInput(EditorInputAction.InsertParagraph)
//                }
//                else -> null
//            }
//        }
//
//        override fun afterTextChanged(s: Editable?) {
//            replacement?.let {
//                s?.replace(0, s.length, it, 0, it.length)
//                if (s?.length == 0) {
//                    replacement = null
//                }
//            }
//        }
//
//        private fun processInput(action: EditorInputAction): CharSequence? {
//            val update = when (action) {
//                is EditorInputAction.InsertText -> {
//                    // This conversion to a plain String might be too simple
//                    composer.replaceText(action.value.toString())
//                }
//                is EditorInputAction.InsertParagraph -> {
//                    composer.enter()
//                }
//                is EditorInputAction.BackPress -> {
//                    composer.backspace()
//                }
//                is EditorInputAction.ApplyInlineFormat -> {
//                    when (action.format) {
//                        is InlineFormat.Bold -> composer.bold()
//                    }
//                }
//                is EditorInputAction.ReplaceAll -> return null
//            }
//            return when (val textUpdate = update.textUpdate()) {
//                is TextUpdate.Keep -> null
//                is TextUpdate.ReplaceAll -> {
//                    // Check flags
//                    Html.fromHtml(textUpdate.replacementHtml, 0)
//                }
//            }
//        }
//    }
}
