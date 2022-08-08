use wysiwyg::ToHtml;

pub struct ComposerState {
    pub html: Vec<u16>,
    pub start: u32,
    pub end: u32,
}

impl From<wysiwyg::ComposerState<u16>> for ComposerState {
    fn from(state: wysiwyg::ComposerState<u16>) -> Self {
        let start_utf16_codeunit: usize = state.start.into();
        let end_utf16_codeunit: usize = state.end.into();
        Self {
            html: state.dom.to_html(),
            start: u32::try_from(start_utf16_codeunit).unwrap(),
            end: u32::try_from(end_utf16_codeunit).unwrap(),
        }
    }
}
