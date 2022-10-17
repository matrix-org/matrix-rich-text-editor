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

use crate::composer_model::example_format::SelectionWriter;
use crate::dom::dom_handle::DomHandle;
use crate::dom::nodes::dom_node::DomNode;
use crate::dom::to_html::ToHtml;
#[cfg(feature = "to-markdown")]
use crate::dom::to_markdown::{MarkdownError, MarkdownOptions, ToMarkdown};
use crate::dom::to_raw_text::ToRawText;
use crate::dom::to_tree::ToTree;
use crate::dom::unicode_string::{UnicodeStrExt, UnicodeStringExt};
use crate::dom::UnicodeString;
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

#[derive(Clone, Debug, PartialEq)]
pub enum ContainerNodeKind<S>
where
    S: UnicodeString,
{
    Generic, // E.g. the root node (the containing div)
    Formatting(InlineFormatType),
    Link(S),
    List,
    ListItem,
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
            kind: ContainerNodeKind::List,
            attrs: None,
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn new_list_item(item_name: S, children: Vec<DomNode<S>>) -> Self {
        Self {
            name: item_name,
            kind: ContainerNodeKind::ListItem,
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

    pub fn replace_child(
        &mut self,
        index: usize,
        nodes: Vec<DomNode<S>>,
    ) -> Vec<DomHandle> {
        assert!(self.handle.is_set());
        assert!(index < self.children().len());

        let mut handles = Vec::new();

        self.children.remove(index);
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

    pub fn get_child_mut(&mut self, idx: usize) -> Option<&mut DomNode<S>> {
        self.children.get_mut(idx)
    }

    pub fn last_child_mut(&mut self) -> Option<&mut DomNode<S>> {
        self.children.last_mut()
    }

    pub fn insert_child(&mut self, index: usize, node: DomNode<S>) {
        assert!(self.handle.is_set());
        assert!(index <= self.children().len());

        self.children.insert(index, node);

        for i in index..self.children.len() {
            let new_handle = self.handle.child_handle(i);
            self.children[i].set_handle(new_handle);
        }
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

    pub fn kind(&self) -> &ContainerNodeKind<S> {
        &self.kind
    }

    pub fn is_list_item(&self) -> bool {
        matches!(self.kind, ContainerNodeKind::ListItem)
    }

    pub(crate) fn is_list_of_type(&self, list_type: ListType) -> bool {
        match self.kind {
            ContainerNodeKind::List => {
                return ListType::try_from(self.name().to_owned()).unwrap()
                    == list_type;
            }
            _ => false,
        }
    }

    pub(crate) fn is_structure_node(&self) -> bool {
        use ContainerNodeKind::*;

        matches!(self.kind, List | ListItem)
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
        use ContainerNodeKind::*;

        matches!(self.kind, Generic | List)
    }

    pub fn text_len(&self) -> usize {
        self.children.iter().map(|child| child.text_len()).sum()
    }

    pub fn new_link(url: S, children: Vec<DomNode<S>>) -> Self {
        Self {
            name: "a".into(),
            kind: ContainerNodeKind::Link(url.clone()),
            attrs: Some(vec![("href".into(), url)]),
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn is_empty_list_item(&self) -> bool {
        match self.kind {
            ContainerNodeKind::ListItem => {
                let raw_text = self.to_raw_text().to_string();
                raw_text.is_empty() || raw_text == "\u{200b}"
            }
            _ => false,
        }
    }

    pub(crate) fn set_list_type(&mut self, list_type: ListType) {
        match self.kind {
            ContainerNodeKind::List => {
                self.name = list_type.tag().into();
            }
            _ => panic!(
                "Setting list type to a non-list container is not allowed"
            ),
        }
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
        _: bool,
    ) {
        let name = self.name();
        if !name.is_empty() {
            formatter.push('<');
            formatter.push(name);
            if let Some(attrs) = &self.attrs {
                for attr in attrs {
                    formatter.push(' ');
                    let (attr_name, value) = attr;
                    formatter.push(&**attr_name);
                    formatter.push('=');
                    formatter.push('"');
                    formatter.push(&**value);
                    formatter.push('"');
                }
            }
            formatter.push('>');
        }

        if let Some(w) = selection_writer {
            for (i, child) in self.children.iter().enumerate() {
                let is_last = self.children().len() == i + 1;
                child.fmt_html(formatter, Some(w), is_last);
            }
        } else {
            for (i, child) in self.children.iter().enumerate() {
                let is_last = self.children().len() == i + 1;
                child.fmt_html(formatter, None, is_last);
            }
        }

        if !name.is_empty() {
            formatter.push('<');
            formatter.push('/');
            formatter.push(name);
            formatter.push('>');
        }
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

#[cfg(feature = "to-markdown")]
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

        // `fmt_children` is super basic loop over children to call
        // `fmt_markdown`, except that it inserts `\n` between block
        // nodes.
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

        let mut options = *options;
        options.insert(MarkdownOptions::IN_A_CONTAINER);

        match self.kind() {
            Generic => {
                fmt_children(self, buffer, &options)?;
            }

            // Simple emphasis.
            Formatting(Italic) => {
                // Many implementations have restricted intrawords
                // simple emphasis to `*` to avoid unwanted emphasis
                // in words containing internal underscores, like
                // `foo_bar_baz`. We reckon it's good to follow this
                // trend to avoid unexpected behaviours for our users.

                buffer.push("*");
                fmt_children(self, buffer, &options)?;
                buffer.push("*");
            }

            // Strong emphasis.
            Formatting(Bold) => {
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
                fmt_children(self, buffer, &options)?;
                buffer.push("__");
            }

            Formatting(StrikeThrough) => {
                // Strikethrough is represented by a pair of one or
                // two `~`. We reckon using two `~` will avoid
                // ambiguous behaviours for users that manipulate
                // filesystem paths, or with Markdown compilers that
                // do not support this format extension.

                buffer.push("~~");
                fmt_children(self, buffer, &options)?;
                buffer.push("~~");
            }

            Formatting(Underline) => {
                // Underline format is absent from Markdown. Let's
                // ignore it!

                fmt_children(self, buffer, &options)?;
            }

            Formatting(InlineCode) => {
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
                fmt_children(self, buffer, &options)?;

                buffer.push(" ``");
            }

            Link(url) => {
                buffer.push('[');

                fmt_children(self, buffer, &options)?;

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
            }

            List => {
                let list_type = self.name();
                let ordered_list_name = "ol";
                let expected_list_item_name = &S::from("li");
                let number_of_children = self.children.len();
                let mut ordered_list_counter = 0i32;

                for (nth, child) in self.children.iter().enumerate() {
                    // Verify the list item is correct.
                    let child = match child {
                        // Valid item.
                        DomNode::Container(
                            child @ Self {
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
                        DomNode::Container(Self {
                            name: child_name, ..
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
                        indentation += counter.len() + 1 /* `.` */;
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
                        child.fmt_markdown(&mut child_buffer, &options)?;

                        // Generate the indentation of form `\n` followed by
                        // $x$ spaces where $x$ is `indentation`.
                        let indentation = {
                            let spaces = " ".repeat(indentation);
                            let mut indentation = String::with_capacity(
                                1 /* `\n` */ + indentation,
                            );
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
            }

            ListItem => {
                fmt_children(self, buffer, &options)?;
            }
        };

        Ok(())
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
    fn replacing_child_updates_the_relevant_handles() {
        let mut node = container_with_handle(&[4, 5, 4]);

        node.append_child(text_node("0"));
        node.append_child(text_node("1"));
        node.append_child(text_node("2"));

        // Replace the middle child with three new ones
        node.replace_child(
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

    fn text_node<S>(content: &str) -> DomNode<S>
    where
        S: UnicodeString,
    {
        DomNode::new_text(content.into())
    }
}
