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
use crate::dom::to_html::{fmt_node_u16, ToHtml};

#[derive(Clone, Debug, PartialEq)]
pub struct ContainerNode<C>
where
    C: Clone,
{
    name: Vec<C>,
    kind: ContainerNodeKind<C>,
    attrs: Option<Vec<(Vec<C>, Vec<C>)>>,
    children: Vec<DomNode<C>>,
    handle: DomHandle,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ContainerNodeKind<C: Clone> {
    Generic,            // E.g. the root node (the containing div)
    Formatting(Vec<C>), // TODO: the format parameter is a copy of name
    Link(Vec<C>),
    List(Vec<C>),
    ListItem(),
}

impl<C> ContainerNode<C>
where
    C: Clone,
{
    /// Create a new ContainerNode
    ///
    /// NOTE: Its handle() will be unset until you call set_handle() or
    /// append() it to another node.
    pub fn new(
        name: Vec<C>,
        kind: ContainerNodeKind<C>,
        attrs: Option<Vec<(Vec<C>, Vec<C>)>>,
        children: Vec<DomNode<C>>,
    ) -> Self {
        Self {
            name,
            kind,
            attrs,
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn new_formatting(format: Vec<C>, children: Vec<DomNode<C>>) -> Self {
        Self {
            name: format.clone(),
            kind: ContainerNodeKind::Formatting(format),
            attrs: None,
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn new_list(list_type: Vec<C>, children: Vec<DomNode<C>>) -> Self {
        Self {
            name: list_type.clone(),
            kind: ContainerNodeKind::List(list_type),
            attrs: None,
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn new_list_item(item_name: Vec<C>, children: Vec<DomNode<C>>) -> Self {
        Self {
            name: item_name,
            kind: ContainerNodeKind::ListItem(),
            attrs: None,
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn append(&mut self, mut child: DomNode<C>) {
        assert!(self.handle.is_set());

        let child_index = self.children.len();
        let child_handle = self.handle.child_handle(child_index);
        child.set_handle(child_handle);
        self.children.push(child);
    }

    pub fn remove(&mut self, index: usize) {
        assert!(self.handle.is_set());
        assert!(index < self.children().len());

        self.children.remove(index);

        for child_index in index..self.children.len() {
            let new_handle = self.handle.child_handle(child_index);
            self.children[child_index].set_handle(new_handle);
        }
    }

    pub fn replace_child(&mut self, index: usize, nodes: Vec<DomNode<C>>) {
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

    pub fn handle(&self) -> DomHandle {
        self.handle.clone()
    }

    pub fn set_handle(&mut self, handle: DomHandle) {
        self.handle = handle;
        for (i, child) in self.children.iter_mut().enumerate() {
            child.set_handle(self.handle.child_handle(i))
        }
    }

    pub fn name(&self) -> &Vec<C> {
        &self.name
    }

    pub fn attributes(&self) -> Option<&Vec<(Vec<C>, Vec<C>)>> {
        self.attrs.as_ref()
    }

    pub fn children(&self) -> &Vec<DomNode<C>> {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut Vec<DomNode<C>> {
        // TODO: replace with soemthing like get_child_mut - we want to avoid
        // anyone pushing onto this, because the handles will be unset
        &mut self.children
    }

    pub fn is_list_item(&self) -> bool {
        match self.kind {
            ContainerNodeKind::ListItem() => true,
            _ => false,
        }
    }

    pub fn len(&self) -> usize {
        let mut result: usize = 0;
        for child in self.children() {
            result += child.len()
        }
        result
    }
}

impl ContainerNode<u16> {
    pub fn new_link(url: Vec<u16>, children: Vec<DomNode<u16>>) -> Self {
        Self {
            name: "a".encode_utf16().collect(),
            kind: ContainerNodeKind::Link(url.clone()),
            attrs: Some(vec![("href".encode_utf16().collect(), url)]),
            children,
            handle: DomHandle::new_unset(),
        }
    }

    pub fn is_empty_list_item(&self) -> bool {
        match self.kind {
            ContainerNodeKind::ListItem() => {
                // TODO: make a helper that reads all the plain text contained in a node and its children
                let html = self.to_html();
                return html == "<li></li>".to_html()
                    || html == "<li>\u{200b}</li>".to_html();
            }
            _ => false,
        }
    }
}

impl ToHtml<u16> for ContainerNode<u16> {
    fn fmt_html(&self, f: &mut HtmlFormatter<u16>) {
        fmt_node_u16(self, f)
    }
}

#[cfg(test)]
mod test {
    use crate::dom::nodes::TextNode;

    use super::*;

    #[test]
    fn adding_a_child_sets_the_correct_handle() {
        let mut node = container_with_handle(&[4, 5, 4]);

        // Append some children to a node
        node.append(text_node("0"));
        node.append(text_node("1"));
        node.append(text_node("2"));

        let text_node0 = &node.children[0];
        let text_node1 = &node.children[1];
        let text_node2 = &node.children[2];

        // Nodes got inserted in the right places
        assert_eq!(text_node0.to_html(), "0".to_html());
        assert_eq!(text_node1.to_html(), "1".to_html());
        assert_eq!(text_node2.to_html(), "2".to_html());

        // And they have the right handles
        assert_eq!(text_node0.handle().raw(), &[4, 5, 4, 0]);
        assert_eq!(text_node1.handle().raw(), &[4, 5, 4, 1]);
        assert_eq!(text_node2.handle().raw(), &[4, 5, 4, 2]);
    }

    #[test]
    fn removing_a_child_sets_the_correct_handles_after() {
        let mut node = container_with_handle(&[4, 5, 4]);
        node.append(text_node("0"));
        node.append(text_node("1"));
        node.append(text_node("2"));
        node.append(text_node("3"));

        // Remove 2 children from a node (reverse order to make indices nice)
        node.remove(2);
        node.remove(0);

        let text_node1 = &node.children[0];
        let text_node3 = &node.children[1];

        // The right nodes got deleted
        assert_eq!(text_node1.to_html(), "1".to_html());
        assert_eq!(text_node3.to_html(), "3".to_html());

        // And they have the right handles
        assert_eq!(text_node1.handle().raw(), &[4, 5, 4, 0]);
        assert_eq!(text_node3.handle().raw(), &[4, 5, 4, 1]);
    }

    #[test]
    fn replacing_child_updates_the_relevant_handles() {
        let mut node = container_with_handle(&[4, 5, 4]);

        node.append(text_node("0"));
        node.append(text_node("1"));
        node.append(text_node("2"));

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
        assert_eq!(text_node0.to_html(), "0".to_html());
        assert_eq!(text_node1a.to_html(), "1a".to_html());
        assert_eq!(text_node1b.to_html(), "1b".to_html());
        assert_eq!(text_node1c.to_html(), "1c".to_html());
        assert_eq!(text_node2.to_html(), "2".to_html());

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
    ) -> ContainerNode<u16> {
        let mut node = ContainerNode::new(
            "div".to_html(),
            ContainerNodeKind::Generic,
            None,
            Vec::new(),
        );
        let handle =
            DomHandle::from_raw(raw_handle.into_iter().cloned().collect());
        node.set_handle(handle);
        node
    }

    fn text_node(content: &str) -> DomNode<u16> {
        DomNode::Text(TextNode::from(content.to_html()))
    }
}
