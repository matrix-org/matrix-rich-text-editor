package io.element.android.wysiwyg.compose.text

import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.InlineTextContent
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.AnnotatedString.Builder
import androidx.compose.ui.text.AnnotatedString.Range
import androidx.compose.ui.text.ExperimentalTextApi
import androidx.compose.ui.text.ParagraphStyle
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TtsAnnotation
import androidx.compose.ui.text.UrlAnnotation
import kotlin.contracts.ExperimentalContracts
import kotlin.contracts.contract

/** The annotation tag used by inline content. */
internal const val INLINE_CONTENT_TAG = "androidx.compose.foundation.text.inlineContent"
// A string that contains a replacement character specified by unicode. It's used as the default
// value of alternate text.
private const val REPLACEMENT_CHAR = "\uFFFD"

/**
 * A builder for [AnnotatedString] that handles complex use cases, such as nested paragraph styles.
 */
class SafeAnnotatedStringBuilder(capacity: Int = 16): Appendable {

    companion object {
        private val annotatedStringConstructor = AnnotatedString::class.constructors.first {
            it.parameters.lastOrNull()?.name == "annotations"
        }
    }

    private data class MutableRange<T>(
        val item: T,
        val start: Int,
        var end: Int = Int.MIN_VALUE,
        val tag: String = ""
    ) {
        /**
         * Create an immutable [Range] object.
         *
         * @param defaultEnd if the end is not set yet, it will be set to this value.
         */
        fun toRange(defaultEnd: Int = Int.MIN_VALUE): Range<T> {
            val end = if (end == Int.MIN_VALUE) defaultEnd else end
            check(end != Int.MIN_VALUE) { "Item.end should be set first" }
            return Range(item = item, start = start, end = end, tag = tag)
        }
    }

    private val text = StringBuilder(capacity)
    private val spanStyles: MutableList<MutableRange<SpanStyle>> = mutableListOf()
    private val paragraphStyles: MutableList<MutableRange<ParagraphStyle>> = mutableListOf()
    private val annotations: MutableList<MutableRange<out Any>> = mutableListOf()
    private val styleStack: MutableList<MutableRange<out Any>> = mutableListOf()

    /**
     * Create an [Builder] instance using the given [String].
     */
    constructor(text: String) : this() {
        append(text)
    }

    /**
     * Create an [Builder] instance using the given [AnnotatedString].
     */
    constructor(text: AnnotatedString) : this() {
        append(text)
    }

    /**
     * Returns the length of the [String].
     */
    val length: Int get() = text.length

    /**
     * Appends the given [String] to this [Builder].
     *
     * @param text the text to append
     */
    fun append(text: String) {
        this.text.append(text)
    }

    /**
     * Appends [text] to this [Builder] if non-null, and returns this [Builder].
     *
     * If [text] is an [AnnotatedString], all spans and annotations will be copied over as well.
     * No other subtypes of [CharSequence] will be treated specially. For example, any
     * platform-specific types, such as `SpannedString` on Android, will only have their text
     * copied and any other information held in the sequence, such as Android `Span`s, will be
     * dropped.
     */
    @Suppress("BuilderSetStyle")
    override fun append(text: CharSequence?): SafeAnnotatedStringBuilder {
        if (text is AnnotatedString) {
            append(text)
        } else {
            this.text.append(text)
        }
        return this
    }


    /**
     * Appends the range of [text] between [start] (inclusive) and [end] (exclusive) to this
     * [Builder] if non-null, and returns this [Builder].
     *
     * If [text] is an [AnnotatedString], all spans and annotations from [text] between
     * [start] and [end] will be copied over as well.
     * No other subtypes of [CharSequence] will be treated specially. For example, any
     * platform-specific types, such as `SpannedString` on Android, will only have their text
     * copied and any other information held in the sequence, such as Android `Span`s, will be
     * dropped.
     *
     * @param start The index of the first character in [text] to copy over (inclusive).
     * @param end The index after the last character in [text] to copy over (exclusive).
     */
    @Suppress("BuilderSetStyle")
    override fun append(text: CharSequence?, start: Int, end: Int): SafeAnnotatedStringBuilder {
        if (text is AnnotatedString) {
            append(text, start, end)
        } else {
            this.text.append(text, start, end)
        }
        return this
    }

