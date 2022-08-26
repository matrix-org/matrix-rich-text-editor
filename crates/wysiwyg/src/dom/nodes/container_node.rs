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

use crate::dom::dom_handle::DomHandle;
use crate::dom::html_formatter::HtmlFormatter;
use crate::dom::nodes::dom_node::DomNode;
use crate::dom::to_html::ToHtml;
use crate::dom::{HtmlChar, UnicodeString};

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
    Generic,       // E.g. the root node (the containing div)
    Formatting(S), // TODO: the format parameter is a copy of name
    Link(S),
    List(S),
    ListItem(),
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

    pub fn new_formatting(format: S, children: Vec<DomNode<S>>) -> Self {
        Self {
            name: format.clone(),
            kind: ContainerNodeKind::Formatting(format),
            attrs: None,
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn new_list(list_type: S, children: Vec<DomNode<S>>) -> Self {
        Self {
            name: list_type.clone(),
            kind: ContainerNodeKind::List(list_type),
            attrs: None,
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn new_list_item(item_name: S, children: Vec<DomNode<S>>) -> Self {
        Self {
            name: item_name,
            kind: ContainerNodeKind::ListItem(),
            attrs: None,
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn append_child(&mut self, mut child: DomNode<S>) {
        assert!(self.handle.is_set());

        let child_index = self.children.len();
        let child_handle = self.handle.child_handle(child_index);
        child.set_handle(child_handle);
        self.children.push(child);
    }

    pub fn remove_child(&mut self, index: usize) {
        assert!(self.handle.is_set());
        assert!(index < self.children().len());

        self.children.remove(index);

        for child_index in index..self.children.len() {
            let new_handle = self.handle.child_handle(child_index);
            self.children[child_index].set_handle(new_handle);
        }
    }

    pub fn replace_child(&mut self, index: usize, nodes: Vec<DomNode<S>>) {
        assert!(self.handle.is_set());
        assert!(index < self.children().len());

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
            self.children[child_index].set_handle(new_handle);
        }
    }

    pub fn get_child_mut(&mut self, idx: usize) -> Option<&mut DomNode<S>> {
        self.children.get_mut(idx)
    }

    pub fn last_child_mut(&mut self) -> Option<&mut DomNode<S>> {
        self.children.last_mut()
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

    pub fn name(&self) -> &S {
        &self.name
    }

    pub fn attributes(&self) -> Option<&Vec<(S, S)>> {
        self.attrs.as_ref()
    }

    pub fn children(&self) -> &Vec<DomNode<S>> {
        &self.children
    }

    pub fn is_list_item(&self) -> bool {
        match self.kind {
            ContainerNodeKind::ListItem() => true,
            _ => false,
        }
    }

    pub fn text_len(&self) -> usize {
        self.children.iter().map(|child| child.text_len()).sum()
    }

    pub fn new_link(url: S, children: Vec<DomNode<S>>) -> Self {
        Self {
            name: S::from_str("a"),
            kind: ContainerNodeKind::Link(url.clone()),
            attrs: Some(vec![(S::from_str("href"), url)]),
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn is_empty_list_item(&self) -> bool {
        match self.kind {
            ContainerNodeKind::ListItem() => {
                // TODO: make a helper that reads all the plain text contained in a node and its children
                let html = self.to_html();
                return html.to_utf8() == "<li></li>"
                    || html.to_utf8() == "<li>\u{200b}</li>";
            }
            _ => false,
        }
    }
}

impl<S> ToHtml<S> for ContainerNode<S>
where
    S: UnicodeString,
{
    fn fmt_html(&self, f: &mut HtmlFormatter<S>) {
        let name = self.name();
        if !name.is_empty() {
            f.write_char(HtmlChar::Lt);
            f.write(name.as_slice());
            if let Some(attrs) = &self.attrs {
                for attr in attrs {
                    f.write_char(HtmlChar::Space);
                    let (attr_name, value) = attr;
                    f.write(attr_name.as_slice());
                    f.write_char(HtmlChar::Equal);
                    f.write_char(HtmlChar::Quote);
                    f.write(value.as_slice());
                    f.write_char(HtmlChar::Quote);
                }
            }
            f.write_char(HtmlChar::Gt);
        }

        for child in &self.children {
            child.fmt_html(f);
        }

        if !name.is_empty() {
            f.write_char(HtmlChar::Lt);
            f.write_char(HtmlChar::ForwardSlash);
            f.write(name.as_slice());
            f.write_char(HtmlChar::Gt);
        }
    }
}

#[cfg(test)]
mod test {
    use widestring::Utf16String;

    use crate::dom::nodes::TextNode;

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

    // TODO: more tests of ContainerNode

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
        DomNode::Text(TextNode::from(S::from_str(content)))
    }
}
