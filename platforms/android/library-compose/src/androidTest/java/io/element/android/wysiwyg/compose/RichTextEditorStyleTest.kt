package io.element.android.wysiwyg.compose

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.unit.dp
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.ViewMatchers.withText
import io.element.android.wysiwyg.compose.testutils.StateFactory.createState
import io.element.android.wysiwyg.compose.testutils.ViewMatchers.isRichTextEditor
import io.element.android.wysiwyg.test.rules.createFlakyEmulatorRule
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import org.junit.Rule
import org.junit.Test

class RichTextEditorStyleTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @get:Rule
    val flakyEmulatorRule = createFlakyEmulatorRule()

    private val state = createState()
    private val bulletRadius = MutableStateFlow(2.dp)
    private val codeBgColor = MutableStateFlow(Color.Blue)

    @Test
    fun testContentIsStillDisplayedAfterSetStyle() = runTest {
        showContent()

        state.setHtml("<ul><li>Hello, world</li></ul>")

        bulletRadius.emit(20.dp)

        composeTestRule.awaitIdle()

        onView(isRichTextEditor())
            .check(matches(withText("Hello, world")))
    }

    private fun showContent() =
        composeTestRule.setContent {
            val bulletRadius by bulletRadius.collectAsState()
            val codeBgColor by codeBgColor.collectAsState()
            val style = RichTextEditorDefaults.style(
                bulletList = RichTextEditorDefaults.bulletListStyle(
                    bulletRadius = bulletRadius
                ),
                codeBlock = RichTextEditorDefaults.codeBlockStyle(
                    background = RichTextEditorDefaults.codeBlockBackgroundStyle(
                        color = codeBgColor
                    )
                )
            )
            MaterialTheme {
                RichTextEditor(
                    state = state,
                    modifier = Modifier.fillMaxWidth(),
                    style = style
                )
            }
        }
}
