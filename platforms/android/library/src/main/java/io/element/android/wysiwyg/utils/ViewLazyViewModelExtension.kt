package io.element.android.wysiwyg.utils

import android.view.View
import androidx.annotation.MainThread
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelLazy
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.findViewTreeViewModelStoreOwner

@MainThread
inline fun <reified VM : ViewModel> View.viewModel(
    noinline viewModelInitializer: (() -> VM)? = null
): Lazy<VM> {
    return ViewModelLazy(
        viewModelClass = VM::class,
        storeProducer = { findViewTreeViewModelStoreOwner()!!.viewModelStore },
        factoryProducer = {
            object : ViewModelProvider.Factory {
                override fun <T : ViewModel> create(modelClass: Class<T>): T {
                    @Suppress("UNCHECKED_CAST")// Casting T as ViewModel
                    return viewModelInitializer?.let { it.invoke() as T }
                        ?: modelClass.newInstance()
                }
            }
        }
    )
}
