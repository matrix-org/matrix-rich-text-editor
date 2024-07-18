use widestring::Utf16String;
use wysiwyg::Dom as InnerDom;
use wysiwyg::DomHandle;
use wysiwyg::DomNode as InnerDomNode;

#[derive(uniffi::Record)]
pub struct Dom {
    pub document: DomNode,
    pub transaction_id: u32,
}

impl Dom {
    pub fn from(inner: InnerDom<Utf16String>) -> Self {
        let node =
            DomNode::from(InnerDomNode::Container(inner.document().clone()));
        Dom {
            document: node,
            transaction_id: u32::try_from(inner.transaction_id).unwrap(),
        }
    }
}

#[derive(uniffi::Enum)]
pub enum DomNode {
    Container {
        path: Vec<u32>,
        kind: ContainerNodeKind,
        children: Vec<DomNode>,
    },
    Text {
        path: Vec<u32>,
        text: String,
    },
    LineBreak {
        path: Vec<u32>,
    },
    Mention {
        path: Vec<u32>,
    },
}

fn into_path(dom_handle: DomHandle) -> Vec<u32> {
    dom_handle
        .path
        .unwrap()
        .into_iter()
        .map(|x| u32::try_from(x).unwrap())
        .collect()
}

impl DomNode {
    pub fn from(inner: wysiwyg::DomNode<Utf16String>) -> Self {
        match inner {
            wysiwyg::DomNode::Container(node) => DomNode::Container {
                path: into_path(node.handle()),
                kind: ContainerNodeKind::from(node.kind().clone()),
                children: node
                    .children()
                    .iter()
                    .map(|x| DomNode::from(x.clone()))
                    .collect::<Vec<_>>(),
            },
            wysiwyg::DomNode::Text(node) => DomNode::Text {
                path: into_path(node.handle()),
                text: node.data().to_string(),
            },
            wysiwyg::DomNode::LineBreak(node) => DomNode::LineBreak {
                path: into_path(node.handle()),
            },
            wysiwyg::DomNode::Mention(node) => DomNode::LineBreak {
                path: into_path(node.handle()),
            },
        }
    }
}

#[derive(uniffi::Enum)]
pub enum ContainerNodeKind {
    Generic, // E.g. the root node (the containing div)
    Formatting(InlineFormatType),
    Link(String),
    List(ListType),
    ListItem,
    CodeBlock,
    Quote,
    Paragraph,
}

impl ContainerNodeKind {
    pub fn from(inner: wysiwyg::ContainerNodeKind<Utf16String>) -> Self {
        match inner {
            wysiwyg::ContainerNodeKind::Generic => ContainerNodeKind::Generic,
            wysiwyg::ContainerNodeKind::Formatting(format_type) => {
                ContainerNodeKind::Formatting(InlineFormatType::from(
                    format_type,
                ))
            }
            wysiwyg::ContainerNodeKind::Link(text) => {
                ContainerNodeKind::Link(text.to_string())
            }
            wysiwyg::ContainerNodeKind::List(list_type) => {
                ContainerNodeKind::List(ListType::from(list_type))
            }
            wysiwyg::ContainerNodeKind::ListItem => ContainerNodeKind::ListItem,
            wysiwyg::ContainerNodeKind::CodeBlock => {
                ContainerNodeKind::CodeBlock
            }
            wysiwyg::ContainerNodeKind::Quote => ContainerNodeKind::Quote,
            wysiwyg::ContainerNodeKind::Paragraph => {
                ContainerNodeKind::Paragraph
            }
        }
    }
}

#[derive(uniffi::Enum)]
pub enum InlineFormatType {
    Bold,
    Italic,
    StrikeThrough,
    Underline,
    InlineCode,
}

impl InlineFormatType {
    pub fn from(inner: wysiwyg::InlineFormatType) -> Self {
        match inner {
            wysiwyg::InlineFormatType::Bold => InlineFormatType::Bold,
            wysiwyg::InlineFormatType::Italic => InlineFormatType::Italic,
            wysiwyg::InlineFormatType::StrikeThrough => {
                InlineFormatType::StrikeThrough
            }
            wysiwyg::InlineFormatType::Underline => InlineFormatType::Underline,
            wysiwyg::InlineFormatType::InlineCode => {
                InlineFormatType::InlineCode
            }
        }
    }
}

#[derive(uniffi::Enum)]
pub enum ListType {
    Ordered,
    Unordered,
}

impl ListType {
    pub fn from(inner: wysiwyg::ListType) -> Self {
        match inner {
            wysiwyg::ListType::Ordered => ListType::Ordered,
            wysiwyg::ListType::Unordered => ListType::Unordered,
        }
    }
}
