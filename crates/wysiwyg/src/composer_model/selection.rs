// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{ComposerModel, ComposerUpdate, Location, UnicodeString};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    /// Select the text at the supplied code unit positions.
    /// The cursor is at end.
    pub fn select(
        &mut self,
        start: Location,
        end: Location,
    ) -> ComposerUpdate<S> {
        self.toggled_format_types.clear();
        self.state.start = start;
        self.state.end = end;

        let menu_state = self.compute_menu_state();
        ComposerUpdate::update_selection(start, end, menu_state)
    }

    /// Return the start and end of the selection, ensuring the first number
    /// returned is <= the second, and they are both between 0 and the number
    /// of code units in the string representation of the Dom.
    pub(crate) fn safe_selection(&self) -> (usize, usize) {
        self.safe_locations_from(self.state.start, self.state.end)
    }

    pub(crate) fn safe_locations_from(
        &self,
        start: Location,
        end: Location,
    ) -> (usize, usize) {
        let len = self.state.dom.text_len();

        let mut s: usize = start.into();
        let mut e: usize = end.into();
        s = s.clamp(0, len);
        e = e.clamp(0, len);
        if s > e {
            (e, s)
        } else {
            (s, e)
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::tests::testutils_composer_model::cm;

    #[test]
    fn safe_selection_leaves_forward_selection_untouched() {
        let model = cm("out{ <b>bol}|d</b> spot");
        assert_eq!((3, 7), model.safe_selection());
    }

    #[test]
    fn safe_selection_reverses_backward_selection() {
        let model = cm("out|{ <b>bol}d</b> spot");
        assert_eq!((3, 7), model.safe_selection());
    }

    #[test]
    fn safe_selection_fixes_too_wide_selection() {
        let mut model = cm("out <b>bol</b> spot|");
        model.state.start = Location::from(0);
        model.state.end = Location::from(13);
        assert_eq!((0, 12), model.safe_selection());

        let mut model = cm("out <b>bol</b> {spot}|");
        model.state.end = Location::from(33);
        assert_eq!((8, 12), model.safe_selection());
    }
}