    // Kdoc comes from interface method.
    override fun append(char: Char): SafeAnnotatedStringBuilder {
        this.text.append(char)
        return this
    }

    /**
     * Used to insert composables into the text layout. This method can be used together with the
     * inlineContent parameter of [BasicText]. It will append the [alternateText] to this
     * [AnnotatedString] and also mark this range of text to be replaced by a composable.
     * [BasicText] will try to find an [InlineTextContent] in the map defined by inlineContent whose
     * key equals to [id], and it will use the [InlineTextContent.children] to replace this range of
     * text.
     *
     * @sample androidx.compose.foundation.samples.InlineTextContentSample
     * @see InlineTextContent
     * @see BasicText
     *
     * @param id The id used to look up the [InlineTextContent], it is referred by the inlineContent
     * parameter of [BasicText] to replace the [alternateText] to the corresponding composable.
     * @param alternateText The text to be replaced by the inline content. It's displayed when
     * the inlineContent parameter of [BasicText] doesn't contain [id].
     * Accessibility features will also use this text to describe the inline content.
     * @throws IllegalArgumentException if [alternateText] has zero length.
     */
    fun appendInlineContent(
        id: String,
        alternateText: String = REPLACEMENT_CHAR
    ) {
        require(alternateText.isNotEmpty()) {
            "alternateText can't be an empty string."
        }
        pushStringAnnotation(INLINE_CONTENT_TAG, id)
        append(alternateText)
        pop()
    }

    /**
     * Set a [SpanStyle] for the given [range].
     *
     * @param style [SpanStyle] to be applied
     * @param start the inclusive starting offset of the range
     * @param end the exclusive end offset of the range
     */
    fun addStyle(style: SpanStyle, start: Int, end: Int) {
        spanStyles.add(MutableRange(item = style, start = start, end = end))
    }

    /**
     * Set a [ParagraphStyle] for the given [range]. When a [ParagraphStyle] is applied to the
     * [AnnotatedString], it will be rendered as a separate paragraph.
     *
     * @param style [ParagraphStyle] to be applied
     * @param start the inclusive starting offset of the range
     * @param end the exclusive end offset of the range
     */
    fun addStyle(style: ParagraphStyle, start: Int, end: Int) {
        paragraphStyles.add(MutableRange(item = style, start = start, end = end))
    }

    /**
     * Set an Annotation for the given [range].
     *
     * @param tag the tag used to distinguish annotations
     * @param annotation the string annotation that is attached
     * @param start the inclusive starting offset of the range
     * @param end the exclusive end offset of the range
     * @see getStringAnnotations
     * @sample androidx.compose.ui.text.samples.AnnotatedStringAddStringAnnotationSample
     */
    fun addStringAnnotation(tag: String, annotation: String, start: Int, end: Int) {
        annotations.add(MutableRange(annotation, start, end, tag))
    }

    /**
     * Set a [TtsAnnotation] for the given [range].
     *
     * @param ttsAnnotation an object that stores text to speech metadata that intended for the
     * TTS engine.
     * @param start the inclusive starting offset of the range
     * @param end the exclusive end offset of the range
     * @see getStringAnnotations
     * @sample androidx.compose.ui.text.samples.AnnotatedStringAddStringAnnotationSample
     */
    @ExperimentalTextApi
    @Suppress("SetterReturnsThis")
    fun addTtsAnnotation(ttsAnnotation: TtsAnnotation, start: Int, end: Int) {
        annotations.add(MutableRange(ttsAnnotation, start, end))
    }

    /**
     * Set a [UrlAnnotation] for the given [range]. URLs may be treated specially by screen
     * readers, including being identified while reading text with an audio icon or being
     * summarized in a links menu.
     *
     * @param urlAnnotation A [UrlAnnotation] object that stores the URL being linked to.
     * @param start the inclusive starting offset of the range
     * @param end the exclusive end offset of the range
     * @see getStringAnnotations
     * @sample androidx.compose.ui.text.samples.AnnotatedStringAddStringAnnotationSample
     */
    @ExperimentalTextApi
    @Suppress("SetterReturnsThis")
    fun addUrlAnnotation(urlAnnotation: UrlAnnotation, start: Int, end: Int) {
        annotations.add(MutableRange(urlAnnotation, start, end))
    }

