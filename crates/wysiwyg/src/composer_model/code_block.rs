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

use crate::dom::action_list::DomActionList;
use crate::dom::nodes::dom_node::DomNodeKind;
use crate::dom::nodes::dom_node::DomNodeKind::*;
use crate::dom::nodes::{ContainerNode, ContainerNodeKind, DomNode};
use crate::dom::unicode_string::UnicodeStr;
use crate::dom::{DomHandle, DomLocation};
use crate::{
    ComposerAction, ComposerModel, ComposerUpdate, ToHtml, UnicodeString,
};
use std::cmp::min;
use std::collections::HashSet;
use std::ops::{Add, AddAssign};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn code_block(&mut self) -> ComposerUpdate<S> {
        if self.action_is_reversed(ComposerAction::CodeBlock) {
            // TODO: add code block removal if selection is inside the code block, otherwise extend it
            self.remove_code_block()
        } else {
            self.add_code_block()
        }
    }

    fn add_code_block(&mut self) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        let Some((parent_handle, idx_start, idx_end)) = self.find_nodes_to_wrap_in_block(s, e) else {
            return ComposerUpdate::keep();
        };
        let mut dom = &mut self.state.dom;
        let mut leaves_to_add: Vec<DomNode<S>> = Vec::new();
        let mut nodes_visited = HashSet::new();
        let start_handle = parent_handle.child_handle(idx_start);
        let up_to_handle = parent_handle.child_handle(idx_end + 1);
        let iter = dom.iter_from_handle(&start_handle);
        for node in iter {
            if node.handle() >= up_to_handle {
                break;
            }
            match node.kind() {
                Text | Formatting(_) | Link => {
                    nodes_visited.insert(node.handle());
                    if nodes_visited.contains(&node.handle().parent_handle()) {
                        continue;
                    }
                    leaves_to_add.push(node.clone());
                }
                LineBreak => {
                    leaves_to_add.push(DomNode::new_text("\n".into()));
                }
                List | ListItem => {
                    let mut needs_to_add_line_break =
                        node.handle().index_in_parent() > 0;
                    if needs_to_add_line_break {
                        if let Some(DomNode::Text(text_node)) =
                            leaves_to_add.last()
                        {
                            if text_node.data().to_string() == "\n" {
                                needs_to_add_line_break = false;
                            }
                        }
                    }

                    if needs_to_add_line_break {
                        leaves_to_add.push(DomNode::new_text("\n".into()));
                        self.state.end.add_assign(1);
                    }
                }
                _ => {}
            }
        }

        for i in (idx_start..=idx_end).rev() {
            dom.remove(&parent_handle.child_handle(i));
        }

        dom.insert_at(&start_handle, DomNode::new_code_block(leaves_to_add));

        // Merge any nodes that need it
        self.merge_adjacent_code_blocks(&start_handle);

        self.create_update_replace_all()
    }

    fn merge_adjacent_code_blocks(&mut self, handle: &DomHandle) {
        let mut handle = handle.clone();
        if let Some(next_code_block_handle) = self
            .state
            .dom
            .next_sibling(&handle)
            .filter(|n| n.kind() == CodeBlock)
            .map(|n| n.handle())
        {
            self.move_children_and_delete_parent(
                &next_code_block_handle,
                &handle,
            );
        }

        if let Some(prev_code_block_handle) = self
            .state
            .dom
            .prev_sibling(&handle)
            .filter(|n| n.kind() == CodeBlock)
            .map(|n| n.handle())
        {
            self.move_children_and_delete_parent(
                &handle,
                &prev_code_block_handle,
            );
            handle = prev_code_block_handle;
        }

        self.join_format_nodes_at_level(
            &handle,
            handle.raw().len() - 1,
            &mut DomActionList::default(),
        );
        self.join_text_nodes_in_parent(&handle);
    }

    fn find_nodes_to_wrap_in_block(
        &self,
        start: usize,
        end: usize,
    ) -> Option<(DomHandle, usize, usize)> {
        let dom = &self.state.dom;
        let range = dom.find_range(start, end);
        let leaves: Vec<&DomLocation> = range.leaves().collect();
        if leaves.is_empty() {
            None
        } else {
            let first_leaf = leaves.first().unwrap();
            let last_leaf = leaves.last().unwrap();
            let iter = dom.iter_from_handle(&last_leaf.node_handle);
            let rev_iter = dom.iter_from_handle(&first_leaf.node_handle).rev();
            let mut nodes_to_cover = (
                HandleWithKind {
                    handle: first_leaf.node_handle.clone(),
                    kind: first_leaf.kind.clone(),
                },
                HandleWithKind {
                    handle: last_leaf.node_handle.clone(),
                    kind: last_leaf.kind.clone(),
                },
            );
            // TODO: check if ancestors of both start and end leaves contains a ListItem instead
            let selection_contains_several_list_items = range
                .locations
                .iter()
                .filter(|l| l.kind == ListItem)
                .count()
                > 1;
            for node in rev_iter {
                if !node.is_block_node()
                    && node.kind() != LineBreak
                    && (node.kind() != ListItem
                        || selection_contains_several_list_items)
                {
                    nodes_to_cover.0 = HandleWithKind {
                        handle: node.handle(),
                        kind: node.kind().clone(),
                    };
                } else {
                    break;
                }
            }

            for node in iter {
                if !node.is_block_node()
                    && node.kind() != LineBreak
                    && (node.kind() != ListItem
                        || selection_contains_several_list_items)
                {
                    nodes_to_cover.1 = HandleWithKind {
                        handle: node.handle(),
                        kind: node.kind().clone(),
                    };
                } else {
                    break;
                }
            }

            let (first, last) = nodes_to_cover;
            let max_level =
                min(first.handle.raw().len(), last.handle.raw().len());
            let mut min_level = 0;
            for i in min_level..max_level {
                min_level = i;
                if first.handle.raw()[i] != last.handle.raw()[i] {
                    break;
                }
            }
            if first.kind == ListItem || last.kind == ListItem {
                // We should wrap their parent List instead
                min_level -= 1;
            }
            let idx_start = first.handle.raw()[min_level];
            let idx_end = last.handle.raw()[min_level];
            let ancestor_handle = first.handle.sub_handle_up_to(min_level);
            Some((ancestor_handle, idx_start, idx_end))
        }
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
        self.state.start.add_assign(1);
        self.state.end.add_assign(1);

        // If we remove the whole code block, we should start by adding a line break
        let mut nodes_to_add = vec![DomNode::new_line_break()];
        // Just remove everything
        let DomNode::Container(block) =
            self.state.dom.lookup_node(&code_block_location.node_handle) else
        {
            panic!("CodeBlock must be a container node");
        };
        for child in block.children() {
            match child {
                DomNode::Text(text_node) => {
                    // Split the TextNode by \n and add them both to the nodes to add
                    let mut text_start: usize = 0;
                    let mut text_end = text_start;
                    let data = text_node.data();
                    for char in data.chars() {
                        text_end += data.char_len(&char);
                        if char == '\n' {
                            let text_to_add =
                                data[text_start..text_end - 1].to_owned();
                            nodes_to_add
                                .push(DomNode::new_text(S::from(text_to_add)));
                            nodes_to_add.push(DomNode::new_line_break());
                            if text_end <= start_in_block {
                                selection_offset_start += 1;
                            }
                            if text_end <= end_in_block {
                                selection_offset_end += 1;
                            }
                            text_start = text_end;
                        }
                    }
                    if text_start != text_end {
                        let text_to_add =
                            text_node.data()[text_start..text_end].to_owned();
                        nodes_to_add
                            .push(DomNode::new_text(S::from(text_to_add)));
                        nodes_to_add.push(DomNode::new_line_break());
                        if text_end <= start_in_block {
                            selection_offset_start += 1;
                        }
                        if text_end <= end_in_block {
                            selection_offset_end += 1;
                        }
                    }
                }
                // Just move the node out
                _ => nodes_to_add.push(child.clone()),
            }
        }

        // Add a final line break if needed
        if let Some(last_node) = nodes_to_add.last() {
            if last_node.kind() != LineBreak {
                nodes_to_add.push(DomNode::new_line_break());
            }
        }

        self.state
            .dom
            .replace(&code_block_location.node_handle, nodes_to_add);

        if selection_offset_start > 0 {
            self.state.start.add_assign(selection_offset_start);
            self.state.end.add_assign(selection_offset_end);
        }

        self.create_update_replace_all()
    }

    fn find_parent_to_split(&self, handle: &DomHandle) -> DomHandle {
        let path_len = handle.raw().len();
        if path_len <= 1 {
            DomHandle::root()
        } else {
            for i in (0..handle.raw().len()).rev() {
                let ancestor_handle = handle.sub_handle_up_to(i);
                let ancestor = self.state.dom.lookup_node(&ancestor_handle);
                match ancestor.kind() {
                    Generic | List | ListItem | CodeBlock => {
                        return ancestor_handle
                    }
                    _ => continue,
                }
            }
            panic!("Should never reach this point, one of the parents surely can be split.");
        }
    }

    fn split_sub_tree(
        &mut self,
        from_handle: &DomHandle,
        offset: usize,
        level: usize,
    ) -> DomNode<S> {
        let subtree_len = from_handle.raw().len();
        // Create new 'root' node to contain the split sub-tree
        let mut new_subtree = DomNode::Container(ContainerNode::new(
            S::default(),
            ContainerNodeKind::Generic,
            None,
            Vec::new(),
        ));
        new_subtree.set_handle(DomHandle::root());
        for cur_level in level..subtree_len {
            let index_at_level = from_handle.raw()[cur_level];
            let handle_up_to = from_handle.sub_handle_up_to(cur_level);
            if let DomNode::Container(cur) =
                self.state.dom.lookup_node_mut(&handle_up_to)
            {
                let mut removed_nodes = Vec::new();
                for idx in (index_at_level..cur.children().len()).rev() {
                    if idx == index_at_level {
                        let child = cur.get_child_mut(idx).unwrap();
                        match child {
                            DomNode::Container(container) => {
                                removed_nodes.insert(
                                    0,
                                    DomNode::Container(
                                        container
                                            .copy_with_new_children(Vec::new()),
                                    ),
                                );
                            }
                            DomNode::Text(text_node) => {
                                if offset == 0 {
                                    removed_nodes.insert(0, child.clone());
                                } else {
                                    let left_data =
                                        text_node.data()[..offset].to_owned();
                                    let right_data =
                                        text_node.data()[offset..].to_owned();
                                    text_node.set_data(left_data);
                                    removed_nodes.insert(
                                        0,
                                        DomNode::new_text(right_data),
                                    );
                                }
                            }
                            _ => {
                                removed_nodes.insert(0, child.clone());
                            }
                        }
                    } else {
                        removed_nodes.insert(0, cur.remove_child(idx));
                    }
                }
                let mut new_subtree_at_prev_level = &mut new_subtree;
                for _ in level..cur_level {
                    if let DomNode::Container(c) = new_subtree_at_prev_level {
                        let child = c.get_child_mut(0).unwrap();
                        new_subtree_at_prev_level = child;
                    }
                }
                if let DomNode::Container(new_subtree) =
                    new_subtree_at_prev_level
                {
                    if !removed_nodes.is_empty() {
                        for node in removed_nodes {
                            new_subtree.append_child(node);
                        }
                    }
                }
            }
        }

        let html = new_subtree.to_html().to_string();
        dbg!(html);

        new_subtree
    }
}

