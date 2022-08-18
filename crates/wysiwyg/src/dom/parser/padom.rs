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

use std::fmt::Display;

use super::{paqual_name, PaDomHandle, PaDomNode, PaNodeContainer, PaNodeText};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PaDom {
    pub(crate) nodes: Vec<PaDomNode>,
    pub(crate) document_handle: PaDomHandle,
}

impl PaDom {
    pub(crate) fn new() -> Self {
        let document = PaDomNode::Document(PaNodeContainer {
            name: paqual_name(""),
            attrs: Vec::new(),
            children: Vec::new(),
        });
        Self::from(document)
    }

    pub(crate) fn from(document: PaDomNode) -> Self {
        Self {
            nodes: vec![document],
            document_handle: PaDomHandle(0),
        }
    }

    pub(crate) fn get_node(&self, handle: &PaDomHandle) -> &PaDomNode {
        self.nodes
            .get(handle.0)
            .expect("Invalid handle passed to get_node")
    }

    pub(crate) fn get_mut_node(
        &mut self,
        handle: &PaDomHandle,
    ) -> &mut PaDomNode {
        self.nodes
            .get_mut(handle.0)
            .expect("Invalid handle passed to get_mut_node")
    }

    pub(crate) fn get_document(&self) -> &PaDomNode {
        self.nodes
            .get(self.document_handle.0)
            .expect("document_handle was invalid!")
    }

    pub(crate) fn document_handle(&self) -> &PaDomHandle {
        &self.document_handle
    }

    pub(crate) fn add_node(&mut self, node: PaDomNode) -> PaDomHandle {
        let handle = PaDomHandle(self.nodes.len());
        self.nodes.push(node);
        handle
    }

    pub(crate) fn create_element(
        &mut self,
        name: html5ever::QualName,
        attrs: Vec<html5ever::Attribute>,
        _flags: html5ever::tree_builder::ElementFlags,
    ) -> PaDomHandle {
        // We ignore flags
        let node = match name.local.as_ref() {
            "" => PaDomNode::Text(PaNodeText {
                content: String::from(""),
            }),
            _ => PaDomNode::Container(PaNodeContainer {
                name,
                attrs: attrs
                    .into_iter()
                    .map(|attr| {
                        (
                            attr.name.local.as_ref().to_owned(),
                            attr.value.as_ref().to_owned(),
                        )
                    })
                    .collect(),
                children: Vec::new(),
            }),
        };

        self.add_node(node)
    }
}

impl Display for PaDom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}
