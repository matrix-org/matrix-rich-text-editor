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

use crate::{
    dom::Range, web_selection::WebSelection, ComposerModel, ComposerUpdate,
    DomHandle, Location, UnicodeString,
};

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
        if self.state.start == start && self.state.end == end {
            return ComposerUpdate::keep();
        }
        self.state.toggled_format_types.clear();
        self.state.start = start;
        self.state.end = end;

        self.create_update_update_selection()
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

    /// Return a boolean to let us know if we have a selection
    pub fn has_selection(&self) -> bool {
        let (s, e) = self.safe_selection();
        s != e
    }

    /// Return a boolean to let us know if we have a cursor, ie a zero length selection
    pub fn has_cursor(&self) -> bool {
        let (s, e) = self.safe_selection();
        s == e
    }

    /// Attempt to reproduce window.getSelection as closely as practicable
    /// https://developer.mozilla.org/en-US/docs/Web/API/Window/getSelection
    ///
    /// Returns the handles of the nodes, as opposed to the nodes themselves, which
    /// allows web to then find the corresponding nodes in the real dom
    pub fn get_web_selection(&self) -> WebSelection {
        let (anchor_node, anchor_offset) = self.get_anchor_node_and_offset();
        let (focus_node, focus_offset) = self.get_focus_node_and_offset();

        WebSelection {
            anchor_node,
            anchor_offset,
            focus_node,
            focus_offset,
            is_collapsed: self.has_cursor(),
        }
    }

    fn get_anchor_node_and_offset(&self) -> (DomHandle, usize) {
        let range = self
            .state
            .dom
            .find_range(self.state.start.into(), self.state.start.into());

        self.get_node_and_offset(range)
    }

    fn get_focus_node_and_offset(&self) -> (DomHandle, usize) {
        let range = self
            .state
            .dom
            .find_range(self.state.end.into(), self.state.end.into());

        self.get_node_and_offset(range)
    }

    fn get_node_and_offset(&self, range: Range) -> (DomHandle, usize) {
        range
            .locations
            .iter()
            .find_map(|l| {
                if l.start_offset <= l.length {
                    Some((l.node_handle.clone(), l.start_offset))
                } else {
                    None
                }
            })
            .unwrap_or((DomHandle::new_unset(), 0))
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

    /**
     * get_anchor_node_and_offset
     */
    #[test]
    fn finds_anchor_and_offset_plain_text() {
        let model = cm("hello |there");

        let (anchor_node, offset) = model.get_anchor_node_and_offset();

        assert_eq!(anchor_node.raw(), &vec![0]);
        assert_eq!(offset, 6)
    }

    #[test]
    fn finds_anchor_and_offset_nested() {
        let model = cm("<p>hi <em>there|</em></p>");

        let (anchor_node, offset) = model.get_anchor_node_and_offset();

        assert_eq!(anchor_node.raw(), &vec![0, 1, 0]);
        assert_eq!(offset, 5)
    }

    /**
     * get_focus_node_and_offset
     */
    #[test]
    fn finds_focus_and_offset_plain_text() {
        let model = cm("hello {there}|");

        let (anchor_node, offset) = model.get_focus_node_and_offset();

        assert_eq!(anchor_node.raw(), &vec![0]);
        assert_eq!(offset, 11)
    }

    #[test]
    fn finds_focus_and_offset_nested() {
        let model = cm("<p>h{i <em>there}|</em></p>");

        let (anchor_node, offset) = model.get_focus_node_and_offset();

        assert_eq!(anchor_node.raw(), &vec![0, 1, 0]);
        assert_eq!(offset, 5)
    }
}
