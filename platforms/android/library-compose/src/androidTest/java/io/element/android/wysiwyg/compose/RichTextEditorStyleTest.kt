package io.element.android.wysiwyg.compose

import android.content.res.Resources.NotFoundException
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.unit.dp
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.ViewMatchers.withText
import io.element.android.wysiwyg.compose.testutils.StateFactory.createState
import io.element.android.wysiwyg.compose.testutils.ViewMatchers.isRichTextEditor
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import org.junit.Rule
import org.junit.Test

class RichTextEditorStyleTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    private val state = createState()
    private val styleFlow = MutableStateFlow(RichTextEditorDefaults.style())

    @Test
    fun testContentIsStillDisplayedAfterSetStyle() = runTest {
        showContent()

        composeTestRule.runOnUiThread {
            state.setHtml("<ul><li>Hello, world</li></ul>")
        }

        styleFlow.emit(
            RichTextEditorDefaults.style(
                bulletList = RichTextEditorDefaults.bulletListStyle(
                    bulletRadius = 20.dp
                )
            )
        )
        composeTestRule.awaitIdle()

        onView(isRichTextEditor())
            .check(matches(withText("Hello, world")))
    }

    @Test(expected = NotFoundException::class)
    fun testBadResourceThrows() = runTest {
        val nonExistentDrawable = 0
        showContent()

        styleFlow.emit(
            RichTextEditorDefaults.style(
                codeBlock = RichTextEditorDefaults.codeBlockStyle(
                    backgroundDrawable = nonExistentDrawable
                )
            )
        )

        composeTestRule.awaitIdle()
    }

    private fun showContent() =
        composeTestRule.setContent {
            val styleState by styleFlow.collectAsState()
            MaterialTheme {
                RichTextEditor(
                    state = state,
                    modifier = Modifier.fillMaxWidth(),
                    style = styleState
                )
            }
        }
}
