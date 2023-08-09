package io.element.android.wysiwyg.compose

import io.element.android.wysiwyg.compose.internal.ViewConnection
import io.element.android.wysiwyg.view.models.InlineFormat
import io.mockk.mockk
import io.mockk.verify
import org.junit.Assert.assertThrows
import org.junit.Test
import org.junit.runner.RunWith
import org.junit.runners.Parameterized
import uniffi.wysiwyg_composer.ComposerAction

@RunWith(Parameterized::class)
internal class RichTextEditorStateHandleActionTest(
    val action: ComposerAction,
    val verifyMethodIsCalled: (ViewConnection) -> Unit
) {

    @Test
    fun `handleAction should call viewConnection handleAction`() {
        val state = RichTextEditorState()
        val mockViewConnection = mockk<ViewConnection>(relaxed = true)
        state.viewConnection = mockViewConnection

        if (action == ComposerAction.LINK) {
            assertThrows(
                NotImplementedError::class.java
            ) {
                state.handleAction(action)
            }
        } else {
            state.handleAction(action)

            verify {
                verifyMethodIsCalled(mockViewConnection)
            }
        }
    }

    companion object {
        @JvmStatic
        @Parameterized.Parameters
        fun data(): Collection<Array<Any>> {
            return ComposerAction.values().map {
                arrayOf(it, when (it) {
                    ComposerAction.BOLD -> { v: ViewConnection ->
                        v.toggleInlineFormat(
                            InlineFormat.Bold
                        )
                    }

                    ComposerAction.ITALIC -> { v: ViewConnection ->
                        v.toggleInlineFormat(
                            InlineFormat.Italic
                        )
                    }

                    ComposerAction.STRIKE_THROUGH -> { v: ViewConnection ->
                        v.toggleInlineFormat(
                            InlineFormat.StrikeThrough
                        )
                    }

                    ComposerAction.UNDERLINE -> { v: ViewConnection ->
                        v.toggleInlineFormat(
                            InlineFormat.Underline
                        )
                    }

                    ComposerAction.INLINE_CODE -> { v: ViewConnection ->
                        v.toggleInlineFormat(
                            InlineFormat.InlineCode
                        )
                    }

                    ComposerAction.LINK -> { _: ViewConnection -> }
                    ComposerAction.UNDO -> { v: ViewConnection -> v.undo() }
                    ComposerAction.REDO -> { v: ViewConnection -> v.redo() }
                    ComposerAction.ORDERED_LIST -> { v: ViewConnection -> v.toggleList(ordered = true) }
                    ComposerAction.UNORDERED_LIST -> { v: ViewConnection -> v.toggleList(ordered = false) }
                    ComposerAction.INDENT -> { v: ViewConnection -> v.indent() }
                    ComposerAction.UNINDENT -> { v: ViewConnection -> v.unindent() }
                    ComposerAction.CODE_BLOCK -> { v: ViewConnection -> v.toggleCodeBlock() }
                    ComposerAction.QUOTE -> { v: ViewConnection -> v.toggleQuote() }
                })
            }
        }
    }
}
