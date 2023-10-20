package io.element.android.wysiwyg.internal.view.models


import io.element.android.wysiwyg.view.models.LinkAction
import uniffi.wysiwyg_composer.LinkAction as InternalLinkAction

internal fun InternalLinkAction.toApiModel(): LinkAction? =
    when (this) {
        is InternalLinkAction.Edit -> LinkAction.EditLink(currentUrl = url, currentText = text)
        is InternalLinkAction.Create -> LinkAction.SetLink(currentUrl = null)
        is InternalLinkAction.CreateWithText -> LinkAction.InsertLink
        is InternalLinkAction.Disabled -> null
    }
