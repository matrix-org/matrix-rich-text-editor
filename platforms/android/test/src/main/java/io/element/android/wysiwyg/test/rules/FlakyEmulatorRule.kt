package io.element.android.wysiwyg.test.rules

import org.junit.rules.RuleChain
import org.junit.rules.TestRule

/**
 * Creates a rule that helps to reduce emulator related flakiness.
 */
fun createFlakyEmulatorRule(): TestRule = RuleChain
    .outerRule(RetryOnFailureRule())
    .around(DismissAnrRule())