struct HandleWithKind {
    handle: DomHandle,
    kind: DomNodeKind,
}

#[cfg(test)]
mod test {
    use crate::tests::testutils_composer_model::{cm, tx};
    use crate::{DomHandle, ToHtml};

    #[test]
    fn find_ranges_to_wrap_simple_text() {
        let model = cm("Some text|");
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap_in_block(s, e);
        assert_eq!(ret, Some((DomHandle::from_raw(Vec::new()), 0, 0)));
    }

    #[test]
    fn find_ranges_to_wrap_several_nodes() {
        let model = cm("Some text| <b>and bold </b><i>and italic</i>");
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap_in_block(s, e);
        assert_eq!(ret, Some((DomHandle::from_raw(Vec::new()), 0, 2)));
    }

    #[test]
    fn find_ranges_to_wrap_several_nodes_with_line_break_at_end() {
        let model = cm("Some text| <b>and bold </b><br/><i>and italic</i>");
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap_in_block(s, e);
        assert_eq!(ret, Some((DomHandle::from_raw(Vec::new()), 0, 1)));
    }

    #[test]
    fn find_ranges_to_wrap_several_nodes_with_line_break_at_start() {
        let model = cm("Some text <br/><b>and bold </b><i>|and italic</i>");
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap_in_block(s, e);
        assert_eq!(ret, Some((DomHandle::from_raw(Vec::new()), 2, 3)));
    }

