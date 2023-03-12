use core::fmt;

use super::{app_resources::IdNamespace, id_tree::NodeId};
use crate::ui_solver::{InlineTextLayout, ResolvedTextLayoutOptions};

pub trait GetTextLayout {
    // self is mutable so that the calculated text can be cached if it hasn't changed since the last frame
    fn get_text_layout(
        &mut self,
        document_id: &DocumentId,
        node_id: NodeId,
        text_layout_options: &ResolvedTextLayoutOptions,
    ) -> InlineTextLayout;
}

#[derive(Copy, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct DocumentId {
    pub namespace_id: IdNamespace,
    pub id: u32,
}

impl ::core::fmt::Display for DocumentId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "DocumentId {{ ns: {}, id: {} }}",
            self.namespace_id, self.id
        )
    }
}

impl ::core::fmt::Debug for DocumentId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
