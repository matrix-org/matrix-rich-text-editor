package io.element.android.wysiwyg.utils

import android.text.style.BulletSpan
import androidx.core.text.buildSpannedString
import androidx.core.text.inSpans
import io.element.android.wysiwyg.spans.ExtraCharacterSpan
import org.junit.Assert
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class EditorIndexMapperTests {

    private val editableText = buildSpannedString {
        append("Plain text.\n") // 12 characters
        inSpans(BulletSpan()) {
            append("First item.") // 11 characters, ends at index 23
        }
        inSpans(ExtraCharacterSpan()) {
            append("\n") // 1 extra character, ends at index 24
        }
        inSpans(BulletSpan()) {
            append("Second item.") // 12 characters, ends at index 36
        }
        inSpans(ExtraCharacterSpan()) {
            append("\n") // 1 extra character, ends at index 37
        }
        append("After list.") // 11 characters, ends at index 48
    }

    //region Index before extra characters
    @Test
    fun `given an index with no extra characters before it, when using fromComposerToEditor, then the indexes match`() {
        val index = 0
        val (newStart, newEnd) = EditorIndexMapper.fromComposerToEditor(index, index, editableText)
        Assert.assertEquals(index, newStart)
        Assert.assertEquals(index, newEnd)
    }

    @Test
    fun `given an index with no extra characters before it, when using fromEditorToComposer, then the indexes match`() {
        val index = 0
        val (newStart, newEnd) = EditorIndexMapper.fromEditorToComposer(index, index, editableText)!!
        Assert.assertEquals(index.toUInt(), newStart)
        Assert.assertEquals(index.toUInt(), newEnd)
    }
    //endregion

    //region Invalid indexes passed
    @Test
    fun `given an invalid index, when using fromComposerToEditor, then the result indexes are -1`() {
        val index = -1
        val (newStart, newEnd) = EditorIndexMapper.fromComposerToEditor(index, index, editableText)
        Assert.assertEquals(-1, newStart)
        Assert.assertEquals(-1, newEnd)
    }

    @Test
    fun `given an invalid index, when using fromEditorToComposer, then it returns null`() {
        val index = -1
        val result = EditorIndexMapper.fromEditorToComposer(index, index, editableText)
        Assert.assertNull(result)
    }
    //endregion

    // region Index after an extra character
    @Test
    fun `given an index with an extra character before it, when using fromComposerToEditor, then the indexes have an offset`() {
        val index = 25
        val (newStart, newEnd) = EditorIndexMapper.fromComposerToEditor(index, index, editableText)
        Assert.assertEquals(index+1, newStart)
        Assert.assertEquals(index+1, newEnd)
    }

    @Test
    fun `given an index with an extra character before it, when using fromEditorToComposer, then the indexes have an offset`() {
        val index = 25
        val (newStart, newEnd) = EditorIndexMapper.fromEditorToComposer(index, index, editableText)!!
        Assert.assertEquals(index-1, newStart.toInt())
        Assert.assertEquals(index-1, newEnd.toInt())
    }
    // endregion

    // region Selection is after an extra character
    @Test
    fun `given a selection with an extra character before it, when using fromComposerToEditor, then the indexes have an offset`() {
        val start = 25
        val end = 28
        val (newStart, newEnd) = EditorIndexMapper.fromComposerToEditor(start, end, editableText)
        Assert.assertEquals(start+1, newStart)
        Assert.assertEquals(end+1, newEnd)
    }

    @Test
    fun `given a selection with an extra character before it, when using fromEditorToComposer, then the indexes have an offset`() {
        val start = 25
        val end = 28
        val (newStart, newEnd) = EditorIndexMapper.fromEditorToComposer(start, end, editableText)!!
        Assert.assertEquals(start-1, newStart.toInt())
        Assert.assertEquals(end-1, newEnd.toInt())
    }
    // endregion

    // region Selection covers an extra character
    @Test
    fun `given a selection with an extra character in the middle of it, when using fromComposerToEditor, then the end index have an offset`() {
        val start = 22
        val end = 25
        val (newStart, newEnd) = EditorIndexMapper.fromComposerToEditor(start, end, editableText)
        Assert.assertEquals(start, newStart)
        Assert.assertEquals(end+1, newEnd)
    }

    @Test
    fun `given a selection with an extra character in the middle of it, when using fromEditorToComposer, then the end index have an offset`() {
        val start = 22
        val end = 25
        val (newStart, newEnd) = EditorIndexMapper.fromEditorToComposer(start, end, editableText)!!
        Assert.assertEquals(start, newStart.toInt())
        Assert.assertEquals(end-1, newEnd.toInt())
    }
    // endregion

    // region Selection covers an extra character after a previous one
    @Test
    fun `given a selection covering an extra character with another one before it, when using fromComposerToEditor, then the indexes have an offset, but start and end won't match`() {
        val start = 28
        val end = 40
        val (newStart, newEnd) = EditorIndexMapper.fromComposerToEditor(start, end, editableText)
        Assert.assertEquals(start+1, newStart)
        Assert.assertEquals(end+2, newEnd)
    }

    @Test
    fun `given a selection covering an extra character with another one before it, when using fromEditorToComposer, then the indexes have an offset, but start and end won't match`() {
        val start = 28
        val end = 40
        val (newStart, newEnd) = EditorIndexMapper.fromEditorToComposer(start, end, editableText)!!
        Assert.assertEquals(start-1, newStart.toInt())
        Assert.assertEquals(end-2, newEnd.toInt())
    }
    // endregion

    // region Selection covers only the extra character
    @Test
    fun `given a selection covering only the extra character, when using fromComposerToEditor, then the indexes have an offset, but start and end won't match`() {
        val start = 23
        val end = 24
        val (newStart, newEnd) = EditorIndexMapper.fromComposerToEditor(start, end, editableText)
        Assert.assertEquals(start, newStart)
        Assert.assertEquals(end+1, newEnd)
    }

    @Test
    fun `given a selection covering only the extra character, when using fromEditorToComposer, then the indexes have an offset, but start and end won't match`() {
        val start = 23
        val end = 24
        val (newStart, newEnd) = EditorIndexMapper.fromEditorToComposer(start, end, editableText)!!
        Assert.assertEquals(start, newStart.toInt())
        Assert.assertEquals(end-1, newEnd.toInt())
    }
    // endregion
}
