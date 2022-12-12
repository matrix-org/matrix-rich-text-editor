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

use std::cmp::min;
use std::collections::HashSet;

use crate::dom::nodes::dom_node::DomNodeKind;
use crate::dom::nodes::dom_node::DomNodeKind::*;
use crate::dom::nodes::{ContainerNode, ContainerNodeKind, DomNode};
use crate::dom::unicode_string::UnicodeStr;
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
        let Some(wrap_result) = self.find_nodes_to_wrap_in_block(s, e) else {
            self.state.dom.append_at_end_of_document(DomNode::new_code_block(vec![DomNode::new_empty_text()]));
            return self.create_update_replace_all();
        };
        let parent_handle = wrap_result.ancestor_handle;
        let idx_start = wrap_result.idx_start;
        let idx_end = wrap_result.idx_end;
        let range = wrap_result.range;
        let leaves: Vec<&DomLocation> = range.leaves().collect();
        let first_leaf = leaves.first().unwrap();
        let last_leaf = leaves.last().unwrap();

        let mut code_block: DomNode<S> = DomNode::new_code_block(Vec::new());
        code_block.set_handle(DomHandle::root());
        let mut nodes_visited = HashSet::new();
        let start_handle = parent_handle.child_handle(idx_start);
        let up_to_handle = parent_handle.child_handle(idx_end + 1);
        let ancestor_to_split = self.find_ancestor_to_split(&start_handle);
        let start_depth = ancestor_to_split.depth();
        let mut path: Vec<usize> = Vec::new();
        // We'll build a new sub-tree with the contents of the code block, then insert it
        for depth in start_depth..start_handle.depth() {
            let mut cur_child = &mut code_block;
            for i in path.iter() {
                if let DomNode::Container(c) = cur_child {
                    cur_child = c.get_child_mut(*i).unwrap();
                }
            }

            let DomNode::Container(cur_container) = cur_child else { break; };
            let path_len = parent_handle.depth();
            if depth == path_len {
                let iter = self.state.dom.iter_from_handle(&start_handle);
                for node in iter {
                    let parent = self.state.dom.parent(&node.handle());

                    if node.handle() == up_to_handle {
                        break;
                    }

                    let is_in_selection = range.contains(&node.handle());
                    let is_before = node.handle() <= first_leaf.node_handle
                        && !is_in_selection;
                    let is_after = node.handle() > last_leaf.node_handle
                        && !is_in_selection;

                    match node.kind() {
                        Text | Formatting(_) | Link | LineBreak => {
                            nodes_visited.insert(node.handle());
                            if nodes_visited
                                .contains(&node.handle().parent_handle())
                            {
                                continue;
                            }
                            cur_container.append_child(
                                Self::format_node_for_code_block(node),
                            );

                            let needs_to_add_line_break = node.kind()
                                != LineBreak
                                && *parent.kind()
                                    == ContainerNodeKind::ListItem
                                && self
                                    .state
                                    .dom
                                    .is_last_in_parent(&node.handle());

                            if needs_to_add_line_break {
                                cur_container.append_child(DomNode::new_text(
                                    "\n".into(),
                                ));
                                if is_before {
                                    self.state.start += 1;
                                    self.state.end += 1;
                                }
                                if !is_after {
                                    if let Some(location) =
                                        range.locations.iter().find(|l| {
                                            l.node_handle == node.handle()
                                        })
                                    {
                                        if location.is_covered() {
                                            self.state.end += 1;
                                        }
                                    } else {
                                        self.state.end += 1;
                                    }
                                }
                            }
                        }
                        List => {
                            // Add line break before the List if it's not at the start
                            if node.handle().index_in_parent() > 0 {
                                cur_container.append_child(DomNode::new_text(
                                    "\n".into(),
                                ));
                                if is_before {
                                    self.state.start += 1;
                                }
                                if !is_after {
                                    self.state.end += 1;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                // We already copied everything we needed to the sub-tree, delete the original nodes
                for i in (idx_start..=idx_end).rev() {
                    self.state.dom.remove(&parent_handle.child_handle(i));
                }
            } else {
                // For every depth level, just copy any ancestor node with no children
                // to re-create the sub-tree
                let cur_sub_handle = start_handle.sub_handle_up_to(depth + 1);
                let index_at_depth = cur_sub_handle.index_in_parent();
                path.push(index_at_depth);
                if let DomNode::Container(container) =
                    self.state.dom.lookup_node(&cur_sub_handle)
                {
                    cur_container.append_child(DomNode::Container(
                        container.copy_with_new_children(Vec::new()),
                    ));
                }
            }
        }

        let insert_at_handle =
            start_handle.sub_handle_up_to(ancestor_to_split.depth() + 1);
        let insert_at_handle =
            if idx_start > 0 && self.state.dom.contains(&insert_at_handle) {
                insert_at_handle.next_sibling()
            } else {
                insert_at_handle
            };
        self.state.dom.insert_at(&insert_at_handle, code_block);

        // Merge any nodes that need it
        self.merge_adjacent_code_blocks(&insert_at_handle);

        self.create_update_replace_all()
    }

    fn merge_adjacent_code_blocks(&mut self, handle: &DomHandle) {
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
    }

    fn find_nodes_to_wrap_in_block(
        &self,
        start: usize,
        end: usize,
    ) -> Option<WrapSearchResult> {
        let dom = &self.state.dom;
        let range = dom.find_range(start, end);
        let leaves: Vec<&DomLocation> = range.leaves().collect();
        if leaves.is_empty() {
            None
        } else {
            let first_leaf = leaves.first().unwrap();
            let last_leaf = leaves.last().unwrap();
            let rev_iter = dom.iter_from_handle(&first_leaf.node_handle).rev();
            let iter = dom.iter_from_handle(&last_leaf.node_handle);
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
            // If we have a selection inside a single ListItem we only want to wrap its contents.
            // However, if the selection covers several list items, we should split its parent List
            // and wrap their contents instead.
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
            let max_depth = min(first.handle.depth(), last.handle.depth());
            let mut min_depth = 0;
            for i in min_depth..max_depth {
                min_depth = i;
                if first.handle.raw()[i] != last.handle.raw()[i] {
                    break;
                }
            }
            if (first.kind == ListItem || last.kind == ListItem)
                && min_depth > 0
            {
                // We should wrap their parent List instead
                min_depth -= 1;
            }
            let idx_start = first.handle.raw()[min_depth];
            let idx_end = last.handle.raw()[min_depth];
            let ancestor_handle = first.handle.sub_handle_up_to(min_depth);
            Some(WrapSearchResult {
                ancestor_handle,
                idx_start,
                idx_end,
                range,
            })
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
        self.state.start += 1;
        self.state.end += 1;

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
                    // Split the TextNode by \n and add them and a line break to the nodes to add
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
            self.state.start += selection_offset_start;
            self.state.end += selection_offset_end;
        }

        self.create_update_replace_all()
    }

    fn find_ancestor_to_split(&self, handle: &DomHandle) -> DomHandle {
        let path_len = handle.depth();
        if path_len <= 1 {
            DomHandle::root()
        } else {
            for i in (0..handle.depth()).rev() {
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

    pub(crate) fn split_sub_tree(
        &mut self,
        from_handle: &DomHandle,
        offset: usize,
        depth: usize,
    ) -> DomNode<S> {
        // Create new 'root' node to contain the split sub-tree
        let mut new_subtree = DomNode::Container(ContainerNode::new(
            S::default(),
            ContainerNodeKind::Generic,
            None,
            Vec::new(),
        ));
        new_subtree.set_handle(DomHandle::root());

        let path = from_handle.sub_handle_up_to(depth).raw().clone();
        self.split_sub_tree_at(
            path,
            from_handle.raw()[depth],
            depth,
            offset,
            from_handle,
            &mut new_subtree,
        );
        new_subtree
    }

    fn split_sub_tree_at<'a>(
        &'a mut self,
        path: Vec<usize>,
        index_in_parent: usize,
        min_depth: usize,
        offset: usize,
        handle: &DomHandle,
        result: &'a mut DomNode<S>,
    ) {
        let mut path = path;
        let cur_handle = DomHandle::from_raw(path.clone());
        path.push(index_in_parent);
        let mut has_next_level = false;
        let cur_subtree = self.state.dom.lookup_node_mut(&cur_handle);
        let mut removed_nodes = Vec::new();
        if let DomNode::Container(container) = cur_subtree {
            for idx in (index_in_parent..container.children().len()).rev() {
                if idx == index_in_parent {
                    let child = container.get_child_mut(idx).unwrap();
                    match child {
                        DomNode::Container(c) => {
                            has_next_level = true;
                            removed_nodes.insert(
                                0,
                                DomNode::Container(
                                    c.copy_with_new_children(Vec::new()),
                                ),
                            );
                        }
                        DomNode::Text(text_node) => {
                            if offset == 0 {
                                removed_nodes.insert(0, child.clone());
                            } else if offset >= text_node.data().chars().count()
                            {
                                // Do nothing
                            } else {
                                let left_data =
                                    text_node.data()[..offset].to_owned();
                                let right_data =
                                    text_node.data()[offset..].to_owned();
                                text_node.set_data(left_data);
                                removed_nodes
                                    .insert(0, DomNode::new_text(right_data));
                            }
                        }
                        _ => {
                            removed_nodes.insert(0, child.clone());
                        }
                    }
                } else {
                    removed_nodes.insert(0, container.remove_child(idx));
                }
            }
        }
        let mut new_subtree_at_prev_level = result;
        if (path.len() - min_depth) > 1 {
            if let DomNode::Container(c) = new_subtree_at_prev_level {
                new_subtree_at_prev_level = c.get_child_mut(0).unwrap();
            }
        }
        if let DomNode::Container(new_subtree) = new_subtree_at_prev_level {
            if !removed_nodes.is_empty() {
                for node in removed_nodes {
                    new_subtree.append_child(node);
                }
            }
        }

        if has_next_level {
            let index_at_level = if path.len() < handle.depth() {
                handle.raw()[path.len()]
            } else {
                0
            };
            self.split_sub_tree_at(
                path,
                index_at_level,
                min_depth,
                offset,
                &handle,
                new_subtree_at_prev_level,
            );
        }
    }

    pub(crate) fn format_node_for_code_block(node: &DomNode<S>) -> DomNode<S> {
        match node {
            DomNode::LineBreak(_) => {
                let mut text_node = DomNode::new_text("\n".into());
                text_node.set_handle(node.handle().clone());
                text_node
            }
            DomNode::Text(_) => node.clone(),
            DomNode::Container(container) => {
                let mut children = Vec::new();
                for c in container.children() {
                    children.push(Self::format_node_for_code_block(c));
                }
                DomNode::Container(container.copy_with_new_children(children))
            }
        }
    }
}

struct HandleWithKind {
    handle: DomHandle,
    kind: DomNodeKind,
}

struct WrapSearchResult {
    ancestor_handle: DomHandle,
    idx_start: usize,
    idx_end: usize,
    range: Range,
}

#[cfg(test)]
mod test {
    use crate::tests::testutils_composer_model::{cm, tx};
    use crate::{DomHandle, ToHtml};

    #[test]
    fn find_ranges_to_wrap_simple_text() {
        let model = cm("Some text|");
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap_in_block(s, e).unwrap();
        assert_eq!(ret.ancestor_handle, DomHandle::from_raw(Vec::new()));
        assert_eq!(ret.idx_start, 0);
        assert_eq!(ret.idx_end, 0);
    }

    #[test]
    fn find_ranges_to_wrap_several_nodes() {
        let model = cm("Some text| <b>and bold </b><i>and italic</i>");
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap_in_block(s, e).unwrap();
        assert_eq!(ret.ancestor_handle, DomHandle::from_raw(Vec::new()));
        assert_eq!(ret.idx_start, 0);
        assert_eq!(ret.idx_end, 2);
    }

    #[test]
    fn find_ranges_to_wrap_several_nodes_with_line_break_at_end() {
        let model = cm("Some text| <b>and bold </b><br/><i>and italic</i>");
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap_in_block(s, e).unwrap();
        assert_eq!(ret.ancestor_handle, DomHandle::from_raw(Vec::new()));
        assert_eq!(ret.idx_start, 0);
        assert_eq!(ret.idx_end, 1);
    }

    #[test]
    fn find_ranges_to_wrap_several_nodes_with_line_break_at_start() {
        let model = cm("Some text <br/><b>and bold </b><i>|and italic</i>");
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap_in_block(s, e).unwrap();
        assert_eq!(ret.ancestor_handle, DomHandle::from_raw(Vec::new()));
        assert_eq!(ret.idx_start, 2);
        assert_eq!(ret.idx_end, 3);
    }

    #[test]
    fn find_ranges_to_wrap_list_item() {
        let model = cm(
            "<ul><li>Some text <b>and bold </b><i>|and italic</i></li></ul>",
        );
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap_in_block(s, e).unwrap();
        assert_eq!(ret.ancestor_handle, DomHandle::from_raw(vec![0, 0]));
        assert_eq!(ret.idx_start, 0);
        assert_eq!(ret.idx_end, 2);
    }

    #[test]
    fn find_ranges_to_wrap_list_item_with_line_breaks() {
        let model = cm(
            "<ul><li>Some text <b>and bold </b><br/><i>and| italic</i></li></ul>",
        );
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap_in_block(s, e).unwrap();
        assert_eq!(ret.ancestor_handle, DomHandle::from_raw(vec![0, 0]));
        assert_eq!(ret.idx_start, 3);
        assert_eq!(ret.idx_end, 3);
    }

    #[test]
    fn find_ranges_to_wrap_several_list_items() {
        let model = cm("<ul><li>{First item</li><li>Second}| item</li></ul>");
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap_in_block(s, e).unwrap();
        assert_eq!(ret.ancestor_handle, DomHandle::from_raw(Vec::new()));
        assert_eq!(ret.idx_start, 0);
        assert_eq!(ret.idx_end, 0);
    }

    #[test]
    fn find_ranges_to_wrap_list_and_external_nodes() {
        let model =
            cm("{Text <ul><li>First item</li><li>Second}| item</li></ul>");
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap_in_block(s, e).unwrap();
        assert_eq!(ret.ancestor_handle, DomHandle::from_raw(Vec::new()));
        assert_eq!(ret.idx_start, 0);
        assert_eq!(ret.idx_end, 1);
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
            "<ul><li><pre>Some text <b>and bold&nbsp;|</b><i>and italic</i>\n</pre></li></ul>"
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
            "<ul><li>Some text <b>and bold&nbsp;</b><br /><pre><i>and| italic</i>\n</pre></li></ul>"
        );
    }

    #[test]
    fn add_code_block_to_several_list_items() {
        let mut model =
            cm("<ul><li>{First item</li><li>Second}| item</li></ul>");
        model.code_block();
        assert_eq!(tx(&model), "<pre>{First item\nSecond}| item\n</pre>");
    }

    #[test]
    fn add_code_block_to_several_lists() {
        let mut model =
            cm("<ul><li>{First item</li><li>Second item</li></ul>Some text<ul><li>Third}| item</li><li>Fourth one</li></ul>");
        model.code_block();
        assert_eq!(tx(&model), "<pre>{First item\nSecond item\nSome text\nThird}| item\nFourth one\n</pre>");
    }

    #[test]
    fn add_code_block_to_list_and_external_nodes() {
        let mut model =
            cm("{Text <ul><li>First item</li><li>Second}| item</li></ul>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "<pre>{Text \nFirst item\nSecond}| item\n</pre>"
        );
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

    #[test]
    fn split_dom_with_partial_handle() {
        let mut model = cm("<u>Text|<b>bold</b><i>italic</i></u>");
        let ret = model.split_sub_tree(&DomHandle::from_raw(vec![0, 1]), 2, 0);
        assert_eq!(ret.to_html().to_string(), "<u><b>ld</b><i>italic</i></u>");
    }
}
