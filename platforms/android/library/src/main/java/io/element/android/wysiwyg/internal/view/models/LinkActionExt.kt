package io.element.android.wysiwyg.internal.view.models


import io.element.android.wysiwyg.view.models.LinkAction
import uniffi.wysiwyg_composer.LinkAction as InternalLinkAction

internal fun InternalLinkAction.toApiModel(
    prevAction: LinkAction?
): LinkAction? =
    when (this) {
        is InternalLinkAction.Keep -> prevAction
        is InternalLinkAction.Edit -> LinkAction.SetLink(currentUrl = url)
        is InternalLinkAction.Create -> LinkAction.SetLink(currentUrl = null)
        is InternalLinkAction.CreateWithText -> LinkAction.InsertLink
        is InternalLinkAction.Disabled -> null
    }
