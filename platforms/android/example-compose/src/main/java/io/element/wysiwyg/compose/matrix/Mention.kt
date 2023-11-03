package io.element.wysiwyg.compose.matrix

/**
 * Utility model class for the sample app to represent a mention to a
 * matrix.org user or room
 */
sealed class Mention(
    val display: String,
) {
    abstract val key: String
    val link get() = "https://matrix.to/#/$key$display:matrix.org"
    val text get() = "$key$display"

    class Room(
        display: String
    ): Mention(display) {
        override val key: String = "#"
    }

    class User(
        display: String
    ): Mention(display) {
        override val key: String = "@"
    }

    class Command(
        display: String
    ): Mention(display) {
        override val key: String = "/"
    }

    object NotifyEveryone: Mention("room") {
        override val key: String = "@"
    }
}
