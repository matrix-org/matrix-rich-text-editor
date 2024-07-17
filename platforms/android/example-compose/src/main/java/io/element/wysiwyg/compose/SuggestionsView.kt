package io.element.wysiwyg.compose

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Divider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.snapshots.SnapshotStateList
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import io.element.wysiwyg.compose.matrix.Mention
import uniffi.wysiwyg_composer.MenuAction
import uniffi.wysiwyg_composer.PatternKey

@Composable
fun SuggestionView(
    modifier: Modifier = Modifier,
    roomMemberSuggestions: SnapshotStateList<Mention>,
    onReplaceSuggestion: (String) -> Unit,
    onInsertMentionAtSuggestion: (text: String, link: String) -> Unit,
    onInsertAtRoomMentionAtSuggestion: () -> Unit,
) {
    LazyColumn(
        modifier = modifier.fillMaxWidth()
    ) {
        items(roomMemberSuggestions) { item ->
            Column {
                Text(
                    text = item.display,
                    modifier = Modifier.fillMaxWidth()
                        .padding(10.dp)
                        .clickable {
                            when (item) {
                                Mention.NotifyEveryone -> {
                                    onInsertAtRoomMentionAtSuggestion()
                                }
                                is Mention.SlashCommand -> {
                                    onReplaceSuggestion(item.text)
                                }
                                else -> {
                                    onInsertMentionAtSuggestion(item.text, item.link)
                                }
                            }
                        })
                Divider(modifier = Modifier.fillMaxWidth())
            }
        }
    }
}

/**
 * Process the menu action and updates the suggestions accordingly. When the [MenuAction] is [MenuAction.Suggestion],
 * different mention suggestions are generated based on the [PatternKey] and the [MenuAction.Suggestion.suggestionPattern].
 * Otherwise, the suggestions are cleared.
 */
fun processMenuAction(menuAction: MenuAction?, roomMemberSuggestions: SnapshotStateList<Mention>) {
    when (menuAction) {
        is MenuAction.Suggestion -> {
            processSuggestion(menuAction, roomMemberSuggestions)
        }
        else -> {
            roomMemberSuggestions.clear()
        }
    }
}

private fun processSuggestion(suggestion: MenuAction.Suggestion, roomMemberSuggestions: SnapshotStateList<Mention>) {
    val text = suggestion.suggestionPattern.text
    val people = listOf("alice", "bob", "carol", "dan").map(Mention::User)
    val rooms = listOf("matrix", "element").map(Mention::Room)
    val slashCommands = listOf("leave", "shrug").map(Mention::SlashCommand)
    val everyone = Mention.NotifyEveryone
    val names = when (suggestion.suggestionPattern.key) {
        PatternKey.At -> people + everyone
        PatternKey.Hash -> rooms
        PatternKey.Slash -> slashCommands
        is PatternKey.Custom -> listOf()
    }

    val suggestions = names
        .filter { it.display.contains(text) }
    roomMemberSuggestions.clear()
    roomMemberSuggestions.addAll(suggestions)
}
