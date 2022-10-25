package io.element.android.wysiwyg.viewmodel

import io.element.android.wysiwyg.extensions.toUShortList
import io.element.android.wysiwyg.utils.BasicHtmlConverter
import io.element.android.wysiwyg.utils.ResourcesProvider
import io.mockk.every
import io.mockk.mockk
import org.hamcrest.MatcherAssert.assertThat
import org.hamcrest.Matchers.equalTo
import org.junit.Test
import uniffi.wysiwyg_composer.ComposerModelInterface

internal class EditorViewModelTest {

    private val resourcesProvider = mockk<ResourcesProvider>()
    private val composer = mockk<ComposerModelInterface>()
    private val htmlConverter = BasicHtmlConverter()
    private val viewModel = EditorViewModel(
        resourcesProvider = resourcesProvider,
        composer = composer,
        htmlConverter = htmlConverter,
    )

    companion object {
        private const val paragraph =
            "Lorem Ipsum is simply dummy text of the printing and typesetting industry."
        private const val htmlParagraphs =
            "<p><b>$paragraph</b></p>" +
                    "<p><i>$paragraph</i></p>"
        private const val plainTextParagraphs = "$paragraph$paragraph"
    }

    @Test
    fun `given formatted text, getHtml returns formatted HTML`() {
        givenComposerHtml(htmlParagraphs)

        val html = viewModel.getHtml()

        assertThat(html, equalTo(htmlParagraphs))
    }

    @Test
    fun `given formatted text, getPlainText returns plain text`() {
        givenComposerHtml(htmlParagraphs)

        val plainText = viewModel.getPlainText()

        assertThat(plainText, equalTo(plainTextParagraphs))
    }

    private fun givenComposerHtml(composerHtml: String) =
        every { composer.getCurrentDomState() } returns mockk {
            every { html } returns composerHtml.toUShortList()
        }
}