# Changelog

# [0.10.0] - 2022-12-08

### Added

* Common: Add extension to character to create ZWSP easily
* Common: Add 'remove word' functionality
* Common: Extended selection/range for DOM
* Common: Add DOM handle depth helper
* iOS: Add links implementation in the example app for testing
* Android: Add link APIs

### Changed

* Common: Change link interface to use string

### Fixed

* Common: Make `find_range_by` work symmetrically
* Common: Lookup and join ancestor on join nodes

# [0.9.0] - 2022-12-06

### Added

* Web: Add `insertText` method in available functions
* Common: Add extension to character to create ZWSP easily

### Changed

Common & iOS: Remove the Rust & Uniffi version duplication from publish.yml
Common: Change Rust panic behaviour from abort to unwind

### Fixed

* Android: Fix crashes in ElementX on config changes
* Common: Fix dom location length with nested tags
* Common: Fix crashes when `replace_text` is called

# [0.8.0] - 2022-11-28

### Added
* Common: inline code formatting now works for selections with several nodes, also, formatting states are disabled when inline code is selected.
* Common: `DomLocation` now has `kind()` method to make finding nodes of some kind easier in ranges.
* Common: `DomIterator` can now be used in a sub-tree of the DOM (from an internal node instead of the root one).

### Fixed:

* Common: when replacing text at the end of a link node, the new text is added to the next text node if exists instead of the link node.
* Android: fixed formatting disappearing from the last typed word when adding a whitespace.
* Web: Handle insertCompositionText as if it were insertText (hopefully fixing accented characters in Element Desktop)

# [0.7.0] - 2022-11-21

### Changed

* Common: update to Rust 1.65 and uniffi-rs 0.21.0.
* Common: internal refactor.
* Common: MenuState updates are now always returned when we change some content.
* Common: new API for retrieving parent nodes: `Dom.parent(&child_handle)` and `Dom.parent_mut(&child_handle)`.

### Added

* Common: `Dom` is now iterable.
* Common: links can now be added to several nodes and updated.
* Android: add Markdown support.
* Android: add `RustErrorCollector` to be able to collect and treat any Rust errors manually in the integrating clients.
* Web: added debug mode.

### Fixed

* Android: text input is now diffed so we don't rely on composition, as it sometimes broke formatting.
* iOS: Voice dictation should work now.

# [0.6.0] - 2022-11-11

### Changed

* Common: MenuState updates now contain a single Map/Dictionary with an entry for each possible action, being either `Enabled`, `Disabled` or `Reversed`.

# [0.5.0] - 2022-11-11

### Added

* Common: initial Markdown support.
* Common: added get/set methods for Markdown text (`set_content_from_markdown`, `get_content_as_markdown`). Also added a getter for HTML contents (`get_content_as_html`).
* iOS: added plain text mode with Markdown support.
* iOS: expose `maxExpandedHeight` and `maxCompressedHeight` properties in `WysiwygComposerViewModel`.
* Web: added `prettier` config to `eslint`.

### Fixed

* Common: prevent crash when deleting an emoji or other complex grapheme.
* Common: fix html5ever output when a text node contains escaped HTML entities.
* Android: fixed `TextWatcher`s being called with an empty String for every change in the composer.
* Android: fixed back system key being intercepted by the editor, preventing back navigation.
* iOS: fixed bold + italic formatting not being correctly rendered on iOS 14-15.
* iOS: fixed bug when deleting whole words with long press on backspace.
* iOS: fixed missing keystrokes when the user typed very fast.
* iOS: fixed the editor contents being cleared when plain text mode was enabled.

### Changed

* Common: `replace_all_html` is now `set_content_from_html`.
* Web: use native `DOMParser` instead of `html5ever` to parse HTML. This should decrease the WASM binary size.
* Web: reduced WASM binary size by 76%.

# [0.4.0] - 2022-10-26

### Added

-   Android: Add plain text APIs

### Fixed

-   Android: Fix issue with hardware backspace key

# [0.3.2] - 2022-10-24

### Added

-   Web: `useWysiwyg` hook can be initialized with a content

### Fixed

-   Web: Fix losing selection after Ctrl-a Ctrl-b

## [0.3.1] - 2022-10-19

### Added

-   iOS: Show placeholder text

### Fixed

-   Web: allow instantiating multiple composers
-   Android: code improvements

## [0.3.0] - 2022-10-19

### Added

-   Web: Allow pressing Enter to send message

### Fixed

-   iOS: use correct fonts

## [0.2.1] - 2022-10-17

### Added

-   iOS: add support for focused state.
-   Android: handle cut & paste events properly.

### Changed

-   Android: only crash on composer exceptions when using DEBUG mode.

### Fixed

-   iOS: use right cursor color and fix blinking issue when replacing text.
-   Fix inserting characters or new lines before a new line at index 0.
-   Android: fix formatting not being replaced at index 0 when using hardware
    keyboard.

### Removed

-   iOS: remove unneeded UIKit integration code.

## [0.2.0] - 2022-10-13

### Added

-   Web: Add formatting states
-   Web: Remove onChange handler and return the content instead

## [0.1.0] - 2022-10-11

### Added

-   Web: support cut and paste
-   Document release process
-   NPM releases via a manual github workflow

## [0.0.2] - 2022-10-10

### Added

-   Improve React integration

## [0.0.1] - 2022-10-10

### Added

-   First attempt at packaging for NPM
-   Basic text editing (newlines, bold, italic etc. formatting)
-   Draft support for lists
-   Draft support for links
