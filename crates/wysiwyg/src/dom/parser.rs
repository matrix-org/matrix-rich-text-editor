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

//! Classes used for parsing HTML into a [super::Dom].
//!
//! We do this by creating a temporary structure held inside a [PaDom]
//! but we throw that away at the end of parsing, and return just a
//! [super::Dom]. All instances of classes within this module are thrown away
//! when parsing finishes.

#[cfg(feature = "sys")]
mod padom;
#[cfg(feature = "sys")]
mod padom_creation_error;
#[cfg(feature = "sys")]
mod padom_creator;
#[cfg(feature = "sys")]
mod padom_handle;
#[cfg(feature = "sys")]
mod padom_node;
#[cfg(feature = "sys")]
mod panode_container;
#[cfg(feature = "sys")]
mod panode_text;
#[cfg(feature = "sys")]
mod paqual_name;
mod parse;

#[cfg(feature = "sys")]
use padom::PaDom;
#[cfg(feature = "sys")]
use padom_creation_error::PaDomCreationError;
#[cfg(feature = "sys")]
use padom_creator::PaDomCreator;
#[cfg(feature = "sys")]
use padom_handle::PaDomHandle;
#[cfg(feature = "sys")]
use padom_node::PaDomNode;
#[cfg(feature = "sys")]
use panode_container::PaNodeContainer;
#[cfg(feature = "sys")]
use panode_text::PaNodeText;
#[cfg(feature = "sys")]
use paqual_name::paqual_name;

pub use parse::parse;
