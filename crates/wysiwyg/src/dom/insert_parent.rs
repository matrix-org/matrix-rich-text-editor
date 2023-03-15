use crate::{DomHandle, DomNode, UnicodeString};

use super::{Dom, Range};

impl<S> Dom<S>
where
    S: UnicodeString,
{
    #[allow(dead_code)]
    /// Insert a node and make this node the parent of a given range.
    pub fn insert_parent(
        &mut self,
        range: &Range,
        mut new_node: DomNode<S>,
    ) -> DomHandle {
        #[cfg(any(test, feature = "assert-invariants"))]
        self.assert_invariants();

        if range.is_empty() {
            panic!("Cannot add a parent to an empty range");
        }

        if !matches!(new_node, DomNode::Container(_)) {
            panic!("New node is not a container");
        }
        // Check if the range has shared ancestry. The new parent node should not contain
        // these nodes, so filter them from the range.
        let shared_depth = range.shared_parent_outside().depth();
        let range = Range::new(range.locations_from_depth(shared_depth));

        // Prepare the new parent node to have the selected range moved into it:
        // - Set the new node's handle to the start of the range
        // - Get the mutable container node to move the selected range into
        let new_handle: DomHandle = range.node_handles().min().unwrap().clone();
        let first_leaf = range.leaves().min().unwrap();
        let new_handle = if first_leaf.is_covered()
            || (first_leaf.ends_inside() && first_leaf.is_end())
        {
            new_handle
        } else {
            // If the first leaf in the range is completely covered, then leave space
            // for the first part of the existing split leaf.
            new_handle.next_sibling()
        };
        new_node.set_handle(new_handle.clone());
        let container = new_node.as_container_mut().unwrap();

        // Remove the selected nodes from the DOM and add them to the new container
        for location in range.locations.iter().rev() {
            let handle = &location.node_handle;

            // If the location is covered, it can be moved to the container.
            if location.is_covered() {
                // Ignore children of covered nodes which are moved in their parents.
                if handle.depth() != shared_depth + 1 {
                    continue;
                }
                let node = self.remove(handle);
                container.insert_child(0, node);
            }
            // ... else handle partially covered nodes
            else {
                // If the a leaf is partially included at the start of the range, split the tree
                // and add the end part to the new container.
                // If the leaf is partially included at the end of the range, split the tree and
                // add the start part to the new container.
                if !self.lookup_node(handle).is_leaf() {
                    continue;
                }

                let offset = if location.ends_inside() {
                    location.end_offset
                } else {
                    location.start_offset
                };

                let (left, left_handle, right, right_handle) =
                    self.split_new_sub_trees(handle, offset, shared_depth);

                // `outers` are the new subtrees outside the given range
                let mut outers = if location.ends_inside() {
                    vec![right.lookup_node(&right_handle).clone()]
                } else {
                    vec![left.lookup_node(&left_handle).clone()]
                };

                // `inner` is the new subtree within the given range
                let mut inner =
                    if location.ends_inside() { left } else { right };

                // If the range both starts and ends inside this leaf then
                // split the leaf again. We've already split the end off so
                // now we need to split the start.
                if location.ends_inside() && location.starts_inside() {
                    let offset = location.start_offset;
                    let before = inner
                        .document_mut()
                        .slice_before(offset)
                        .take_children();
                    outers.splice(0..0, before);
                }
                let inner_children = inner.into_container().take_children();

                container.insert_children(0, inner_children);
                self.replace(handle, outers);
            }
        }

        // Insert the new container into the DOM
        let inserted = self.insert_at(&new_handle, new_node);

        #[cfg(any(test, feature = "assert-invariants"))]
        self.assert_invariants();

        inserted
    }
}

#[cfg(test)]
mod test {
    use crate::{
        tests::{testutils_composer_model::cm, testutils_conversion::utf16},
        DomNode, ToHtml,
    };

