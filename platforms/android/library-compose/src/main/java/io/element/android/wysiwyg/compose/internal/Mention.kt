package io.element.android.wysiwyg.compose.internal

/**
 * Utility model class for to represent a mention to a user or room
 */
sealed class Mention(
    val display: String,
    val link: String?,
) {
    abstract val key: String
    val text get() = display.prependIfMissing(key)

    class Room(
        display: String,
        link: String,
    ): Mention(display, link) {
        override val key: String = "#"
    }

    class User(
        display: String,
        link: String,
    ): Mention(display, link) {
        override val key: String = "@"
    }

    class SlashCommand(
        display: String
    ): Mention(display, null) {
        override val key: String = "/"
    }

    data object NotifyEveryone: Mention("room", null) {
        override val key: String = "@"
    }
}

private fun String.prependIfMissing(prefix: String): String {
    return if (startsWith(prefix)) this else "$prefix$this"
}