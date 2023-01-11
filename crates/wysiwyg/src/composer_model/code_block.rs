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
use crate::dom::unicode_string::{UnicodeStr, UnicodeStrExt};
use crate::dom::{DomHandle, DomLocation, Range};
use crate::{
    ComposerAction, ComposerModel, ComposerUpdate, ToHtml, UnicodeString,
};

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
            if leaves.is_empty() {
                self.state.dom.append_at_end_of_document(DomNode::new_code_block(vec![DomNode::new_zwsp()]));
            } else {
                let first_leaf_loc = leaves.first().unwrap();
                let insert_at = if first_leaf_loc.is_start() {
                    first_leaf_loc.node_handle.next_sibling()
                } else {
                   first_leaf_loc.node_handle.clone()
                };
                self.state.dom.insert_at(&insert_at, DomNode::new_code_block(vec![DomNode::new_zwsp()]));
            }
            self.state.start += 1;
            self.state.end += 1;
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
        let subtree_container = subtree.document_mut();

        let mut children: Vec<DomNode<S>> = Vec::new();
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

        // TODO: improve detection? Not sure if trailing line break will always be at the top level
        if let Some(DomNode::Text(text_node)) = children.last_mut() {
            if text_node.to_html().to_string().ends_with('\n') {
                let data = text_node.data();
                let new_data = data[..data.len() - 1].to_owned();
                text_node.set_data(new_data);
            }
        }

        let insert_at_handle =
            self.state.dom.find_insert_handle_for_extracted_block_node(
                &start_handle,
                &parent_handle,
                &subtree.document_node(),
            );
        let code_block = DomNode::new_code_block(children);
        self.state.dom.insert_at(&insert_at_handle, code_block);

        // Merge any nodes that need it
        let new_code_block_handle =
            self.merge_adjacent_code_blocks(&insert_at_handle);

        if let Some(merged_code_block_container) = self
            .state
            .dom
            .lookup_node_mut(&new_code_block_handle)
            .as_container_mut()
        {
            if !merged_code_block_container.has_leading_zwsp() {
                merged_code_block_container
                    .insert_child(0, DomNode::new_zwsp());
                self.state.start += 1;
                self.state.end += 1;
            }
        }

        self.state.dom.remove_empty_container_nodes(false);

        self.create_update_replace_all()
    }

    fn merge_adjacent_code_blocks(&mut self, handle: &DomHandle) -> DomHandle {
        let mut handle = handle.clone();
        // TODO: remove intermediate ZWSP chars?
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
        let Some(code_block_location) = range.locations.iter().find(|l| l.kind == CodeBlock) else {
            return ComposerUpdate::keep();
        };

        let start_in_block = code_block_location.start_offset;
        let end_in_block = code_block_location.end_offset;
        let mut selection_offset_start = 0;
        let mut selection_offset_end = 0;

        // If we remove the whole code block and it's not the first child, we should start by adding a line break
        let has_previous_line_break = self
            .state
            .dom
            .prev_leaf(&code_block_location.node_handle)
            .map_or(false, |n| n.is_line_break());
        let mut nodes_to_add =
            if code_block_location.node_handle.index_in_parent() > 0
                && !has_previous_line_break
            {
                self.state.start += 1;
                self.state.end += 1;
                vec![DomNode::new_line_break()]
            } else {
                Vec::new()
            };
        // Just remove everything
        let DomNode::Container(block) =
            self.state.dom.lookup_node(&code_block_location.node_handle) else
        {
            panic!("CodeBlock must be a container node");
        };
        for child in block.children() {
            match child {
                DomNode::Text(text_node) => {
                    // Split the TextNode by \n and add them and a line break to the nodes to add
                    let mut text_start: usize = 0;
                    let mut text_end = text_start;
                    let data = text_node.data();
                    for char in data.chars() {
                        text_end += data.char_len(&char);
                        if char == '\n' {
                            let text_to_add =
                                data[text_start..text_end - 1].to_owned();
                            nodes_to_add.push(DomNode::new_text(text_to_add));
                            nodes_to_add.push(DomNode::new_line_break());
                            text_start = text_end;
                        }
                    }
                    // We moved to a new line
                    if text_start != text_end {
                        nodes_to_add.push(DomNode::Text(
                            text_node.clone_with_range(text_start..text_end),
                        ));
                        nodes_to_add.push(DomNode::new_line_break());
                        if text_end <= start_in_block {
                            selection_offset_start += 1;
                        }
                        if text_end <= end_in_block {
                            selection_offset_end += 1;
                        }
                    }
                }
                DomNode::Zwsp(_) => {
                    self.state.start -= 1;
                    self.state.end -= 1;
                }
                // Just move the node out
                _ => nodes_to_add.push(child.clone()),
            }
        }

        let has_next_line_break = self
            .state
            .dom
            .next_leaf(&code_block_location.node_handle)
            .map_or(false, |n| n.is_line_break());
        // Add a final line break if needed
        if let Some(last_node) = nodes_to_add.last() {
            if last_node.kind() != LineBreak && !has_next_line_break {
                nodes_to_add.push(DomNode::new_line_break());
            }
        }

        self.state
            .dom
            .replace(&code_block_location.node_handle, nodes_to_add);

        if selection_offset_start > 0 {
            self.state.start += selection_offset_start;
            self.state.end += selection_offset_end;
        }

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
            DomNode::LineBreak(_) => {
                // Just turn them into line break characters
                let mut text_node = DomNode::new_text("\n".into());
                text_node.set_handle(node.handle());
                vec![text_node]
            }
            DomNode::Text(_) => vec![node.clone()],
            DomNode::Container(container) => self
                .format_container_node_for_code_block(
                    container, range, first_leaf, last_leaf,
                ),
            DomNode::Zwsp(_) => {
                // We remove the ZWSP, then fix the selection
                self.state.start -= 1;
                self.state.end -= 1;
                Vec::new()
            }
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

        // These checks are used to verify if the changes in the Dom should also move the selection
        let handle = container.handle();
        let is_in_selection = range.contains(&handle);
        let is_before = handle <= first_leaf.node_handle && !is_in_selection;
        let is_after = handle > last_leaf.node_handle && !is_in_selection;

        // We process each child node
        for c in container.children() {
            children.extend(
                self.format_node_for_code_block(
                    c, range, first_leaf, last_leaf,
                ),
            );
        }
        // Then post-process the containers to add line breaks and move selection if needed
        if container.is_block_node() || container.is_list_item() {
            if container.is_list() {
                // Add line break before the List if it's not at the start
                self.format_list_for_code_block(
                    &handle,
                    is_before,
                    &mut children,
                );
            } else if container.is_list_item() {
                self.format_list_item_for_code_block(
                    is_before,
                    is_after,
                    &mut children,
                );
            } else if *container.kind() == ContainerNodeKind::Quote {
                self.format_quote_for_code_block(
                    &handle,
                    range,
                    is_in_selection,
                    &mut children,
                )
            }
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

    #[test]
    fn add_code_block_to_empty_dom() {
        let mut model = cm("|");
        model.code_block();
        assert_eq!(tx(&model), "<pre>~|</pre>");
    }

    #[test]
    fn add_code_block_to_simple_text() {
        let mut model = cm("Some text|");
        model.code_block();
        assert_eq!(tx(&model), "<pre>~Some text|</pre>");
    }

    #[test]
    fn add_code_block_to_several_nodes() {
        let mut model = cm("Some text| <b>and bold </b><i>and italic</i>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "<pre>~Some text| <b>and bold&nbsp;</b><i>and italic</i></pre>"
        );
    }

    #[test]
    fn add_code_block_to_several_nodes_with_line_break_at_end() {
        let mut model = cm("Some text| <b>and bold </b><br/><i>and italic</i>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "<pre>~Some text| <b>and bold&nbsp;</b></pre><br /><i>and italic</i>"
        );
    }

    #[test]
    fn add_code_block_to_several_nodes_with_line_break_at_start() {
        let mut model = cm("Some text <br/><b>and bold </b><i>and |italic</i>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "Some text <br /><pre>~<b>and bold&nbsp;</b><i>and |italic</i></pre>"
        );
    }

    #[test]
    fn add_code_block_to_list_item() {
        let mut model = cm(
            "<ul><li>~Some text <b>and bold </b><i>|and italic</i></li></ul>",
        );
        model.code_block();
        assert_eq!(
            tx(&model),
            "<ul><li><pre>~Some text <b>and bold&nbsp;|</b><i>and italic</i></pre></li></ul>"
        );
    }

    #[test]
    fn add_code_block_to_list_item_with_line_breaks() {
        let mut model = cm(
            "<ul><li>~Some text <b>and bold </b><br/><i>and| italic</i></li></ul>",
        );
        model.code_block();
        assert_eq!(
            tx(&model),
            "<ul><li>~Some text <b>and bold&nbsp;</b><br /><pre>~<i>and| italic</i></pre></li></ul>"
        );
    }

    #[test]
    fn add_code_block_to_several_list_items() {
        let mut model =
            cm("<ul><li>~{First item</li><li>~Second}| item</li></ul>");
        model.code_block();
        assert_eq!(tx(&model), "<pre>~{First item\nSecond}| item</pre>");
    }

    #[test]
    fn add_code_block_to_several_lists() {
        let mut model =
            cm("<ul><li>~{First item</li><li>~Second item</li></ul>Some text<ul><li>~Third}| item</li><li>~Fourth one</li></ul>");
        model.code_block();
        assert_eq!(tx(&model), "<pre>~{First item\nSecond item\nSome text\nThird}| item</pre><ul><li>~Fourth one</li></ul>");
    }

    #[test]
    fn add_code_block_to_list_and_external_nodes() {
        let mut model =
            cm("{Text <ul><li>~First item</li><li>~Second}| item</li></ul>");
        model.code_block();
        assert_eq!(tx(&model), "<pre>~{Text \nFirst item\nSecond}| item</pre>");
    }

    #[test]
    fn add_code_block_to_existing_code_block() {
        let mut model = cm("{Text <pre>~code}|</pre>");
        model.code_block();
        assert_eq!(tx(&model), "<pre>~{Text code}|</pre>");
    }

    #[test]
    fn add_code_block_to_existing_code_block_partially_selected() {
        let mut model = cm("{Text <pre>~<b>code}|</b><i> and italic</i></pre>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "<pre>~{Text <b>code}|</b><i> and italic</i></pre>"
        );
    }

    #[test]
    fn add_code_block_to_nested_item_in_formatting_node() {
        let mut model = cm("<b>Text<br /><i>{in italic}|</i></b>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "<b>Text<br /></b><pre>~<b><i>{in italic}|</i></b></pre>"
        );
    }

    #[test]
    fn add_code_block_to_deep_nested_item_in_formatting_nodes() {
        let mut model = cm("<u><b>Text<br /><i>{in italic}|</i></b></u>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "<u><b>Text<br /></b></u><pre>~<u><b><i>{in italic}|</i></b></u></pre>"
        );
    }

    #[test]
    fn add_code_block_to_quote() {
        let mut model = cm("<blockquote>~Quot|e</blockquote>");
        model.code_block();
        assert_eq!(tx(&model), "<pre>~Quot|e</pre>");
    }

    #[test]
    fn add_code_block_to_quote_text_before() {
        let mut model = cm("Te{xt <blockquote>~Quot}|e</blockquote>");
        model.code_block();
        assert_eq!(tx(&model), "<pre>~Te{xt \nQuot}|e</pre>");
    }

    #[test]
    fn add_code_block_to_quote_text_after() {
        let mut model = cm("<blockquote>~Quo{te</blockquote>Te}|xt");
        model.code_block();
        assert_eq!(tx(&model), "<pre>~Quo{te\nTe}|xt</pre>");
    }

    #[test]
    fn remove_code_block_moves_its_children_out() {
        let mut model = cm("Text <pre>~<b>code|</b><i> and italic</i></pre>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "Text <br /><b>code|</b><i> and italic</i><br />"
        );
    }

    #[test]
    fn remove_code_block_moves_its_children_and_restores_line_breaks() {
        let mut model = cm("Text <pre>~with|\nline\nbreaks</pre>");
        model.code_block();
        assert_eq!(tx(&model), "Text <br />with|<br />line<br />breaks<br />");
    }

    #[test]
    fn remove_code_block_moves_its_children_and_keeps_selection_in_place() {
        let mut model = cm("Text <pre>~wi{th\nline\nbrea}|ks</pre>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "Text <br />wi{th<br />line<br />brea}|ks<br />"
        );
    }

    #[test]
    fn test_creating_code_block_at_the_end_of_editor() {
        let mut model = cm("Test<br/>|");
        model.code_block();
        assert_eq!(tx(&model), "Test<br /><pre>~|</pre>");
    }

    #[test]
    fn creating_and_removing_code_block_works() {
        let mut model = cm("|");
        model.code_block();
        assert_eq!(tx(&model), "<pre>~|</pre>");
        model.code_block();
        assert_eq!(tx(&model), "|");
    }

    #[test]
    fn removing_code_block_doesnt_add_extra_line_break_at_start() {
        let mut model = cm("<br />|");
        model.code_block();
        assert_eq!(tx(&model), "<br /><pre>~|</pre>");
        model.code_block();
        assert_eq!(tx(&model), "<br />|");
    }

    #[test]
    fn removing_code_block_doesnt_add_extra_line_break_at_end() {
        let mut model = cm("|<br />");
        model.code_block();
        assert_eq!(tx(&model), "<pre>~|</pre><br />");
        model.code_block();
        assert_eq!(tx(&model), "|<br />");
    }
}
