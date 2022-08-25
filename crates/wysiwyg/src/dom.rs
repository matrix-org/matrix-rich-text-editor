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

pub mod dom_creation_error;
pub mod dom_handle;
pub mod dom_struct;
mod find_range;
pub mod find_result;
pub mod html_formatter;
pub mod nodes;
pub mod parser;
pub mod range;
pub mod to_html;
pub mod to_raw_text;
pub mod unicode_string;

pub use dom_creation_error::DomCreationError;
pub use dom_handle::DomHandle;
pub use dom_struct::Dom;
pub use find_result::FindResult;
pub use html_formatter::HtmlChar;
pub use html_formatter::HtmlFormatter;
pub use range::DomLocation;
pub use range::MultipleNodesRange;
pub use range::Range;
pub use range::SameNodeRange;
pub use to_html::ToHtml;
pub use to_raw_text::ToRawText;
pub use unicode_string::UnicodeString;
