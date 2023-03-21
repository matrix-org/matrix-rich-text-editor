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

use std::ops::ControlFlow;

use crate::char::CharExt;
use crate::composer_model::example_format::SelectionWriter;
use crate::dom::dom_handle::DomHandle;
use crate::dom::nodes::dom_node::{DomNode, DomNodeKind};
use crate::dom::to_html::{ToHtml, ToHtmlState};
use crate::dom::to_markdown::{MarkdownError, MarkdownOptions, ToMarkdown};
use crate::dom::to_plain_text::ToPlainText;
use crate::dom::to_raw_text::ToRawText;
use crate::dom::to_tree::ToTree;
use crate::dom::unicode_string::{UnicodeStr, UnicodeStrExt, UnicodeStringExt};
use crate::dom::{self, UnicodeString};
use crate::{InlineFormatType, ListType};

#[derive(Clone, Debug, PartialEq)]
pub struct ContainerNode<S>
where
    S: UnicodeString,
{
    name: S,
    kind: ContainerNodeKind<S>,
    attrs: Option<Vec<(S, S)>>,
    children: Vec<DomNode<S>>,
    handle: DomHandle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContainerNodeKind<S>
where
    S: UnicodeString,
{
    Generic, // E.g. the root node (the containing div)
    Formatting(InlineFormatType),
    Link(S),
    List(ListType),
    ListItem,
    CodeBlock,
    Quote,
    Paragraph,
}

impl<S: dom::unicode_string::UnicodeString> Default for ContainerNode<S> {
    fn default() -> ContainerNode<S> {
        // implementation here is as per pub fn new in dom_struct.rs
        let mut document = Self::new(
            S::default(),
            ContainerNodeKind::Generic,
            None,
            Vec::new(),
        );

        document.set_handle(DomHandle::from_raw(Vec::new()));
        document
    }
}

impl<S> ContainerNode<S>
where
    S: UnicodeString,
{
    /// Create a new ContainerNode
    ///
    /// NOTE: Its handle() will be unset until you call set_handle() or
    /// append() it to another node.
    pub fn new(
        name: S,
        kind: ContainerNodeKind<S>,
        attrs: Option<Vec<(S, S)>>,
        children: Vec<DomNode<S>>,
    ) -> Self {
        Self {
            name,
            kind,
            attrs,
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn new_paragraph(children: Vec<DomNode<S>>) -> Self {
        Self {
            name: "p".into(),
            kind: ContainerNodeKind::Paragraph,
            attrs: None,
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn new_formatting_from_tag(
        format: S,
        children: Vec<DomNode<S>>,
    ) -> Option<Self> {
        InlineFormatType::try_from(format.clone())
            .map(|f| Self {
                name: format,
                kind: ContainerNodeKind::Formatting(f),
                attrs: None,
                children,
                handle: DomHandle::new_unset(),
            })
            .ok()
    }

    pub fn new_formatting(
        format: InlineFormatType,
        children: Vec<DomNode<S>>,
    ) -> Self {
        Self {
            name: format.tag().into(),
            kind: ContainerNodeKind::Formatting(format),
            attrs: None,
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn new_list(list_type: ListType, children: Vec<DomNode<S>>) -> Self {
        Self {
            name: list_type.tag().into(),
            kind: ContainerNodeKind::List(list_type),
            attrs: None,
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn new_list_item(children: Vec<DomNode<S>>) -> Self {
        Self {
            name: "li".into(),
            kind: ContainerNodeKind::ListItem,
            attrs: None,
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn new_code_block(children: Vec<DomNode<S>>) -> Self {
        Self {
            name: "codeblock".into(),
            kind: ContainerNodeKind::CodeBlock,
            attrs: None,
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn new_quote(children: Vec<DomNode<S>>) -> Self {
        Self {
            name: "blockquote".into(),
            kind: ContainerNodeKind::Quote,
            attrs: None,
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn append_child(&mut self, mut child: DomNode<S>) -> DomHandle {
        assert!(self.handle.is_set());

        let child_index = self.children.len();
        let child_handle = self.handle.child_handle(child_index);
        child.set_handle(child_handle.clone());
        self.children.push(child);
        child_handle
    }

    pub fn append_children(&mut self, children: Vec<DomNode<S>>) {
        for child in children {
            self.append_child(child);
        }
    }

    pub fn remove_child(&mut self, index: usize) -> DomNode<S> {
        assert!(self.handle.is_set());
        assert!(index < self.children().len());

        let ret = self.children.remove(index);

        for child_index in index..self.children.len() {
            let new_handle = self.handle.child_handle(child_index);
            self.children[child_index].set_handle(new_handle);
        }

        ret
    }

    /// Insert an array of children [nodes] at given [index].
    /// Returns new handles for moved nodes.
    pub fn insert_children(
        &mut self,
        index: usize,
        nodes: Vec<DomNode<S>>,
    ) -> Vec<DomHandle> {
        let mut handles = Vec::new();

        let mut current_index = index;
        for mut node in nodes {
            let child_handle = self.handle.child_handle(current_index);
            node.set_handle(child_handle);
            self.children.insert(current_index, node);
            current_index += 1;
        }

        for child_index in current_index..self.children.len() {
            let new_handle = self.handle.child_handle(child_index);
            self.children[child_index].set_handle(new_handle.clone());
            handles.push(new_handle);
        }
        handles
    }

    /// Replace child at given [index] with an array of children [nodes].
    /// Returns new handles for moved nodes.
    pub fn replace_child(
        &mut self,
        index: usize,
        nodes: Vec<DomNode<S>>,
    ) -> Vec<DomHandle> {
        assert!(self.handle.is_set());
        assert!(index < self.children().len());

        self.children.remove(index);
        self.insert_children(index, nodes)
    }

    pub fn get_child_mut(&mut self, idx: usize) -> Option<&mut DomNode<S>> {
        self.children.get_mut(idx)
    }

    pub fn get_child(&self, idx: usize) -> Option<&DomNode<S>> {
        self.children.get(idx)
    }

    pub fn first_child_mut(&mut self) -> Option<&mut DomNode<S>> {
        self.get_child_mut(0)
    }

    pub fn last_child_mut(&mut self) -> Option<&mut DomNode<S>> {
        self.children.last_mut()
    }

    pub fn insert_child(
        &mut self,
        index: usize,
        node: DomNode<S>,
    ) -> &DomNode<S> {
        assert!(self.handle.is_set());
        assert!(index <= self.children().len());

        self.children.insert(index, node);

        for i in index..self.children.len() {
            let new_handle = self.handle.child_handle(i);
            self.children[i].set_handle(new_handle);
        }

        self.children.get(index).unwrap()
    }

    pub fn handle(&self) -> DomHandle {
        self.handle.clone()
    }

    pub fn set_handle(&mut self, handle: DomHandle) {
        self.handle = handle;
        for (i, child) in self.children.iter_mut().enumerate() {
            child.set_handle(self.handle.child_handle(i))
        }
    }

    pub fn name(&self) -> &S::Str {
        &self.name
    }

    pub fn attributes(&self) -> Option<&Vec<(S, S)>> {
        self.attrs.as_ref()
    }

    pub fn children(&self) -> &Vec<DomNode<S>> {
        &self.children
    }

    pub(crate) fn take_children(self) -> Vec<DomNode<S>> {
        self.children
    }

    /// Removes all children from the container and returns them
    pub(crate) fn remove_children(&mut self) -> Vec<DomNode<S>> {
        self.children.drain(..).collect()
    }

    pub(crate) fn take_children_after(
        &mut self,
        position: usize,
    ) -> Vec<DomNode<S>> {
        return self.children.drain(position..self.children.len()).collect();
    }

    pub fn kind(&self) -> &ContainerNodeKind<S> {
        &self.kind
    }

    pub fn is_link(&self) -> bool {
        matches!(self.kind, ContainerNodeKind::Link(_))
    }

    pub fn is_immutable(&self) -> bool {
        self.attributes()
            .unwrap_or(&vec![])
            .contains(&("contenteditable".into(), "false".into()))
    }

    pub fn is_immutable_link(&self) -> bool {
        matches!(self.kind, ContainerNodeKind::Link(_) if self.is_immutable())
    }

    pub fn is_list_item(&self) -> bool {
        matches!(self.kind, ContainerNodeKind::ListItem)
    }

    pub fn is_list(&self) -> bool {
        matches!(self.kind, ContainerNodeKind::List(_))
    }

    pub(crate) fn is_list_of_type(&self, list_type: &ListType) -> bool {
        matches!(&self.kind, ContainerNodeKind::List(f) if f == list_type)
    }

    pub(crate) fn is_structure_node(&self) -> bool {
        use ContainerNodeKind::*;

        matches!(self.kind, List(_) | ListItem)
    }

    pub(crate) fn is_formatting_node(&self) -> bool {
        matches!(self.kind, ContainerNodeKind::Formatting(_))
    }

    pub(crate) fn is_formatting_node_of_type(
        &self,
        format_type: &InlineFormatType,
    ) -> bool {
        matches!(&self.kind, ContainerNodeKind::Formatting(f) if f == format_type)
    }

    pub(crate) fn is_block_node(&self) -> bool {
        DomNodeKind::from_container_kind(&self.kind).is_block_kind()
    }

    pub fn text_len(&self) -> usize {
        let children_len: usize =
            self.children.iter().map(|child| child.text_len()).sum();
        let block_nodes_extra: usize = self
            .children
            .iter()
            .filter(|child| child.is_block_node())
            .count();
        let block_nodes_extra = if block_nodes_extra > 0 {
            block_nodes_extra - 1
        } else {
            block_nodes_extra
        };
        children_len + block_nodes_extra
    }

    pub fn new_link(
        url: S,
        children: Vec<DomNode<S>>,
        attributes: Vec<(S, S)>,
    ) -> Self {
        // Hosting application may provide attributes but always provides url, this
        // allows the Rust code to stay as generic as possible, since it should only care about
        // `contenteditable="false"` to implement custom behaviours for immutable links.
        let mut attrs = attributes.clone();
        attrs.push(("href".into(), url.clone())); // don't clone here

        Self {
            name: "a".into(),
            kind: ContainerNodeKind::Link(url),
            attrs: Some(attrs),
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub(crate) fn get_list_type(&self) -> Option<&ListType> {
        match &self.kind {
            ContainerNodeKind::List(t) => Some(t),
            _ => None,
        }
    }

    pub(crate) fn set_list_type(&mut self, list_type: ListType) {
        match self.kind {
            ContainerNodeKind::List(_) => {
                self.name = list_type.tag().into();
                self.kind = ContainerNodeKind::List(list_type);
            }
            _ => panic!(
                "Setting list type to a non-list container is not allowed"
            ),
        }
    }

    pub(crate) fn get_link(&self) -> Option<S> {
        let ContainerNodeKind::Link(link) = self.kind.clone() else {
            return None
        };
        Some(link)
    }

    /// Creates a container with the same kind & attributes
    /// as self, with given children and an unset handle.
    pub(crate) fn clone_with_new_children(
        &self,
        children: Vec<DomNode<S>>,
    ) -> Self {
        Self {
            name: self.name.clone(),
            kind: self.kind.clone(),
            attrs: self.attrs.clone(),
            children,
            handle: DomHandle::new_unset(),
        }
    }

    /// Remove leading Line break char from this container.
    /// Returns false if no updates were done.
    pub fn remove_leading_line_break(&mut self) -> bool {
        let Some(first_child) = self.children.get_mut(0) else {
            return false;
        };
        match first_child {
            DomNode::Container(c) => c.remove_leading_line_break(),
            DomNode::LineBreak(_) => {
                if self.handle().is_set() {
                    self.remove_child(0);
                } else {
                    self.children.remove(0);
                }
                true
            }
            _ => false,
        }
    }

    /// Returns whether this container first text-like
    /// child is a line break.
    pub fn has_leading_line_break(&self) -> bool {
        let Some(first_child) = self.children.get(0) else {
            return false;
        };
        first_child.has_leading_line_break()
    }

    /// Push content of the given container node into self. Panics
    /// if given container node is not of the same kind.
    pub(crate) fn push(&mut self, other_node: &mut ContainerNode<S>) {
        if other_node.kind != self.kind {
            panic!("Trying to push a non-matching container kind");
        }
        let last_child = self.children.last().unwrap();
        let other_node_first_child = other_node.get_child(0).unwrap();
        if last_child.can_push(other_node_first_child) {
            let mut next_child = other_node.remove_child(0);
            self.last_child_mut().unwrap().push(&mut next_child);
        }
        while !other_node.children().is_empty() {
            let child = other_node.remove_child(0);
            self.append_child(child);
        }
    }

    /// Slice this container after given position.
    /// Returns a new container of the same kind with the
    /// removed content, with both nodes keeping
    /// expected hierarchy.
    pub fn slice_after(&mut self, position: usize) -> ContainerNode<S> {
        assert!(position <= self.text_len());
        let result = self.find_slice_location(position);

        match result {
            ControlFlow::Continue(_) => self.clone_with_new_children(vec![]),
            ControlFlow::Break((current_loc, child_index, should_slice)) => {
                let mut removed_children = Vec::new();
                let index_to_remove: usize;
                if should_slice {
                    index_to_remove = child_index + 1;
                    let sliced = self
                        .get_child_mut(child_index)
                        .unwrap()
                        .slice_after(position - current_loc);
                    removed_children.push(sliced);
                } else {
                    index_to_remove = child_index;
                }
                while self.children.len() > index_to_remove {
                    removed_children
                        .push(self.children.remove(index_to_remove));
                }
                self.clone_with_new_children(removed_children)
            }
        }
    }

    /// Slice this container before given position.
    /// Returns a new container of the same kind with the
    /// removed content, with both nodes keeping
    /// expected hierarchy.
    pub fn slice_before(&mut self, position: usize) -> ContainerNode<S> {
        assert!(position <= self.text_len());
        let result = self.find_slice_location(position);

        match result {
            ControlFlow::Continue(_) => self.clone_with_new_children(vec![]),
            ControlFlow::Break((current_loc, child_index, should_slice)) => {
                let mut removed_children = Vec::new();
                if should_slice {
                    let sliced = self
                        .get_child_mut(child_index)
                        .unwrap()
                        .slice_before(position - current_loc);
                    removed_children.push(sliced);
                }
                for i in (0..child_index).rev() {
                    removed_children.insert(0, self.children.remove(i));
                }
                self.clone_with_new_children(removed_children)
            }
        }
    }

    /// Returns true if the ContainerNode has no children.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Returns true if there is no text in this ContainerNode.
    pub fn has_no_text(&self) -> bool {
        self.children.iter().all(|c| c.has_no_text())
    }

    fn find_slice_location(
        &self,
        position: usize,
    ) -> ControlFlow<(usize, usize, bool), usize> {
        self.children.iter().enumerate().try_fold(
            0,
            |current_loc, (index, child)| {
                let child_length = child.text_len();
                if current_loc + child_length <= position {
                    ControlFlow::Continue(current_loc + child_length)
                } else if current_loc < position {
                    ControlFlow::Break((current_loc, index, true))
                } else {
                    ControlFlow::Break((current_loc, index, false))
                }
            },
        )
    }

    /// Returns the positions of linebreaks inside container.
    pub fn line_break_positions(&self) -> Vec<usize> {
        let mut current_offset = 0;
        let mut positions: Vec<usize> = Vec::new();
        for child in self.children() {
            match child {
                DomNode::Container(c) => {
                    let mut child_positions: Vec<usize> = c
                        .line_break_positions()
                        .iter()
                        .map(|p| p + current_offset)
                        .collect();
                    positions.append(&mut child_positions);
                }
                DomNode::LineBreak(_) => {
                    positions.push(current_offset);
                }
                _ => {}
            }
            current_offset += child.text_len();
        }
        positions
    }
}

impl<S> ToHtml<S> for ContainerNode<S>
where
    S: UnicodeString,
{
    fn fmt_html(
        &self,
        formatter: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        state: ToHtmlState,
    ) {
        match self.kind() {
            ContainerNodeKind::Paragraph => {
                self.fmt_paragraph_html(formatter, selection_writer, state)
            }
            ContainerNodeKind::CodeBlock => {
                self.fmt_code_block_html(formatter, selection_writer, state)
            }
            _ => self.fmt_default_html(formatter, selection_writer, state),
        };
    }
}

impl<S: UnicodeString> ContainerNode<S> {
    /// Wrap the nodes children in an html node of the same name
    fn fmt_default_html(
        &self,
        formatter: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        state: ToHtmlState,
    ) {
        let name = self.name();
        if !name.is_empty() {
            self.fmt_tag_open(name, formatter, &self.attrs);
        }

        self.fmt_children_html(formatter, selection_writer, state);

        if !name.is_empty() {
            self.fmt_tag_close(name, formatter);
        }
    }

    fn fmt_paragraph_html(
        &self,
        formatter: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        state: ToHtmlState,
    ) {
        assert!(matches!(self.kind, ContainerNodeKind::Paragraph));
        if state.is_inside_code_block {
            self.fmt_code_paragraph_html(formatter, selection_writer, state)
        } else {
            self.fmt_default_paragraph_html(formatter, selection_writer, state)
        }
    }

    fn fmt_default_paragraph_html(
        &self,
        formatter: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        state: ToHtmlState,
    ) {
        assert!(matches!(self.kind, ContainerNodeKind::Paragraph));
        let name = self.name();

        self.fmt_tag_open(name, formatter, &self.attrs);
        if self.is_empty() {
            formatter.push(char::nbsp());
        }
        self.fmt_children_html(formatter, selection_writer, state);
        self.fmt_tag_close(name, formatter);
    }

    fn fmt_code_paragraph_html(
        &self,
        formatter: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        state: ToHtmlState,
    ) {
        assert!(matches!(self.kind, ContainerNodeKind::Paragraph));
        if self.is_empty()
            && (state.is_last_node_in_parent || state.is_first_node_in_parent)
        {
            formatter.push(char::nbsp());
        }
        self.fmt_children_html(formatter, selection_writer, state);
        if !state.is_last_node_in_parent {
            formatter.push('\n');
        }
    }

    fn fmt_code_block_html(
        &self,
        formatter: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        state: ToHtmlState,
    ) {
        assert!(matches!(self.kind, ContainerNodeKind::CodeBlock));
        self.fmt_tag_open(&S::from("pre"), formatter, &self.attrs);
        let mut state = state;
        state.is_inside_code_block = true;

        self.fmt_tag_open(&S::from("code"), formatter, &None::<Vec<(S, S)>>);

        self.fmt_children_html(formatter, selection_writer, state);

        self.fmt_tag_close(&S::from("code"), formatter);
        self.fmt_tag_close(&S::from("pre"), formatter);
    }

    fn fmt_children_html(
        &self,
        formatter: &mut S,
        selection_writer: Option<&mut SelectionWriter>,
        state: ToHtmlState,
    ) {
        if let Some(w) = selection_writer {
            for (i, child) in self.children.iter().enumerate() {
                let state = self.updated_state(state, i);
                child.fmt_html(formatter, Some(w), state);
            }
            if self.is_empty() {
                w.write_selection_empty_container(
                    formatter,
                    formatter.len(),
                    self,
                )
            }
        } else {
            for (i, child) in self.children.iter().enumerate() {
                let state = self.updated_state(state, i);
                child.fmt_html(formatter, None, state);
            }
        }
    }

    fn fmt_tag_open(
        &self,
        name: &S::Str,
        formatter: &mut S,
        attrs: &Option<Vec<(S, S)>>,
    ) {
        formatter.push('<');
        formatter.push(name);
        if let Some(attrs) = attrs {
            for attr in attrs {
                let (attr_name, value) = attr;
                formatter.push(' ');
                formatter.push(&**attr_name);
                formatter.push("=\"");
                formatter.push(&**value);
                formatter.push('"');
            }
        }
        formatter.push('>');
    }

    fn fmt_tag_close(&self, name: &S::Str, formatter: &mut S) {
        formatter.push("</");
        formatter.push(name);
        formatter.push('>');
    }

    fn updated_state(
        &self,
        initial_state: ToHtmlState,
        child_index: usize,
    ) -> ToHtmlState {
        let is_last = self.children().len() == child_index + 1;
        let is_first = child_index == 0;
        let mut state = initial_state;
        state.is_last_node_in_parent = is_last;
        state.is_first_node_in_parent = is_first;
        state
    }
}

impl<S> ToRawText<S> for ContainerNode<S>
where
    S: UnicodeString,
{
    fn to_raw_text(&self) -> S {
        let mut text = S::default();
        for child in &self.children {
            text.push(child.to_raw_text());
        }
        text
    }
}

impl<S> ToPlainText<S> for ContainerNode<S>
where
    S: UnicodeString,
{
    fn to_plain_text(&self) -> S {
        let mut text = S::default();
        match self.kind {
            ContainerNodeKind::List(_) => fmt_list(self, &mut text),
            ContainerNodeKind::ListItem => fmt_list_item(self, &mut text),
            _ => fmt_default(self, &mut text),
        }
        return text;

        #[inline(always)]
        fn fmt_list<S: UnicodeString>(
            container: &ContainerNode<S>,
            text: &mut S,
        ) {
            for (index, child) in container.children.iter().enumerate() {
                if index != 0 && !matches!(text.chars().last(), Some('\n')) {
                    text.push("\n");
                }
                text.push(child.to_plain_text());
            }
            text.push("\n");
        }

        #[inline(always)]
        fn fmt_list_item<S: UnicodeString>(
            container: &ContainerNode<S>,
            text: &mut S,
        ) {
            for child in container.children() {
                text.push(child.to_plain_text());
            }
        }

        #[inline(always)]
        fn fmt_default<S: UnicodeString>(
            container: &ContainerNode<S>,
            text: &mut S,
        ) {
            for child in &container.children {
                text.push(child.to_plain_text());
            }
            if container.is_block_node()
                && !container.handle.is_root()
                && !matches!(text.chars().last(), Some('\n'))
            {
                text.push("\n");
            }
        }
    }
}

impl<S> ToTree<S> for ContainerNode<S>
where
    S: UnicodeString,
{
    fn to_tree_display(&self, continuous_positions: Vec<usize>) -> S {
        let mut description = self.name.clone();
        if let ContainerNodeKind::Link(url) = self.kind() {
            description.push(" \"");
            description.push(url.clone());
            description.push("\"");
        }

        let mut tree_part = self.tree_line(
            description,
            self.handle.raw().len(),
            continuous_positions.clone(),
        );

        for (i, child) in self.children.iter().enumerate() {
            let mut new_positions = continuous_positions.clone();
            if i < self.children.len() - 1 {
                new_positions.push(self.handle.raw().len());
            }
            tree_part.push(child.to_tree_display(new_positions));
        }
        tree_part
    }
}

impl<S> ToMarkdown<S> for ContainerNode<S>
where
    S: UnicodeString,
{
    fn fmt_markdown(
        &self,
        buffer: &mut S,
        options: &MarkdownOptions,
    ) -> Result<(), MarkdownError<S>> {
        use ContainerNodeKind::*;
        use InlineFormatType::*;

        let mut options = *options;

        match self.kind() {
            Generic => {
                fmt_children(self, buffer, &options)?;
            }

            // Simple emphasis.
            Formatting(Italic) => {
                fmt_italic(self, buffer, &options)?;
            }

            // Strong emphasis.
            Formatting(Bold) => {
                fmt_bold(self, buffer, &options)?;
            }

            Formatting(StrikeThrough) => {
                fmt_strikethrough(self, buffer, &options)?;
            }

            Formatting(Underline) => {
                fmt_underline(self, buffer, &options)?;
            }

            Formatting(InlineCode) => {
                fmt_inline_code(self, buffer, &mut options)?;
            }

            Link(url) => {
                fmt_link(self, buffer, &options, url)?;
            }

            List(_) => {
                fmt_list(self, buffer, &options)?;
            }

            ListItem => {
                fmt_list_item(self, buffer, &options)?;
            }

            CodeBlock => {
                fmt_code_block(self, buffer, &options)?;
            }

            Quote => {
                fmt_quote(self, buffer, &options)?;
            }

            Paragraph => {
                fmt_paragraph(self, buffer, &options)?;
            }
        };

        return Ok(());

        // `fmt_children` is a super basic loop over children to call
        // `fmt_markdown`, except that it inserts `\n` between block
        // nodes.
        #[inline(always)]
        fn fmt_children<S>(
            this: &ContainerNode<S>,
            buffer: &mut S,
            options: &MarkdownOptions,
        ) -> Result<(), MarkdownError<S>>
        where
            S: UnicodeString,
        {
            for (nth, child) in this.children.iter().enumerate() {
                if nth > 0 && child.is_block_node() {
                    buffer.push("\n");
                }

                child.fmt_markdown(buffer, options)?;
            }

            Ok(())
        }

        #[inline(always)]
        fn fmt_italic<S>(
            this: &ContainerNode<S>,
            buffer: &mut S,
            options: &MarkdownOptions,
        ) -> Result<(), MarkdownError<S>>
        where
            S: UnicodeString,
        {
            // Many implementations have restricted intrawords
            // simple emphasis to `*` to avoid unwanted emphasis
            // in words containing internal underscores, like
            // `foo_bar_baz`. We reckon it's good to follow this
            // trend to avoid unexpected behaviours for our users.

            buffer.push("*");
            fmt_children(this, buffer, options)?;
            buffer.push("*");

            Ok(())
        }

        #[inline(always)]
        fn fmt_bold<S>(
            this: &ContainerNode<S>,
            buffer: &mut S,
            options: &MarkdownOptions,
        ) -> Result<(), MarkdownError<S>>
        where
            S: UnicodeString,
        {
            // `Formatting(Italic)` already uses `*` to represent
            // a simple emphasis.
            //
            // We reckon it is better to use `_` to represent a
            // strong emphasis instead of `*` so that
            // `<em><strong>…</strong></em>` does _not_ produce
            // `***…***` or `___…___` which can be ambigiously
            // interpreted by various Markdown compilers out
            // there. Instead, it will produce `*__…__*`.

            buffer.push("__");
            fmt_children(this, buffer, options)?;
            buffer.push("__");

            Ok(())
        }

        #[inline(always)]
        fn fmt_strikethrough<S>(
            this: &ContainerNode<S>,
            buffer: &mut S,
            options: &MarkdownOptions,
        ) -> Result<(), MarkdownError<S>>
        where
            S: UnicodeString,
        {
            // Strikethrough is represented by a pair of one or
            // two `~`. We reckon using two `~` will avoid
            // ambiguous behaviours for users that manipulate
            // filesystem paths, or with Markdown compilers that
            // do not support this format extension.

            buffer.push("~~");
            fmt_children(this, buffer, options)?;
            buffer.push("~~");

            Ok(())
        }

        #[inline(always)]
        fn fmt_underline<S>(
            this: &ContainerNode<S>,
            buffer: &mut S,
            options: &MarkdownOptions,
        ) -> Result<(), MarkdownError<S>>
        where
            S: UnicodeString,
        {
            // Underline format is absent from Markdown. Let's
            // use raw HTML.

            buffer.push("<u>");
            fmt_children(this, buffer, options)?;
            buffer.push("</u>");

            Ok(())
        }

        #[inline(always)]
        fn fmt_inline_code<S>(
            this: &ContainerNode<S>,
            buffer: &mut S,
            options: &mut MarkdownOptions,
        ) -> Result<(), MarkdownError<S>>
        where
            S: UnicodeString,
        {
            // An inline code usually is usually delimited by an
            // opening and a closing single backtick. However, if
            // the inline code string contains a backtick, it is
            // preferable to use an opening and a closing double
            // backticks to delimit the inline code string.
            //
            // In addition to this subtlety, we add a space after
            // and before the opening and closing double backticks
            // to allow an inline code string to start by a
            // backtick. Those spaces are removed during
            // normalization.

            buffer.push("`` ");

            options.insert(MarkdownOptions::IGNORE_LINE_BREAK);
            fmt_children(this, buffer, options)?;

            buffer.push(" ``");

            Ok(())
        }

        #[inline(always)]
        fn fmt_link<S>(
            this: &ContainerNode<S>,
            buffer: &mut S,
            options: &MarkdownOptions,
            url: &S,
        ) -> Result<(), MarkdownError<S>>
        where
            S: UnicodeString,
        {
            buffer.push('[');

            fmt_children(this, buffer, options)?;

            // A link destination can be delimited by `<` and
            // `>`.
            //
            // The link URL can contain `<`, `>`, `(` and `)` if
            // they are escaped. Parenthesis, if unbalanced, can
            // not be escaped, but we are playing safety and
            // simplicity.

            buffer.push("](<");
            buffer.push(
                url.to_string()
                    .replace('<', "\\<")
                    .replace('>', "\\>")
                    .replace('(', "\\(")
                    .replace(')', "\\)")
                    .as_str(),
            );
            buffer.push(">)");

            Ok(())
        }

        #[inline(always)]
        fn fmt_list<S>(
            this: &ContainerNode<S>,
            buffer: &mut S,
            options: &MarkdownOptions,
        ) -> Result<(), MarkdownError<S>>
        where
            S: UnicodeString,
        {
            let list_type = this.name();
            let ordered_list_name = "ol";
            let expected_list_item_name = &S::from("li");
            let number_of_children = this.children.len();
            let mut ordered_list_counter = 0i32;

            for (nth, child) in this.children.iter().enumerate() {
                // Verify the list item is correct.
                let child = match child {
                    // Valid item.
                    DomNode::Container(
                        child @ ContainerNode {
                            name,
                            kind: ListItem,
                            ..
                        },
                    ) if name == expected_list_item_name => child,

                    // Item to ignore.
                    DomNode::Text(t) if t.is_blank() => {
                        continue;
                    }

                    // All the following are invalid items.
                    DomNode::Container(ContainerNode {
                        name: child_name,
                        ..
                    }) => {
                        return Err(MarkdownError::InvalidListItem(Some(
                            child_name.to_owned(),
                        )))
                    }

                    DomNode::LineBreak(line_break) => {
                        return Err(MarkdownError::InvalidListItem(Some(
                            line_break.name(),
                        )))
                    }

                    DomNode::Text(_) => {
                        return Err(MarkdownError::InvalidListItem(None))
                    }
                };

                // What's the current indentation, for this specific list only.
                let mut indentation = 0;

                // It's an ordered list.
                if list_type == ordered_list_name {
                    // Update the counter.
                    ordered_list_counter += 1;

                    // Generate something like `1.` (arabic numbers only,
                    // as requested by the specification).
                    let counter = ordered_list_counter.to_string();

                    buffer.push(counter.as_str());
                    buffer.push('.');

                    // Indentation will match the counter size.
                    indentation += counter.len();
                }
                // It's an unordered list.
                else {
                    // Generate something like `*`.
                    buffer.push('*');

                    // Indentation will match the counter size.
                    indentation += 1;
                }

                // Insert a space between the counter and the item's content.
                buffer.push(' ');

                // And update the indentation.
                indentation += 1;

                {
                    // Let's create a new buffer for the child formatting.
                    let mut child_buffer = S::default();
                    child.fmt_markdown(&mut child_buffer, options)?;

                    // Generate the indentation of form `\n` followed by
                    // $x$ spaces where $x$ is `indentation`.
                    let indentation = {
                        let spaces = " ".repeat(indentation);
                        let mut indentation =
                            String::with_capacity(1 /* `\n` */ + indentation);
                        indentation.push('\n');
                        indentation.push_str(&spaces);

                        indentation
                    };

                    // Insert the child's buffer after `\n`s have been
                    // replaced by `\n` followed by spaces for indentation.
                    buffer.push(
                        child_buffer
                            .to_string()
                            .replace('\n', &indentation)
                            .as_str(),
                    );
                }

                let is_last = nth == number_of_children - 1;

                if !is_last {
                    buffer.push('\n');
                }
            }

            Ok(())
        }

        #[inline(always)]
        fn fmt_list_item<S>(
            this: &ContainerNode<S>,
            buffer: &mut S,
            options: &MarkdownOptions,
        ) -> Result<(), MarkdownError<S>>
        where
            S: UnicodeString,
        {
            fmt_children(this, buffer, options)?;

            Ok(())
        }

        #[inline(always)]
        fn fmt_code_block<S>(
            this: &ContainerNode<S>,
            buffer: &mut S,
            options: &MarkdownOptions,
        ) -> Result<(), MarkdownError<S>>
        where
            S: UnicodeString,
        {
            buffer.push("```\n");
            fmt_children(this, buffer, options)?;
            buffer.push("\n```\n");

            Ok(())
        }

        #[inline(always)]
        fn fmt_quote<S>(
            this: &ContainerNode<S>,
            buffer: &mut S,
            options: &MarkdownOptions,
        ) -> Result<(), MarkdownError<S>>
        where
            S: UnicodeString,
        {
            buffer.push("> ");
            fmt_children(this, buffer, options)?;
            buffer.push("\n");

            Ok(())
        }

        #[inline(always)]
        fn fmt_paragraph<S>(
            this: &ContainerNode<S>,
            buffer: &mut S,
            options: &MarkdownOptions,
        ) -> Result<(), MarkdownError<S>>
        where
            S: UnicodeString,
        {
            fmt_children(this, buffer, options)?;

            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use widestring::Utf16String;

    use crate::tests::testutils_conversion::utf16;

    use super::*;

    #[test]
    fn adding_a_child_sets_the_correct_handle() {
        let mut node = container_with_handle(&[4, 5, 4]);

        // Append some children to a node
        node.append_child(text_node("0"));
        node.append_child(text_node("1"));
        node.append_child(text_node("2"));

        let text_node0 = &node.children[0];
        let text_node1 = &node.children[1];
        let text_node2 = &node.children[2];

        // Nodes got inserted in the right places
        assert_eq!(text_node0.to_html(), utf16("0"));
        assert_eq!(text_node1.to_html(), utf16("1"));
        assert_eq!(text_node2.to_html(), utf16("2"));

        // And they have the right handles
        assert_eq!(text_node0.handle().raw(), &[4, 5, 4, 0]);
        assert_eq!(text_node1.handle().raw(), &[4, 5, 4, 1]);
        assert_eq!(text_node2.handle().raw(), &[4, 5, 4, 2]);
    }

    #[test]
    fn removing_a_child_sets_the_correct_handles_after() {
        let mut node = container_with_handle(&[4, 5, 4]);
        node.append_child(text_node("0"));
        node.append_child(text_node("1"));
        node.append_child(text_node("2"));
        node.append_child(text_node("3"));

        // Remove 2 children from a node (reverse order to make indices nice)
        node.remove_child(2);
        node.remove_child(0);

        let text_node1 = &node.children[0];
        let text_node3 = &node.children[1];

        // The right nodes got deleted
        assert_eq!(text_node1.to_html(), utf16("1"));
        assert_eq!(text_node3.to_html(), utf16("3"));

        // And they have the right handles
        assert_eq!(text_node1.handle().raw(), &[4, 5, 4, 0]);
        assert_eq!(text_node3.handle().raw(), &[4, 5, 4, 1]);
    }

    #[test]
    fn inserting_children_updates_the_relevant_handles() {
        let mut node = container_with_handle(&[4, 5, 4]);

        node.append_child(text_node("0"));
        node.append_child(text_node("2"));
        node.append_child(text_node("3"));

        // Insert three new children before the second node
        let moved_handles = node.insert_children(
            1,
            vec![text_node("1a"), text_node("1b"), text_node("1c")],
        );

        let text_node0 = &node.children[0];
        let text_node1a = &node.children[1];
        let text_node1b = &node.children[2];
        let text_node1c = &node.children[3];
        let text_node2 = &node.children[4];
        let text_node3 = &node.children[5];

        // The new nodes got inserted in the right places
        assert_eq!(text_node0.to_html(), utf16("0"));
        assert_eq!(text_node1a.to_html(), utf16("1a"));
        assert_eq!(text_node1b.to_html(), utf16("1b"));
        assert_eq!(text_node1c.to_html(), utf16("1c"));
        assert_eq!(text_node2.to_html(), utf16("2"));
        assert_eq!(text_node3.to_html(), utf16("3"));

        assert_eq!(text_node0.handle().raw(), &[4, 5, 4, 0]);

        // The new children got inserted with the right handles
        assert_eq!(text_node1a.handle().raw(), &[4, 5, 4, 1]);
        assert_eq!(text_node1b.handle().raw(), &[4, 5, 4, 2]);
        assert_eq!(text_node1c.handle().raw(), &[4, 5, 4, 3]);

        // The previous node 2 & 3 were updated because it has moved to the right
        assert_eq!(text_node2.handle().raw(), &[4, 5, 4, 4]);
        assert_eq!(text_node3.handle().raw(), &[4, 5, 4, 5]);

        // Returned vec matches moved handles.
        assert_eq!(
            moved_handles,
            vec![text_node2.handle(), text_node3.handle()]
        );
    }

    #[test]
    fn replacing_child_updates_the_relevant_handles() {
        let mut node = container_with_handle(&[4, 5, 4]);

        node.append_child(text_node("0"));
        node.append_child(text_node("1"));
        node.append_child(text_node("2"));

        // Replace the middle child with three new ones
        let moved_handles = node.replace_child(
            1,
            vec![text_node("1a"), text_node("1b"), text_node("1c")],
        );

        let text_node0 = &node.children[0];
        let text_node1a = &node.children[1];
        let text_node1b = &node.children[2];
        let text_node1c = &node.children[3];
        let text_node2 = &node.children[4];

        // The new nodes got inserted in the right places
        assert_eq!(text_node0.to_html(), utf16("0"));
        assert_eq!(text_node1a.to_html(), utf16("1a"));
        assert_eq!(text_node1b.to_html(), utf16("1b"));
        assert_eq!(text_node1c.to_html(), utf16("1c"));
        assert_eq!(text_node2.to_html(), utf16("2"));

        assert_eq!(text_node0.handle().raw(), &[4, 5, 4, 0]);

        // The new children got inserted with the right handles
        assert_eq!(text_node1a.handle().raw(), &[4, 5, 4, 1]);
        assert_eq!(text_node1b.handle().raw(), &[4, 5, 4, 2]);
        assert_eq!(text_node1c.handle().raw(), &[4, 5, 4, 3]);

        // The previous node 2 was updated because it has moved to the right
        assert_eq!(text_node2.handle().raw(), &[4, 5, 4, 4]);

        // Returned vec matches moved handles.
        assert_eq!(moved_handles, vec![text_node2.handle()]);
    }

    #[test]
    fn pushing_container_of_same_kind() {
        let mut c1 =
            format_container_with_handle(InlineFormatType::Bold, &[0, 0]);
        c1.append_child(text_node("abc"));
        let mut c2 =
            format_container_with_handle(InlineFormatType::Bold, &[0, 1]);
        c2.append_child(text_node("def"));
        c2.append_child(DomNode::new_line_break());
        c1.push(&mut c2);
        assert!(c2.children().is_empty());

        let mut expected =
            format_container_with_handle(InlineFormatType::Bold, &[0, 0]);
        expected.append_child(text_node("abcdef"));
        expected.append_child(DomNode::new_line_break());

        assert_eq!(c1, expected)
    }

    #[test]
    #[should_panic]
    fn pushing_container_of_different_kind_panics() {
        let mut c1 =
            format_container_with_handle(InlineFormatType::Bold, &[0, 0]);
        c1.append_child(text_node("abc"));
        let mut c2 =
            format_container_with_handle(InlineFormatType::Italic, &[0, 1]);
        c2.append_child(text_node("def"));
        c1.push(&mut c2);
    }

    #[test]
    fn slicing_container_before() {
        let mut bold = create_container_with_nested_children();
        let mut before = bold.slice_before(2);
        assert_eq!(before.to_html(), "<strong><em>ab</em></strong>");
        assert_eq!(bold.to_html(), "<strong><em>c</em>def</strong>");
        // Just need any set handle, we're detached from any DOM.
        before.set_handle(DomHandle::root());
        before.push(&mut bold);
        assert_eq!(before.to_html(), "<strong><em>abc</em>def</strong>")
    }

    #[test]
    fn slicing_container_after() {
        let mut container = create_container_with_nested_children();
        let mut after = container.slice_after(2);
        assert_eq!(after.to_html(), "<strong><em>c</em>def</strong>");
        assert_eq!(container.to_html(), "<strong><em>ab</em></strong>");
        // Just need any set handle, we're detached from any DOM.
        after.set_handle(DomHandle::root());
        container.push(&mut after);
        assert_eq!(container.to_html(), "<strong><em>abc</em>def</strong>")
    }

    #[test]
    fn slicing_container_on_edge_does_nothing() {
        let mut container = create_container_with_nested_children();
        container.slice_before(0);
        container.slice_after(6);
        assert_eq!(container.to_html(), "<strong><em>abc</em>def</strong>")
    }

    #[test]
    #[should_panic]
    fn slicing_after_edge_panics() {
        let mut container = create_container_with_nested_children();
        container.slice_after(42);
    }

    /// Result HTML is "<strong><em>abc</em>def</strong>".
    fn create_container_with_nested_children() -> ContainerNode<Utf16String> {
        let mut bold =
            format_container_with_handle(InlineFormatType::Bold, &[0]);
        let mut italic =
            format_container_with_handle(InlineFormatType::Italic, &[0]);
        italic.append_child(text_node("abc"));
        bold.append_child(DomNode::Container(italic));
        bold.append_child(text_node("def"));
        assert_eq!(bold.to_html(), "<strong><em>abc</em>def</strong>");
        bold
    }

    fn container_with_handle<'a>(
        raw_handle: impl IntoIterator<Item = &'a usize>,
    ) -> ContainerNode<Utf16String> {
        let mut node = ContainerNode::new(
            Utf16String::from_str("div"),
            ContainerNodeKind::Generic,
            None,
            Vec::new(),
        );
        let handle =
            DomHandle::from_raw(raw_handle.into_iter().cloned().collect());
        node.set_handle(handle);
        node
    }

    fn format_container_with_handle<'a>(
        format: InlineFormatType,
        raw_handle: impl IntoIterator<Item = &'a usize>,
    ) -> ContainerNode<Utf16String> {
        let mut node = ContainerNode::new_formatting(format, vec![]);
        let handle =
            DomHandle::from_raw(raw_handle.into_iter().cloned().collect());
        node.set_handle(handle);
        node
    }

    fn text_node<S>(content: &str) -> DomNode<S>
    where
        S: UnicodeString,
    {
        DomNode::new_text(content.into())
    }
}
