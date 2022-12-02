package io.element.android.wysiwyg.utils

import android.content.Context
import android.content.ContextWrapper
import android.view.View
import androidx.annotation.MainThread
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelLazy
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.ViewModelStoreOwner
import androidx.lifecycle.findViewTreeViewModelStoreOwner

@MainThread
inline fun <reified VM : ViewModel> View.viewModel(
    noinline viewModelInitializer: (() -> VM)? = null,
): Lazy<VM> {
    return ViewModelLazy(
        viewModelClass = VM::class,
        storeProducer = {
            // We no longer use `findViewTreeViewModelStoreOwner` here because it can cause crashes
            // in Compose AndroidViews. See https://github.com/matrix-org/matrix-rich-text-editor/pull/365.
            context.getViewModelStoreOwner().viewModelStore
        },
        factoryProducer = {
            object : ViewModelProvider.Factory {
                override fun <T : ViewModel> create(modelClass: Class<T>): T {
                    @Suppress("UNCHECKED_CAST")// Casting T as ViewModel
                    return viewModelInitializer?.let { it.invoke() as T }
                        ?: modelClass.newInstance()
                }
            }
        },
    )
}

fun Context.getViewModelStoreOwner(): ViewModelStoreOwner {
    var currentContext = this
    while (currentContext !is ViewModelStoreOwner) {
        if (currentContext is ContextWrapper) {
            currentContext = currentContext.baseContext
        } else {
            error("There is no base context that is a ViewModelStoreOwner")
        }
    }
    return currentContext
}
