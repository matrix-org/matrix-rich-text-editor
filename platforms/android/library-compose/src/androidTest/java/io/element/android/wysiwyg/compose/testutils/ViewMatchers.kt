package io.element.android.wysiwyg.compose.testutils

import android.view.View
import androidx.test.espresso.matcher.ViewMatchers
import io.element.android.wysiwyg.EditorEditText
import org.hamcrest.Matcher

object ViewMatchers {
    fun isRichTextEditor(): Matcher<View> =
        ViewMatchers.isAssignableFrom(EditorEditText::class.java)
}