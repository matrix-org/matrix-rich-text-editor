use widestring::Utf16String;

#[derive(uniffi::Enum)]
pub enum DomNode {
    Container {
        id: u32,
        kind: ContainerNodeKind,
        children: Vec<DomNode>,
    },
    Text {
        id: u32,
        text: String,
    },
    LineBreak {
        id: u32,
    },
    Mention {
        id: u32,
    },
}

impl DomNode {
    pub fn from(inner: wysiwyg::DomNode<Utf16String>) -> Self {
        match inner {
            wysiwyg::DomNode::Container(node) => DomNode::Container {
                id: u32::try_from(node.id).unwrap(),
                kind: ContainerNodeKind::from(node.kind().clone()),
                children: node
                    .children()
                    .iter()
                    .map(|x| DomNode::from(x.clone()))
                    .collect::<Vec<_>>(),
            },
            wysiwyg::DomNode::Text(node) => DomNode::Text {
                id: u32::try_from(node.id).unwrap(),
                text: node.data().to_string(),
            },
            wysiwyg::DomNode::LineBreak(node) => DomNode::LineBreak {
                id: u32::try_from(node.id).unwrap(),
            },
            wysiwyg::DomNode::Mention(node) => DomNode::LineBreak {
                id: u32::try_from(node.id).unwrap(),
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