    /**
     * Applies the given [SpanStyle] to any appended text until a corresponding [pop] is
     * called.
     *
     * @sample androidx.compose.ui.text.samples.AnnotatedStringBuilderPushSample
     *
     * @param style SpanStyle to be applied
     */
    fun pushStyle(style: SpanStyle): Int {
        MutableRange(item = style, start = text.length).also {
            styleStack.add(it)
            spanStyles.add(it)
        }
        return styleStack.size - 1
    }

    /**
     * Applies the given [ParagraphStyle] to any appended text until a corresponding [pop]
     * is called.
     *
     * @sample androidx.compose.ui.text.samples.AnnotatedStringBuilderPushParagraphStyleSample
     *
     * @param style ParagraphStyle to be applied
     */
    fun pushStyle(style: ParagraphStyle): Int {
        val lastParagraphStyle = paragraphStyles.lastOrNull()
        if (lastParagraphStyle != null) {
            when {
                lastParagraphStyle.end == Int.MIN_VALUE -> lastParagraphStyle.end = text.length
                // TODO revisit this code, it might be possible to keep the last paragraph style in some cases
                lastParagraphStyle.start == text.length -> {
                    styleStack.removeLastOrNull()
                    paragraphStyles.removeLast()
                }
            }
        }
        MutableRange(item = style, start = text.length).also {
            styleStack.add(it)
            paragraphStyles.add(it)
        }
        return styleStack.size - 1
    }

    /**
     * Attach the given [annotation] to any appended text until a corresponding [pop]
     * is called.
     *
     * @sample androidx.compose.ui.text.samples.AnnotatedStringBuilderPushStringAnnotationSample
     *
     * @param tag the tag used to distinguish annotations
     * @param annotation the string annotation attached on this AnnotatedString
     * @see getStringAnnotations
     * @see Range
     */
    fun pushStringAnnotation(tag: String, annotation: String): Int {
        MutableRange(item = annotation, start = text.length, tag = tag).also {
            styleStack.add(it)
            annotations.add(it)
        }
        return styleStack.size - 1
    }

    /**
     * Attach the given [ttsAnnotation] to any appended text until a corresponding [pop]
     * is called.
     *
     * @sample androidx.compose.ui.text.samples.AnnotatedStringBuilderPushStringAnnotationSample
     *
     * @param ttsAnnotation an object that stores text to speech metadata that intended for the
     * TTS engine.
     * @see getStringAnnotations
     * @see Range
     */
    fun pushTtsAnnotation(ttsAnnotation: TtsAnnotation): Int {
        MutableRange(item = ttsAnnotation, start = text.length).also {
            styleStack.add(it)
            annotations.add(it)
        }
        return styleStack.size - 1
    }

    /**
     * Attach the given [UrlAnnotation] to any appended text until a corresponding [pop]
     * is called.
     *
     * @sample androidx.compose.ui.text.samples.AnnotatedStringBuilderPushStringAnnotationSample
     *
     * @param urlAnnotation A [UrlAnnotation] object that stores the URL being linked to.
     * @see getStringAnnotations
     * @see Range
     */
    @Suppress("BuilderSetStyle")
    @ExperimentalTextApi
    fun pushUrlAnnotation(urlAnnotation: UrlAnnotation): Int {
        MutableRange(item = urlAnnotation, start = text.length).also {
            styleStack.add(it)
            annotations.add(it)
        }
        return styleStack.size - 1
    }

    /**
     * Ends the style or annotation that was added via a push operation before.
     *
     * @see pushStyle
     * @see pushStringAnnotation
     */
    fun pop() {
        if (styleStack.isEmpty()) return
        // pop the last element
        val item = styleStack.removeAt(styleStack.size - 1)
        if (item.end == Int.MIN_VALUE) {
            item.end = text.length
        }
    }