    #[test]
    fn find_ranges_to_wrap_list_item() {
        let model = cm(
            "<ul><li>Some text <b>and bold </b><i>|and italic</i></li></ul>",
        );
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap_in_block(s, e);
        assert_eq!(ret, Some((DomHandle::from_raw(vec![0, 0]), 0, 2)));
    }

    #[test]
    fn find_ranges_to_wrap_list_item_with_line_breaks() {
        let model = cm(
            "<ul><li>Some text <b>and bold </b><br/><i>and| italic</i></li></ul>",
        );
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap_in_block(s, e);
        assert_eq!(ret, Some((DomHandle::from_raw(vec![0, 0]), 3, 3)));
    }

    #[test]
    fn find_ranges_to_wrap_several_list_items() {
        let model = cm("<ul><li>{First item</li><li>Second}| item</li></ul>");
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap_in_block(s, e);
        assert_eq!(ret, Some((DomHandle::from_raw(vec![]), 0, 0)));
    }

    #[test]
    fn find_ranges_to_wrap_list_and_external_nodes() {
        let model =
            cm("{Text <ul><li>First item</li><li>Second}| item</li></ul>");
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap_in_block(s, e);
        assert_eq!(ret, Some((DomHandle::from_raw(vec![]), 0, 1)));
    }

