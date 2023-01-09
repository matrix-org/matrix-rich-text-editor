use crate::dom::nodes::dom_node::DomNodeKind::Paragraph;
use crate::dom::DomLocation;
use crate::{ComposerModel, DomHandle, DomNode, ToHtml, ToTree, UnicodeString};

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
        let leaves: Vec<&DomLocation> = range.leaves().into_iter().collect();

        if leaves.is_empty() {
            return;
        }

        // If the selection covered several characters, remove them first
        if range.is_selection() {
            self.replace_text(S::default());
        }

        let first_leaf = leaves[0];
        let block_node_handle = self
            .state
            .dom
            .find_block_ancestor_to_split(&first_leaf.node_handle);
        let block_node_is_paragraph =
            self.state.dom.lookup_node(&block_node_handle).kind() == Paragraph;
        let child_count = self
            .state
            .dom
            .lookup_node(&block_node_handle)
            .as_container()
            .unwrap()
            .children()
            .len();
        let last_child_handle = block_node_handle.child_handle(child_count - 1);

        // Wrap the contents of the "right" sub tree into a paragraph and insert it
        let sub_tree = self.state.dom.split_sub_tree_between(
            &first_leaf.node_handle,
            first_leaf.start_offset,
            &last_child_handle,
            usize::MAX,
            block_node_handle.depth(),
        );
        let DomNode::Container(mut sub_tree_container) = sub_tree else {
            panic!("Sub tree must be a container node");
        };
        let mut children = sub_tree_container.take_children();
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
        // Update selection if selection was not at the start of the block node
        let block_node_location = range.find_location(&block_node_handle);
        let selection_is_at_start = if block_node_location.is_some() {
            block_node_location.unwrap().start_offset == 0
        } else {
            first_leaf.index_in_dom() == 0
        };
        if !selection_is_at_start {
            self.state.advance_selection();
        }

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
        }
    }
}

#[cfg(test)]
mod test {
    use crate::dom::nodes::dom_node::DomNodeKind;
    use crate::tests::testutils_composer_model::{cm, tx};
    use crate::DomHandle;
    use std::ops::Not;

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
        assert_eq!(tx(&model), "<p>|</p><p>Test lines</p>");
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
    fn adds_line_break_with_single_paragraph_returns_true() {
        let model = cm("<p>|A</p>");
        assert!(model
            .state
            .dom
            .adds_line_break(&DomHandle::from_raw(vec![0])));
    }

    #[test]
    fn adds_line_break_with_nested_paragraph_returns_false() {
        let model = cm("<blockquote><p>|A</p></blockquote>");
        // The paragraph won't add the extra line break
        assert!(model
            .state
            .dom
            .adds_line_break(&DomHandle::from_raw(vec![0, 0]))
            .not());
        // The quote will add the extra line break
        assert!(model
            .state
            .dom
            .adds_line_break(&DomHandle::from_raw(vec![0])));
    }
}
