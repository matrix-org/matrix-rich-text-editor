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

use crate::dom::nodes::dom_node::DomNodeKind::*;
use crate::dom::nodes::{ContainerNode, ContainerNodeKind, DomNode};
use crate::dom::{DomHandle, DomLocation, Range};
use crate::{ComposerAction, ComposerModel, ComposerUpdate, UnicodeString};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn code_block(&mut self) -> ComposerUpdate<S> {
        if self.action_is_reversed(ComposerAction::CodeBlock) {
            self.remove_code_block()
        } else {
            self.add_code_block()
        }
    }

    fn add_code_block(&mut self) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        let Some(wrap_result) = self.state.dom.find_nodes_to_wrap_in_block(s, e) else {
            // No suitable nodes found to be wrapped inside the code block. Add an empty block.
            let range = self.state.dom.find_range(s, e);
            let leaves: Vec<&DomLocation> = range.leaves().collect();
            let node = DomNode::new_code_block(vec![DomNode::new_paragraph(Vec::new())]);
            if leaves.is_empty() {
                if let Some(deepest_block_location) = range.deepest_block_node(None) {
                    let block_node = self.state.dom.remove(&deepest_block_location.node_handle);
                    let node = DomNode::new_code_block(vec![block_node]);
                    self.state.dom.insert_at(&deepest_block_location.node_handle, node);
                } else {
                    self.state.dom.append_at_end_of_document(node);
                }
            } else {
                let first_leaf_loc = leaves.first().unwrap();
                let insert_at = if first_leaf_loc.is_start() {
                    first_leaf_loc.node_handle.next_sibling()
                } else {
                   first_leaf_loc.node_handle.clone()
                };
                self.state.dom.insert_at(&insert_at, node);
            }
            return self.create_update_replace_all();
        };
        let parent_handle = wrap_result.ancestor_handle;
        let start_handle = wrap_result.start_handle;
        let end_handle = wrap_result.end_handle;
        let range = wrap_result.range;
        let leaves: Vec<&DomLocation> = range.leaves().collect();
        let first_leaf = leaves.first().unwrap();
        let last_leaf = leaves.last().unwrap();

        let mut subtree = self.state.dom.split_sub_tree_between(
            &start_handle,
            0,
            &end_handle,
            usize::MAX,
            parent_handle.depth(),
        );

        let mut children: Vec<DomNode<S>> = Vec::new();
        let subtree_container = subtree.document_mut();
        while !subtree_container.children().is_empty() {
            let last_child = subtree_container
                .remove_child(subtree_container.children().len() - 1);

            let mut new_children = self.format_node_for_code_block(
                &last_child,
                &range,
                first_leaf,
                last_leaf,
            );
            new_children.extend(children);
            children = new_children;
        }

        let insert_at_handle =
            self.state.dom.find_insert_handle_for_extracted_block_node(
                &start_handle,
                &parent_handle,
                &subtree.document_node(),
            );

        let code_block = DomNode::new_code_block(children);
        if subtree.document_node().kind() == ListItem {
            self.state.dom.insert_at(
                &insert_at_handle,
                DomNode::new_list_item(vec![code_block]),
            );
        } else {
            self.state.dom.insert_at(&insert_at_handle, code_block);
        }

        // Merge any nodes that need it
        self.merge_adjacent_code_blocks(&insert_at_handle);

        self.create_update_replace_all()
    }

    fn merge_adjacent_code_blocks(&mut self, handle: &DomHandle) -> DomHandle {
        let mut handle = handle.clone();
        // If there is a next code block, add its contents to the current one and remove it
        if let Some(next_code_block_handle) = self
            .state
            .dom
            .next_sibling(&handle)
            .filter(|n| n.kind() == CodeBlock)
            .map(|n| n.handle())
        {
            self.state.dom.move_children_and_delete_parent(
                &next_code_block_handle,
                &handle,
            );
        }

        // If there is a previous code block, add the contents of the current one to it and remove it
        if let Some(prev_code_block_handle) = self
            .state
            .dom
            .prev_sibling(&handle)
            .filter(|n| n.kind() == CodeBlock)
            .map(|n| n.handle())
        {
            self.state.dom.move_children_and_delete_parent(
                &handle,
                &prev_code_block_handle,
            );
            handle = prev_code_block_handle;
        }

        // Join any nodes inside the current code block
        self.state.dom.join_nodes_in_container(&handle);

        handle
    }

    fn remove_code_block(&mut self) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        let Some(block_location) = range.locations.iter().find(|l| l.kind == CodeBlock) else {
            return ComposerUpdate::keep();
        };

        self.state
            .dom
            .remove_and_keep_children(&block_location.node_handle);

        self.create_update_replace_all()
    }

    /// Converts any nodes to be added to a code block to the right format, recursively.
    /// Line breaks get turned into `\n` chars.
    /// Text nodes are just cloned.
    /// Container nodes will be added to the code block and their contents will also be formatted:
    /// Block nodes and list items will add extra line breaks (`\n` chars).
    pub(crate) fn format_node_for_code_block(
        &mut self,
        node: &DomNode<S>,
        range: &Range,
        first_leaf: &DomLocation,
        last_leaf: &DomLocation,
    ) -> Vec<DomNode<S>> {
        // TODO: try to diff node positions and offsets in a more straightforward way
        match node {
            DomNode::Container(container) => self
                .format_container_node_for_code_block(
                    container, range, first_leaf, last_leaf,
                ),
            node => vec![node.clone()],
        }
    }

    fn format_container_node_for_code_block(
        &mut self,
        container: &ContainerNode<S>,
        range: &Range,
        first_leaf: &DomLocation,
        last_leaf: &DomLocation,
    ) -> Vec<DomNode<S>> {
        let mut children = Vec::new();
        // We process each child node
        for c in container.children() {
            children.extend(
                self.format_node_for_code_block(
                    c, range, first_leaf, last_leaf,
                ),
            );
        }

        if container.is_list_item() {
            vec![DomNode::new_paragraph(children)]
        } else if matches!(container.kind(), ContainerNodeKind::Paragraph) {
            vec![DomNode::new_paragraph(children)]
        } else if container.is_block_node() {
            children
        } else {
            vec![DomNode::Container(
                container.clone_with_new_children(children),
            )]
        }
    }

    fn format_list_for_code_block(
        &mut self,
        handle: &DomHandle,
        is_before: bool,
        children: &mut Vec<DomNode<S>>,
    ) {
        // Add line break before the List if it's not at the start
        if handle.index_in_parent() > 0 {
            children.insert(0, DomNode::new_text("\n".into()));
            if is_before {
                self.state.start += 1;
            }
        }
    }

    fn format_list_item_for_code_block(
        &mut self,
        is_before: bool,
        is_after: bool,
        children: &mut Vec<DomNode<S>>,
    ) {
        // Every list item adds a line break at the end
        // So "<ul><li>Item A</li><li>Item B</li></ul>End"
        // Will become: "<pre>Item A\nItem B\nEnd</pre>"
        children.push(DomNode::new_text("\n".into()));
        if is_before {
            self.state.start += 1;
        }
        if !is_after {
            self.state.end += 1;
        }
    }

    fn format_quote_for_code_block(
        &mut self,
        handle: &DomHandle,
        range: &Range,
        is_in_selection: bool,
        children: &mut Vec<DomNode<S>>,
    ) {
        // We add a new line where it's needed
        let added_at_start = if handle.index_in_parent() == 0 {
            children.push(DomNode::new_text("\n".into()));
            false
        } else {
            children.insert(0, DomNode::new_text("\n".into()));
            true
        };
        if is_in_selection {
            let location = range.find_location(handle).unwrap();
            if location.starts_inside() && location.ends_inside() {
                // Do nothing
            } else if location.starts_inside() {
                if added_at_start {
                    self.state.start += 1;
                }
                self.state.end += 1;
            } else if location.ends_inside() {
                self.state.start += 1;
                self.state.end += 1;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tests::testutils_composer_model::{cm, tx};
    use indoc::indoc;

    #[test]
    fn code_block_roundtrips() {
        let model = cm("<pre>Test|\nCode</pre>");
        // <pre> internally works as any other block node, with paragraphs
        let tree = model.to_tree().to_string();
        let expected_tree = indoc! { r#"
        
            └>pre
              ├>p
              │ └>"Test"
              └>p
                └>"Code"
        "#};
        assert_eq!(tree, expected_tree);
        // But it gets translated back to the proper HTML output
        assert_eq!(tx(&model), "<pre>Test|\nCode</pre>");
    }

    #[test]
    fn add_code_block_to_empty_dom() {
        let mut model = cm("|");
        model.code_block();
        assert_eq!(tx(&model), "<pre>|</pre>");
    }

    #[test]
    fn add_code_block_to_simple_text() {
        let mut model = cm("Some text|");
        model.code_block();
        assert_eq!(tx(&model), "<pre>Some text|</pre>");
    }

    #[test]
    fn add_code_block_to_several_nodes() {
        let mut model = cm("Some text| <b>and bold </b><i>and italic</i>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "<pre>Some text| <b>and bold </b><i>and italic</i></pre>"
        );
    }

    #[test]
    fn add_code_block_to_several_nodes_in_single_paragraph() {
        let mut model =
            cm("<p>Some text| <b>and bold </b></p><p><i>and italic</i></p>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "<pre>Some text| <b>and bold </b></pre><p><i>and italic</i></p>"
        );
    }

    #[test]
    fn add_code_block_to_list_item() {
        let mut model = cm(
            "<ul><li>Some text <b>and bold </b><i>|and italic</i></li></ul>",
        );
        model.code_block();
        assert_eq!(
            tx(&model),
            "<ul><li><pre>Some text <b>and bold |</b><i>and italic</i></pre></li></ul>"
        );
    }

    #[test]
    fn add_code_block_to_list_item_with_line_breaks() {
        let mut model = cm(
            "<ul><li>Some text <b>and bold </b><br/><i>and| italic</i></li></ul>",
        );
        model.code_block();
        assert_eq!(
            tx(&model),
            "<ul><li>Some text <b>and bold&nbsp;</b><br /><pre>~<i>and| italic</i></pre></li></ul>"
        );
    }

    #[test]
    fn add_code_block_to_several_list_items() {
        let mut model =
            cm("<ul><li>{First item</li><li>Second}| item</li></ul>");
        model.code_block();
        assert_eq!(tx(&model), "<pre>{First item\nSecond}| item</pre>");
    }

    #[test]
    fn add_code_block_to_several_lists() {
        let mut model =
            cm("<ul><li>{First item</li><li>Second item</li></ul><p>Some text</p><ul><li>Third}| item</li><li>Fourth one</li></ul>");
        model.code_block();
        assert_eq!(tx(&model), "<pre>{First item\nSecond item\nSome text\nThird}| item</pre><ul><li>Fourth one</li></ul>");
    }

    #[test]
    fn add_code_block_to_list_and_external_nodes() {
        let mut model = cm(
            "<p>{Text</p><ul><li>First item</li><li>Second}| item</li></ul>",
        );
        model.code_block();
        assert_eq!(tx(&model), "<pre>{Text\nFirst item\nSecond}| item</pre>");
    }

    #[test]
    fn add_code_block_to_existing_code_block() {
        let mut model = cm("<p>{Text</p><pre>code}|</pre>");
        model.code_block();
        assert_eq!(tx(&model), "<pre>{Text\ncode}|</pre>");
    }

    #[test]
    fn add_code_block_to_existing_code_block_partially_selected() {
        let mut model =
            cm("<p>{Text</p><pre><b>code}|</b><i> and italic</i></pre>");
        model.code_block();
        let tree = model.to_tree().to_string();
        assert_eq!(
            tx(&model),
            "<pre>{Text\n<b>code}|</b><i> and italic</i></pre>"
        );
    }

    #[test]
    fn add_code_block_to_nested_item_in_formatting_node() {
        let mut model =
            cm("<p><b>Text</b></p><p><b><i>{in italic}|</i></b></p>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "<p><b>Text</b></p><pre><b><i>{in italic}|</i></b></pre>"
        );
    }

    #[test]
    fn add_code_block_to_deep_nested_item_in_formatting_nodes() {
        let mut model = cm(
            "<p><u><b>Text</b></u></p><p><u><b><i>{in italic}|</i></b></u></p>",
        );
        model.code_block();
        assert_eq!(
            tx(&model),
            "<p><u><b>Text</b></u></p><pre><u><b><i>{in italic}|</i></b></u></pre>"
        );
    }

    #[test]
    fn add_code_block_to_quote() {
        let mut model = cm("<blockquote><p>Quot|e</p></blockquote>");
        model.code_block();
        assert_eq!(tx(&model), "<pre>Quot|e</pre>");
    }

    #[test]
    fn add_code_block_to_quote_text_before() {
        let mut model =
            cm("<p>Te{xt </p><blockquote><p>Quot}|e</p></blockquote>");
        model.code_block();
        assert_eq!(tx(&model), "<pre>Te{xt \nQuot}|e</pre>");
    }

    #[test]
    fn add_code_block_to_quote_text_after() {
        let mut model =
            cm("<blockquote><p>Quo{te</p></blockquote><p>Te}|xt</p>");
        model.code_block();
        assert_eq!(tx(&model), "<pre>Quo{te\nTe}|xt</pre>");
    }

    #[test]
    fn remove_code_block_moves_its_children_out() {
        let mut model =
            cm("<p>Text</p><pre><b>code|</b><i> and italic</i></pre>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "<p>Text</p><p><b>code|</b><i> and italic</i></p>"
        );
    }

    #[test]
    fn remove_code_block_moves_its_children_and_restores_line_breaks() {
        let mut model = cm("<p>Text</p><pre>with|\nline\nbreaks</pre>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "<p>Text</p><p>with|</p><p>line</p><p>breaks</p>"
        );
    }

    #[test]
    fn remove_code_block_moves_its_children_and_keeps_selection_in_place() {
        let mut model = cm("<p>Text</p><pre>wi{th\nline\nbrea}|ks</pre>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "<p>Text</p><p>wi{th</p><p>line</p><p>brea}|ks</p>"
        );
    }

    #[test]
    fn test_creating_code_block_at_the_end_of_editor() {
        let mut model = cm("<p>Test</p><p>|</p>");
        model.code_block();
        assert_eq!(tx(&model), "<p>Test</p><pre>|</pre>");
    }

    #[test]
    fn creating_and_removing_code_block_works() {
        let mut model = cm("|");
        model.code_block();
        assert_eq!(tx(&model), "<pre>|</pre>");
        model.code_block();
        assert_eq!(tx(&model), "<p>|</p>");
    }
}
