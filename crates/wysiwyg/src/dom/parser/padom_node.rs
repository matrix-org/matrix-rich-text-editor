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

use html5ever::QualName;
use once_cell::sync::OnceCell;

use super::{paqual_name, PaNodeContainer, PaNodeText};

static TEXT: OnceCell<QualName> = OnceCell::new();
static ERROR: OnceCell<QualName> = OnceCell::new();

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PaDomNode {
    Container(PaNodeContainer),
    Document(PaNodeContainer),
    Text(PaNodeText),
    Error,
}

impl PaDomNode {
    pub(crate) fn name(&self) -> &QualName {
        match self {
            PaDomNode::Container(n) => &n.name,
            PaDomNode::Document(n) => &n.name,
            PaDomNode::Text(_) => q(&TEXT, ""),
            PaDomNode::Error => q(&ERROR, ""),
        }
    }
}

fn q<'a>(once_cell: &'a OnceCell<QualName>, local: &str) -> &'a QualName {
    once_cell.get_or_init(|| paqual_name(local))
}