    // Tests: Code block

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
            "<pre>Some text| <b>and bold&nbsp;</b><i>and italic</i></pre>"
        );
    }

    #[test]
    fn add_code_block_to_several_nodes_with_line_break_at_end() {
        let mut model = cm("Some text| <b>and bold </b><br/><i>and italic</i>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "<pre>Some text| <b>and bold&nbsp;</b></pre><br /><i>and italic</i>"
        );
    }

    #[test]
    fn add_code_block_to_several_nodes_with_line_break_at_start() {
        let mut model = cm("Some text <br/><b>and bold </b><i>and |italic</i>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "Some text <br /><pre><b>and bold&nbsp;</b><i>and |italic</i></pre>"
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
            "<ul><li><pre>Some text <b>and bold&nbsp;|</b><i>and italic</i></pre></li></ul>"
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
            "<ul><li>Some text <b>and bold&nbsp;</b><br /><pre><i>and| italic</i></pre></li></ul>"
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
    fn add_code_block_to_list_and_external_nodes() {
        let mut model =
            cm("{Text <ul><li>First item</li><li>Second}| item</li></ul>");
        model.code_block();
        assert_eq!(tx(&model), "<pre>{Text \nFirst item\nSecond}| item</pre>");
    }

    #[test]
    fn add_code_block_to_existing_code_block() {
        let mut model = cm("{Text <pre>code}|</pre>");
        model.code_block();
        assert_eq!(tx(&model), "<pre>{Text code}|</pre>");
    }

    #[test]
    fn add_code_block_to_existing_code_block_partially_selected() {
        let mut model = cm("{Text <pre><b>code}|</b><i> and italic</i></pre>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "<pre>{Text <b>code}|</b><i> and italic</i></pre>"
        );
    }

    #[test]
    fn add_code_block_to_nested_item_in_formatting_node() {
        let mut model = cm("<b>Text<br /><i>{in italic}|</i></b>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "<b>Text<br /></b><pre><b><i>{in italic}|</i></b></pre>"
        );
    }

    #[test]
    fn add_code_block_to_deep_nested_item_in_formatting_nodes() {
        let mut model = cm("<u><b>Text<br /><i>{in italic}|</i></b></u>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "<u><b>Text<br /></b></u><pre><u><b><i>{in italic}|</i></b></u></pre>"
        );
    }

    #[test]
    fn remove_code_block_moves_its_children_out() {
        let mut model = cm("Text <pre><b>code|</b><i> and italic</i></pre>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "Text <br /><b>code|</b><i> and italic</i><br />"
        );
    }

    #[test]
    fn remove_code_block_moves_its_children_and_restores_line_breaks() {
        let mut model = cm("Text <pre>with|\nline\nbreaks</pre>");
        model.code_block();
        assert_eq!(tx(&model), "Text <br />with|<br />line<br />breaks<br />");
    }

    #[test]
    fn remove_code_block_moves_its_children_and_keeps_selection_in_place() {
        let mut model = cm("Text <pre>wi{th\nline\nbrea}|ks</pre>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "Text <br />wi{th<br />line<br />brea}|ks<br />"
        );
    }

    #[test]
    fn split_dom_simple() {
        let mut model = cm("Text|<b>bold</b><i>italic</i>");
        let ret = model.split_sub_tree(&DomHandle::from_raw(vec![1, 0]), 2, 0);
        assert_eq!(ret.to_html().to_string(), "<b>ld</b><i>italic</i>");
    }

    #[test]
    fn split_dom_with_nested_formatting() {
        let mut model = cm("<u>Text|<b>bold</b><i>italic</i></u>");
        let ret =
            model.split_sub_tree(&DomHandle::from_raw(vec![0, 1, 0]), 2, 0);
        assert_eq!(ret.to_html().to_string(), "<u><b>ld</b><i>italic</i></u>");
    }

    #[test]
    fn split_dom_with_nested_formatting_at_sub_level() {
        let mut model = cm("<u>Text|<b>bold</b><i>italic</i></u>");
        let ret =
            model.split_sub_tree(&DomHandle::from_raw(vec![0, 1, 0]), 2, 1);
        assert_eq!(ret.to_html().to_string(), "<b>ld</b><i>italic</i>")
    }

    #[test]
    fn split_dom_with_lists() {
        let mut model =
            cm("<ul><li>Text|</li><li><b>bold</b><i>italic</i></li></ul>");
        let depth = 0;
        let offset = 2;
        let ret = model.split_sub_tree(
            &DomHandle::from_raw(vec![0, 1, 0, 0]),
            offset,
            depth,
        );
        assert_eq!(
            ret.to_html().to_string(),
            "<ul><li><b>ld</b><i>italic</i></li></ul>"
        )
    }

    #[test]
    fn split_dom_with_lists_at_sub_level() {
        let mut model =
            cm("<ul><li>Text|</li><li><b>bold</b><i>italic</i></li></ul>");
        let depth = 1;
        let offset = 2;
        let ret = model.split_sub_tree(
            &DomHandle::from_raw(vec![0, 1, 0, 0]),
            offset,
            depth,
        );
        assert_eq!(ret.to_html().to_string(), "<li><b>ld</b><i>italic</i></li>")
    }
}
