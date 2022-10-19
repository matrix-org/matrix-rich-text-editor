# Changelog

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
