use crate::dom::nodes::dom_node::DomNodeKind;
use crate::dom::nodes::dom_node::DomNodeKind::{Generic, Paragraph};
use crate::dom::{DomLocation, Range};
use crate::{ComposerModel, DomHandle, DomNode, UnicodeString};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn new_line(&mut self) {
        self.push_state_to_history();
        self.do_new_line();
    }

    fn do_new_line(&mut self) {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);

        // No selection, just exit
        if range.locations.is_empty() {
            return;
        }

        // If the selection covered several characters, remove them first
        if range.is_selection() {
            self.replace_text(S::default());
        }

        let block_location = range.deepest_block_node(None).expect(
            "No block node selected (at least the root one should be here)",
        );

        let first_leaf = range.leaves().next();
        match block_location.kind {
            Paragraph => {
                let ancestor_block_location = range.deepest_block_node(Some(
                    block_location.node_handle.clone(),
                ));
                if let Some(ancestor_block_location) = ancestor_block_location {
                    if ancestor_block_location.kind != Generic {
                        self.do_new_line_in_block_node(
                            first_leaf,
                            &block_location,
                            &ancestor_block_location,
                        );
                    } else {
                        self.do_new_line_in_paragraph(
                            first_leaf,
                            &block_location,
                        );
                    }
                } else {
                    self.do_new_line_in_paragraph(first_leaf, &block_location);
                }
            }
            DomNodeKind::Generic => {
                self.do_new_line_in_paragraph(first_leaf, &block_location);
            }
            _ => panic!("Unexpected kind block node with inline contents"),
        }
    }

    fn do_new_line_in_paragraph(
        &mut self,
        first_leaf: Option<&DomLocation>,
        paragraph_location: &DomLocation,
    ) {
        if let Some(first_leaf) = first_leaf {
            let block_node_handle = paragraph_location.node_handle.clone();
            let block_node_is_paragraph =
                self.state.dom.lookup_node(&block_node_handle).kind()
                    == Paragraph;
            let child_count = self
                .state
                .dom
                .lookup_node(&block_node_handle)
                .as_container()
                .unwrap()
                .children()
                .len();
            let last_child_handle =
                block_node_handle.child_handle(child_count - 1);

            // Wrap the contents of the "right" sub tree into a paragraph and insert it
            let mut sub_tree = self.state.dom.split_sub_tree_between(
                &first_leaf.node_handle,
                first_leaf.start_offset,
                &last_child_handle,
                usize::MAX,
                block_node_handle.depth(),
            );
            let sub_tree_container = sub_tree.document_mut();

            let cur_block_node_was_removed =
                !self.state.dom.contains(&block_node_handle);

            let mut children = sub_tree_container.remove_children();
            let new_paragraph =
                if children.first().map_or(false, |n| n.kind() == Paragraph) {
                    children.remove(0)
                } else {
                    DomNode::new_paragraph(children)
                };
            let depth = if block_node_is_paragraph {
                block_node_handle.depth()
            } else {
                block_node_handle.depth() + 1
            };
            let mut new_paragraph_handle =
                first_leaf.node_handle.sub_handle_up_to(depth);
            if self.state.dom.contains(&new_paragraph_handle) {
                new_paragraph_handle = new_paragraph_handle.next_sibling();
            }
            self.state
                .dom
                .insert_at(&new_paragraph_handle, new_paragraph);
            self.state.advance_selection();

            // Now do the same for any children remaining in the tree
            if !block_node_is_paragraph {
                let DomNode::Container(block_container) =
                    self.state.dom.lookup_node_mut(&block_node_handle) else {
                    panic!("Block container must be a container node");
                };
                let mut children = Vec::new();
                for _ in 0..new_paragraph_handle.index_in_parent() {
                    children.push(block_container.remove_child(0));
                }
                let new_paragraph = DomNode::new_paragraph(children);
                block_container.insert_child(0, new_paragraph);
            } else if block_node_is_paragraph && cur_block_node_was_removed {
                let new_paragraph = DomNode::new_paragraph(Vec::new());
                self.state.dom.insert_at(&block_node_handle, new_paragraph);
                // self.state.advance_selection();
            }
        } else {
            // Just add a new paragraph before the current block
            self.state.dom.insert_at(
                &paragraph_location.node_handle,
                DomNode::new_paragraph(Vec::new()),
            );
            self.state.advance_selection();
        }
    }

    fn do_new_line_in_block_node(
        &mut self,
        first_leaf: Option<&DomLocation>,
        paragraph_location: &DomLocation,
        ancestor_block_location: &DomLocation,
    ) {
        if first_leaf.is_some() {
            self.do_new_line_in_paragraph(first_leaf, paragraph_location);
        } else {
            // let needs_to_exit_block =
            //     if let Some(handle) = paragraph_location.node_handle {
            //         self.needs_to_exit_block(
            //             &ancestor_block_location.node_handle,
            //             &handle,
            //         )
            //     } else {
            //         false
            //     };

            // Remove existing empty paragraph
            self.state.dom.remove(&paragraph_location.node_handle);
            let mut sub_tree = self.state.dom.split_sub_tree_from(
                &paragraph_location.node_handle,
                0,
                ancestor_block_location.node_handle.depth(),
            );
            let sub_tree_container = &mut sub_tree.document();

            let insert_at = if self
                .state
                .dom
                .contains(&ancestor_block_location.node_handle)
            {
                if ancestor_block_location.start_offset > 0 {
                    ancestor_block_location.node_handle.next_sibling()
                } else {
                    ancestor_block_location.node_handle.clone()
                }
            } else {
                ancestor_block_location.node_handle.clone()
            };

            if !sub_tree_container.is_empty() {
                self.state
                    .dom
                    .insert_at(&insert_at, sub_tree.remove(&DomHandle::root()));
            }

            self.state
                .dom
                .insert_at(&insert_at, DomNode::new_paragraph(Vec::new()));

            self.state.advance_selection();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tests::testutils_composer_model::{cm, tx};
    use crate::DomHandle;

    #[test]
    fn test_new_line_in_plain_text() {
        let mut model = cm("Test| lines");
        model.new_line();
        assert_eq!(tx(&model), "<p>Test</p><p>| lines</p>");
    }

    #[test]
    fn test_new_line_at_start() {
        let mut model = cm("|Test lines");
        model.new_line();
        assert_eq!(tx(&model), "<p></p><p>|Test lines</p>");
    }

    #[test]
    fn test_new_line_at_end() {
        let mut model = cm("Test lines|");
        model.new_line();
        assert_eq!(tx(&model), "<p>Test lines</p><p>|</p>");
    }

    #[test]
    fn test_new_line_in_formatted_text() {
        let mut model = cm("<b>Test| lines</b>");
        model.new_line();
        assert_eq!(tx(&model), "<p><b>Test</b></p><p><b>| lines</b></p>");
    }

    #[test]
    fn test_new_line_in_paragraph() {
        let mut model = cm("<p>Test| lines</p>");
        model.new_line();
        assert_eq!(tx(&model), "<p>Test</p><p>| lines</p>");
    }

    #[test]
    fn selection_in_paragraphs_roundtrips() {
        let model = cm("<p>A</p><p>|B</p>");
        assert_eq!(tx(&model), "<p>A</p><p>|B</p>");
    }

    #[test]
    fn selection_in_paragraphs_roundtrips_2() {
        let model = cm("<blockquote><p>A</p><p>|B</p></blockquote>");
        assert_eq!(tx(&model), "<blockquote><p>A</p><p>|B</p></blockquote>");
    }

    #[test]
    fn adds_line_break_with_simple_paragraphs() {
        let model = cm("<p>|A</p><p>test</p>");
        let dom = model.state.dom;
        assert!(dom.adds_line_break(&DomHandle::from_raw(vec![0])));
        assert!(!dom.adds_line_break(&DomHandle::from_raw(vec![1])));
    }

    #[test]
    fn adds_line_break_with_nested_block_nodes() {
        let model = cm("<blockquote><p>|A</p></blockquote><p>test</p>");
        let dom = model.state.dom;
        // The internal paragraph won't add the extra line break as it's the last child
        assert!(!dom.adds_line_break(&DomHandle::from_raw(vec![0, 0])));
        // The quote will add the extra line break since it has a sibling node
        assert!(dom.adds_line_break(&DomHandle::from_raw(vec![0])));
    }

    #[test]
    fn add_line_at_start_of_paragraph() {
        let mut model = cm("<p>|Test</p>");
        model.new_line();
        assert_eq!(tx(&model), "<p></p><p>|Test</p>");
        model.select(0.into(), 0.into());
        assert_eq!(tx(&model), "<p>|</p><p>Test</p>");
        model.new_line();
        assert_eq!(tx(&model), "<p></p><p>|</p><p>Test</p>");
    }

    #[test]
    fn add_line_at_start_of_empty_paragraph() {
        let mut model = cm("<p>|</p><p>Test</p>");
        model.new_line();
        assert_eq!(tx(&model), "<p></p><p>|</p><p>Test</p>");
    }

    #[test]
    fn repeated_line_breaks_in_quote_split_it() {
        let mut model = cm("<blockquote><p>First|Second</p></blockquote>");
        model.new_line();
        assert_eq!(
            tx(&model),
            "<blockquote><p>First</p><p>|Second</p></blockquote>"
        );
        model.new_line();
        assert_eq!(
            tx(&model),
            "<blockquote><p>First</p><p></p><p>|Second</p></blockquote>"
        );
        model.select(6.into(), 6.into());
        model.new_line();
        assert_eq!(tx(&model), "<blockquote><p>First</p></blockquote><p>|</p><blockquote><p>Second</p></blockquote>");
    }

    #[test]
    fn line_break_in_quote_splits_quote() {
        let mut model =
            cm("<blockquote><p>First</p><p>|</p><p>Second</p></blockquote>");
        model.new_line();
        assert_eq!(
            tx(&model),
            "<blockquote><p>First</p></blockquote><p>|</p><blockquote><p>Second</p></blockquote>"
        );
    }
}