    /**
     * Ends the styles or annotation up to and `including` the [pushStyle] or
     * [pushStringAnnotation] that returned the given index.
     *
     * @param index the result of the a previous [pushStyle] or [pushStringAnnotation] in order
     * to pop to
     *
     * @see pop
     * @see pushStyle
     * @see pushStringAnnotation
     */
    fun pop(index: Int) {
        check(index < styleStack.size) { "$index should be less than ${styleStack.size}" }
        while ((styleStack.size - 1) >= index) {
            pop()
        }
    }

    /**
     * Constructs an [AnnotatedString] based on the configurations applied to the [Builder].
     */
    fun toAnnotatedString(): AnnotatedString {
        if (text.isEmpty() && paragraphStyles.isNotEmpty()) {
            text.append(" ")
        }
        return annotatedStringConstructor.call(
            text.toString(),
            spanStyles
                .fastMap { it.toRange(text.length) }
                .ifEmpty { null },
            paragraphStyles
                .fastMap { it.toRange(text.length) }
                .filter { it.start != it.end }
                .ifEmpty { null },
            annotations
                .fastMap { it.toRange(text.length) }
                .ifEmpty { null }
        )
    }

    fun restoreLastParagraphStyle() {
        val lastStyle = styleStack.lastOrNull()?.item
        if (lastStyle is ParagraphStyle) {
            styleStack.removeLast()
            pushStyle(lastStyle)
        }
    }
}

/**
 * Iterates through a [List] using the index and calls [action] for each item.
 * This does not allocate an iterator like [Iterable.forEach].
 *
 * **Do not use for collections that come from public APIs**, since they may not support random
 * access in an efficient way, and this method may actually be a lot slower. Only use for
 * collections that are created by code we control and are known to support random access.
 */
@Suppress("BanInlineOptIn") // Treat Kotlin Contracts as non-experimental.
@OptIn(ExperimentalContracts::class)
internal inline fun <T> List<T>.fastForEach(action: (T) -> Unit) {
    contract { callsInPlace(action) }
    for (index in indices) {
        val item = get(index)
        action(item)
    }
}

/**
 * Returns a list containing the results of applying the given [transform] function
 * to each element in the original collection.
 *
 * **Do not use for collections that come from public APIs**, since they may not support random
 * access in an efficient way, and this method may actually be a lot slower. Only use for
 * collections that are created by code we control and are known to support random access.
 */
@Suppress("BanInlineOptIn") // Treat Kotlin Contracts as non-experimental.
@OptIn(ExperimentalContracts::class)
internal inline fun <T, R> List<T>.fastMap(transform: (T) -> R): List<R> {
    contract { callsInPlace(transform) }
    val target = ArrayList<R>(size)
    fastForEach {
        target += transform(it)
    }
    return target
}

/**
 * Helper function that checks if the range [baseStart, baseEnd) contains the range
 * [targetStart, targetEnd).
 *
 * @return true if [baseStart, baseEnd) contains [targetStart, targetEnd), vice versa.
 * When [baseStart]==[baseEnd] it return true iff [targetStart]==[targetEnd]==[baseStart].
 */
internal fun contains(baseStart: Int, baseEnd: Int, targetStart: Int, targetEnd: Int) =
    (baseStart <= targetStart && targetEnd <= baseEnd) &&
            (baseEnd != targetEnd || (targetStart == targetEnd) == (baseStart == baseEnd))

/**
 * Helper function that checks if the range [lStart, lEnd) intersects with the range
 * [rStart, rEnd).
 *
 * @return [lStart, lEnd) intersects with range [rStart, rEnd), vice versa.
 */
internal fun intersect(lStart: Int, lEnd: Int, rStart: Int, rEnd: Int) =
    maxOf(lStart, rStart) < minOf(lEnd, rEnd) ||
            contains(lStart, lEnd, rStart, rEnd) || contains(rStart, rEnd, lStart, lEnd)

inline fun buildSafeAnnotatedString(builder: (SafeAnnotatedStringBuilder).() -> Unit): AnnotatedString =
    SafeAnnotatedStringBuilder().apply(builder).toAnnotatedString()