    #[test]
    fn insert_parent_flat_part() {
        let mut model = cm("A{B}|C");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], &None),
        );

        assert_eq!(model.state.dom.to_html(), r#"A<a href="link">B</a>C"#)
    }

    #[test]
    fn insert_parent_flat_start() {
        let mut model = cm("{AB}|C");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], &None),
        );

        assert_eq!(model.state.dom.to_html(), r#"<a href="link">AB</a>C"#)
    }

    #[test]
    fn insert_parent_flat_end() {
        let mut model = cm("A{BC}|");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], &None),
        );

        assert_eq!(model.state.dom.to_html(), r#"A<a href="link">BC</a>"#)
    }

    #[test]
    fn insert_parent_flat_whole() {
        let mut model = cm("{ABC}|");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], &None),
        );

        assert_eq!(model.state.dom.to_html(), r#"<a href="link">ABC</a>"#)
    }

    #[test]
    fn insert_parent_simple() {
        let mut model = cm("{<b>AB</b><i><u>C</u></i>}|");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], &None),
        );

        assert_eq!(
            model.state.dom.to_html(),
            r#"<a href="link"><b>AB</b><i><u>C</u></i></a>"#
        )
    }

    #[test]
    fn insert_parent_ignores_adjacent_nodes() {
        let mut model = cm("X{<b>AB</b><i><u>C</u></i>}|<u>D</u>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], &None),
        );

        assert_eq!(
            model.state.dom.to_html(),
            r#"X<a href="link"><b>AB</b><i><u>C</u></i></a><u>D</u>"#
        )
    }

    #[test]
    fn insert_parent_splits_single_partially_covered_node() {
        let mut model = cm("<b><u>XX{ABC}|YY</u></b>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], &None),
        );

        assert_eq!(
            model.state.dom.to_html(),
            r#"<b><u>XX<a href="link">ABC</a>YY</u></b>"#
        )
    }

    #[test]
    fn insert_parent_splits_multiple_partially_covered_nodes() {
        let mut model = cm("XX{A<b>B</b><u>C}|YY</u>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], &None),
        );

        assert_eq!(
            model.state.dom.to_html(),
            r#"XX<a href="link">A<b>B</b><u>C</u></a><u>YY</u>"#
        )
    }

    #[test]
    fn insert_parent_splits_multiple_nested_partially_covered_nodes() {
        let mut model = cm("<i><u>X{X</u>ABC<b>Y}|Y</b></i>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], &None),
        );

        assert_eq!(
            model.state.dom.to_html(),
            r#"<i><u>X</u><a href="link"><u>X</u>ABC<b>Y</b></a><b>Y</b></i>"#
        )
    }

    #[test]
    fn insert_parent_ignores_shared_parent_nodes() {
        let mut model = cm("<i><em>XX{A<b>B</b><u>C}|YY</u></em></i>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], &None),
        );

        assert_eq!(
            model.state.dom.to_html(),
            r#"<i><em>XX<a href="link">A<b>B</b><u>C</u></a><u>YY</u></em></i>"#
        )
    }

    #[test]
    fn insert_parent_ignores_shared_parent_nodes_with_siblings() {
        let mut model =
            cm("<u>U</u><i><em>XX{A<b>B</b><u>C}|YY</u></em></i><u>W</u>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], &None),
        );

        assert_eq!(
            model.state.dom.to_html(),
            r#"<u>U</u><i><em>XX<a href="link">A<b>B</b><u>C</u></a><u>YY</u></em></i><u>W</u>"#
        )
    }

    #[test]
    fn insert_parent_includes_covered_shared_parent_nodes() {
        let mut model = cm("<i><em>{A<b>B</b><u>C</u>}|</em>D</i>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_parent(
            &range,
            DomNode::new_link(utf16("link"), vec![], &None),
        );

        assert_eq!(
            model.state.dom.to_html(),
            r#"<i><a href="link"><em>A<b>B</b><u>C</u></em></a>D</i>"#
        )
    }

    #[test]
    #[should_panic]
    fn insert_parent_panics_if_new_is_not_container() {
        let mut model = cm("{X}|");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model
            .state
            .dom
            .insert_parent(&range, DomNode::new_text(utf16("not a container")));
    }
}
