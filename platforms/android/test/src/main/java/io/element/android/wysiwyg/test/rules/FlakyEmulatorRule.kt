package io.element.android.wysiwyg.test.rules

import org.junit.rules.RuleChain
import org.junit.rules.TestRule

/**
 * Creates a rule that helps to reduce emulator related flakiness.
 */
fun createFlakyEmulatorRule(retry: Boolean = true): TestRule = if (retry) {
    RuleChain
        .outerRule(RetryOnFailureRule())
        .around(DismissAnrRule())
} else {
    RuleChain
        .outerRule(DismissAnrRule())
}
