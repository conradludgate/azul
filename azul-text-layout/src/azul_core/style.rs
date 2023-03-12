//! DOM tree to CSS style tree cascading

use crate::azul_core::{
    dom::NodeData,
    id_tree::{NodeDataContainer, NodeDataContainerRef, NodeHierarchyRef, NodeId},
    styled_dom::NodeHierarchyItem,
};
use crate::css::{
    CssContentGroup, CssNthChildSelector::*, CssPath, CssPathPseudoSelector, CssPathSelector,
};

/// Has all the necessary information about the style CSS path
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct CascadeInfo {
    pub index_in_parent: u32,
    pub is_last_child: bool,
}

// impl CascadeInfoVec {
//     pub fn as_container<'a>(&'a self) -> NodeDataContainerRef<'a, CascadeInfo> {
//         NodeDataContainerRef {
//             internal: self.as_ref(),
//         }
//     }
// }

/// Returns if the style CSS path matches the DOM node (i.e. if the DOM node should be styled by that element)
pub(crate) fn matches_html_element(
    css_path: &CssPath,
    node_id: NodeId,
    node_hierarchy: &NodeDataContainerRef<NodeHierarchyItem>,
    node_data: &NodeDataContainerRef<NodeData>,
    html_node_tree: &NodeDataContainerRef<CascadeInfo>,
    expected_path_ending: Option<CssPathPseudoSelector>,
) -> bool {
    use self::CssGroupSplitReason::*;

    if css_path.selectors.is_empty() {
        return false;
    }

    let mut current_node = Some(node_id);
    let mut direct_parent_has_to_match = false;
    let mut last_selector_matched = true;

    let mut iterator = CssGroupIterator::new(css_path.selectors.as_ref());
    while let Some((content_group, reason)) = iterator.next() {
        let is_last_content_group = iterator.is_last_content_group();
        let cur_node_id = match current_node {
            Some(c) => c,
            None => {
                // The node has no parent, but the CSS path
                // still has an extra limitation - only valid if the
                // next content group is a "*" element
                return *content_group == [&CssPathSelector::Global];
            }
        };

        let current_selector_matches = selector_group_matches(
            &content_group,
            &html_node_tree[cur_node_id],
            &node_data[cur_node_id],
            expected_path_ending,
            is_last_content_group,
        );

        if direct_parent_has_to_match && !current_selector_matches {
            // If the element was a ">" element and the current,
            // direct parent does not match, return false
            return false; // not executed (maybe this is the bug)
        }

        // If the current selector matches, but the previous one didn't,
        // that means that the CSS path chain is broken and therefore doesn't match the element
        if current_selector_matches && !last_selector_matched {
            return false;
        }

        // Important: Set if the current selector has matched the element
        last_selector_matched = current_selector_matches;
        // Select if the next content group has to exactly match or if it can potentially be skipped
        direct_parent_has_to_match = reason == DirectChildren;
        current_node = node_hierarchy[cur_node_id].parent_id();
    }

    last_selector_matched
}

/// A CSS group is a group of css selectors in a path that specify the rule that a
/// certain node has to match, i.e. "div.main.foo" has to match three requirements:
///
/// - the node has to be of type div
/// - the node has to have the class "main"
/// - the node has to have the class "foo"
///
/// If any of these requirements are not met, the CSS block is discarded.
///
/// The CssGroupIterator splits the CSS path into semantic blocks, i.e.:
///
/// "body > .foo.main > #baz" will be split into ["body", ".foo.main" and "#baz"]
pub(crate) struct CssGroupIterator<'a> {
    pub css_path: &'a [CssPathSelector],
    pub current_idx: usize,
    pub last_reason: CssGroupSplitReason,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum CssGroupSplitReason {
    /// ".foo .main" - match any children
    Children,
    /// ".foo > .main" - match only direct children
    DirectChildren,
}

impl<'a> CssGroupIterator<'a> {
    pub fn new(css_path: &'a [CssPathSelector]) -> Self {
        let initial_len = css_path.len();
        Self {
            css_path,
            current_idx: initial_len,
            last_reason: CssGroupSplitReason::Children,
        }
    }
    pub fn is_last_content_group(&self) -> bool {
        self.current_idx.saturating_add(1) == self.css_path.len().saturating_sub(1)
    }
}

impl<'a> Iterator for CssGroupIterator<'a> {
    type Item = (CssContentGroup<'a>, CssGroupSplitReason);

    fn next(&mut self) -> Option<(CssContentGroup<'a>, CssGroupSplitReason)> {
        use self::CssPathSelector::*;

        let mut new_idx = self.current_idx;

        if new_idx == 0 {
            return None;
        }

        let mut current_path = Vec::new();

        while new_idx != 0 {
            match self.css_path.get(new_idx - 1)? {
                Children => {
                    self.last_reason = CssGroupSplitReason::Children;
                    break;
                }
                DirectChildren => {
                    self.last_reason = CssGroupSplitReason::DirectChildren;
                    break;
                }
                other => current_path.push(other),
            }
            new_idx -= 1;
        }

        // NOTE: Order inside of a ContentGroup is not important
        // for matching elements, only important for testing
        #[cfg(test)]
        current_path.reverse();

        if new_idx == 0 {
            if current_path.is_empty() {
                None
            } else {
                // Last element of path
                self.current_idx = 0;
                Some((current_path, self.last_reason))
            }
        } else {
            // skip the "Children | DirectChildren" element itself
            self.current_idx = new_idx - 1;
            Some((current_path, self.last_reason))
        }
    }
}

pub(crate) fn construct_html_cascade_tree(
    node_hierarchy: &NodeHierarchyRef,
    node_depths_sorted: &[(usize, NodeId)],
) -> NodeDataContainer<CascadeInfo> {
    let mut nodes = (0..node_hierarchy.len())
        .map(|_| CascadeInfo {
            index_in_parent: 0,
            is_last_child: false,
        })
        .collect::<Vec<_>>();

    for (_depth, parent_id) in node_depths_sorted {
        // Note: :nth-child() starts at 1 instead of 0
        let index_in_parent = parent_id.preceding_siblings(node_hierarchy).count();

        let parent_html_matcher = CascadeInfo {
            index_in_parent: (index_in_parent - 1) as u32,
            is_last_child: node_hierarchy[*parent_id].next_sibling.is_none(), // Necessary for :last selectors
        };

        nodes[parent_id.index()] = parent_html_matcher;

        for (child_idx, child_id) in parent_id.children(node_hierarchy).enumerate() {
            let child_html_matcher = CascadeInfo {
                index_in_parent: child_idx as u32,
                is_last_child: node_hierarchy[child_id].next_sibling.is_none(),
            };

            nodes[child_id.index()] = child_html_matcher;
        }
    }

    NodeDataContainer { internal: nodes }
}

/// TODO: This is wrong, but it's fast
#[inline]
pub fn rule_ends_with(path: &CssPath, target: Option<CssPathPseudoSelector>) -> bool {
    match target {
        None => match path.selectors.as_ref().last() {
            None => false,
            Some(q) => match q {
                CssPathSelector::PseudoSelector(_) => false,
                _ => true,
            },
        },
        Some(s) => match path.selectors.as_ref().last() {
            None => false,
            Some(q) => match q {
                CssPathSelector::PseudoSelector(q) => *q == s,
                _ => false,
            },
        },
    }
}

/// Matches a single group of items, panics on Children or DirectChildren selectors
///
/// The intent is to "split" the CSS path into groups by selectors, then store and cache
/// whether the direct or any parent has matched the path correctly
pub(crate) fn selector_group_matches(
    selectors: &[&CssPathSelector],
    html_node: &CascadeInfo,
    node_data: &NodeData,
    expected_path_ending: Option<CssPathPseudoSelector>,
    is_last_content_group: bool,
) -> bool {
    use self::CssPathSelector::*;

    for selector in selectors {
        match selector {
            Global => {}
            Type(t) => {
                if node_data.get_node_type().get_path() != *t {
                    return false;
                }
            }
            Class(c) => {
                if !node_data
                    .get_ids_and_classes()
                    .iter()
                    .filter_map(|i| i.as_class())
                    .any(|class| class == c.as_str())
                {
                    return false;
                }
            }
            Id(id) => {
                if !node_data
                    .get_ids_and_classes()
                    .iter()
                    .filter_map(|i| i.as_id())
                    .any(|html_id| html_id == id.as_str())
                {
                    return false;
                }
            }
            PseudoSelector(p) => {
                match p {
                    CssPathPseudoSelector::First => {
                        // Notice: index_in_parent is 1-indexed
                        if html_node.index_in_parent != 0 {
                            return false;
                        }
                    }
                    CssPathPseudoSelector::Last => {
                        // Notice: index_in_parent is 1-indexed
                        if !html_node.is_last_child {
                            return false;
                        }
                    }
                    CssPathPseudoSelector::NthChild(x) => {
                        use crate::css::CssNthChildPattern;
                        let index_in_parent = html_node.index_in_parent + 1; // nth-child starts at 1!
                        match *x {
                            Number(value) => {
                                if index_in_parent != value {
                                    return false;
                                }
                            }
                            Even => {
                                if index_in_parent % 2 == 0 {
                                    return false;
                                }
                            }
                            Odd => {
                                if index_in_parent % 2 == 1 {
                                    return false;
                                }
                            }
                            Pattern(CssNthChildPattern { repeat, offset }) => {
                                if index_in_parent >= offset
                                    && ((index_in_parent - offset) % repeat != 0)
                                {
                                    return false;
                                }
                            }
                        }
                    }

                    // NOTE: for all other selectors such as :hover, :focus and :active,
                    // we can only apply them if they appear in the last content group,
                    // i.e. this will match "body > #main:hover", but not "body:hover > #main"
                    CssPathPseudoSelector::Hover => {
                        if !is_last_content_group {
                            return false;
                        }
                        if expected_path_ending != Some(CssPathPseudoSelector::Hover) {
                            return false;
                        }
                    }
                    CssPathPseudoSelector::Active => {
                        if !is_last_content_group {
                            return false;
                        }
                        if expected_path_ending != Some(CssPathPseudoSelector::Active) {
                            return false;
                        }
                    }
                    CssPathPseudoSelector::Focus => {
                        if !is_last_content_group {
                            return false;
                        }
                        if expected_path_ending != Some(CssPathPseudoSelector::Focus) {
                            return false;
                        }
                    }
                }
            }
            DirectChildren | Children => {
                // panic!("Unreachable: DirectChildren or Children in CSS path!");
                return false;
            }
        }
    }

    true
}

// #[test]
// fn test_case_issue_93() {

//     use azul_css::CssPathSelector::*;
//     use azul_css::*;
//     use crate::dom::*;

//     fn render_tab() -> Dom {
//         Dom::div().with_class("tabwidget-tab")
//             .with_child(Dom::label("").with_class("tabwidget-tab-label"))
//             .with_child(Dom::label("").with_class("tabwidget-tab-close"))
//     }

//     let dom = Dom::div().with_id("editor-rooms")
//     .with_child(
//         Dom::div().with_class("tabwidget-bar")
//         .with_child(render_tab().with_class("active"))
//         .with_child(render_tab())
//         .with_child(render_tab())
//         .with_child(render_tab())
//     );

//     let dom = convert_dom_into_compact_dom(dom);

//     let tab_active_close = CssPath { selectors: vec![
//         Class("tabwidget-tab".to_string().into()),
//         Class("active".to_string().into()),
//         Children,
//         Class("tabwidget-tab-close".to_string().into())
//     ].into() };

//     let node_hierarchy = &dom.arena.node_hierarchy;
//     let node_data = &dom.arena.node_data;
//     let nodes_sorted: Vec<_> = node_hierarchy.get_parents_sorted_by_depth();
//     let html_node_tree = construct_html_cascade_tree(
//         &node_hierarchy,
//         &nodes_sorted,
//         None,
//         &BTreeMap::new(),
//         false,
//     );

//     //  rules: [
//     //    ".tabwidget-tab-label"                        : ColorU::BLACK,
//     //    ".tabwidget-tab.active .tabwidget-tab-label"  : ColorU::WHITE,
//     //    ".tabwidget-tab.active .tabwidget-tab-close"  : ColorU::RED,
//     //  ]

//     //  0: [div #editor-rooms ]
//     //   |-- 1: [div  .tabwidget-bar]
//     //   |    |-- 2: [div  .tabwidget-tab .active]
//     //   |    |    |-- 3: [p  .tabwidget-tab-label]
//     //   |    |    |-- 4: [p  .tabwidget-tab-close]
//     //   |    |-- 5: [div  .tabwidget-tab]
//     //   |    |    |-- 6: [p  .tabwidget-tab-label]
//     //   |    |    |-- 7: [p  .tabwidget-tab-close]
//     //   |    |-- 8: [div  .tabwidget-tab]
//     //   |    |    |-- 9: [p  .tabwidget-tab-label]
//     //   |    |    |-- 10: [p  .tabwidget-tab-close]
//     //   |    |-- 11: [div  .tabwidget-tab]
//     //   |    |    |-- 12: [p  .tabwidget-tab-label]
//     //   |    |    |-- 13: [p  .tabwidget-tab-close]

//     // Test 1:
//     // ".tabwidget-tab.active .tabwidget-tab-label"
//     // should not match
//     // ".tabwidget-tab.active .tabwidget-tab-close"
//     assert_eq!(matches_html_element(&tab_active_close, NodeId::new(3), &node_hierarchy, &node_data, &html_node_tree), false);

//     // Test 2:
//     // ".tabwidget-tab.active .tabwidget-tab-close"
//     // should match
//     // ".tabwidget-tab.active .tabwidget-tab-close"
//     assert_eq!(matches_html_element(&tab_active_close, NodeId::new(4), &node_hierarchy, &node_data, &html_node_tree), true);
// }

#[test]
fn test_css_group_iterator() {
    use self::CssPathSelector::*;
    use crate::css::*;

    // ".hello > #id_text.new_class div.content"
    // -> ["div.content", "#id_text.new_class", ".hello"]
    let selectors = vec![
        Class("hello".to_string().into()),
        DirectChildren,
        Id("id_test".to_string().into()),
        Class("new_class".to_string().into()),
        Children,
        Type(NodeTypeTag::Div),
        Class("content".to_string().into()),
    ];

    let mut it = CssGroupIterator::new(&selectors);

    assert_eq!(
        it.next(),
        Some((
            vec![
                &Type(NodeTypeTag::Div),
                &Class("content".to_string().into()),
            ],
            CssGroupSplitReason::Children
        ))
    );

    assert_eq!(
        it.next(),
        Some((
            vec![
                &Id("id_test".to_string().into()),
                &Class("new_class".to_string().into()),
            ],
            CssGroupSplitReason::DirectChildren
        ))
    );

    assert_eq!(
        it.next(),
        Some((
            vec![&Class("hello".into()),],
            CssGroupSplitReason::DirectChildren
        ))
    ); // technically not correct

    assert_eq!(it.next(), None);

    // Test single class
    let selectors_2 = vec![Class("content".to_string().into())];

    let mut it = CssGroupIterator::new(&selectors_2);

    assert_eq!(
        it.next(),
        Some((
            vec![&Class("content".to_string().into()),],
            CssGroupSplitReason::Children
        ))
    );

    assert_eq!(it.next(), None);
}
