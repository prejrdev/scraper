use html5ever::{LocalName, Namespace};
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::matching;
use selectors::{Element, OpaqueElement};

use super::ElementRef;
use crate::selector::{NonTSPseudoClass, PseudoElement, Simple};

fn map_b_to_cs(case_sensitive: bool) -> CaseSensitivity{
    if case_sensitive{
        CaseSensitivity::CaseSensitive
    }else{
        CaseSensitivity::AsciiCaseInsensitive
    }
}

/// Note: will never match against non-tree-structure pseudo-classes.
impl<'a> Element for ElementRef<'a> {
    type Impl = Simple;

    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(self.node.value())
    }

    fn parent_element(&self) -> Option<Self> {
        self.parent().and_then(ElementRef::wrap)
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        false
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        None
    }

    fn is_pseudo_element(&self) -> bool {
        false
    }

    fn is_part(&self, _name: &LocalName) -> bool {
        false
    }

    fn is_same_type(&self, other: &Self) -> bool {
        self.value().name == other.value().name
    }

    fn exported_part(&self, _: &LocalName) -> Option<LocalName> {
        None
    }

    fn imported_part(&self, _: &LocalName) -> Option<LocalName> {
        None
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        self.prev_siblings()
            .find(|sibling| sibling.value().is_element())
            .map(ElementRef::new)
    }

    fn next_sibling_element(&self) -> Option<Self> {
        self.next_siblings()
            .find(|sibling| sibling.value().is_element())
            .map(ElementRef::new)
    }

    fn is_html_element_in_html_document(&self) -> bool {
        // FIXME: Is there more to this?
        self.value().name.ns == ns!(html)
    }

    fn has_local_name(&self, name: &LocalName) -> bool {
        &self.value().name.local == name
    }

    fn has_namespace(&self, namespace: &Namespace) -> bool {
        &self.value().name.ns == namespace
    }

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&Namespace>,
        local_name: &LocalName,
        operation: &AttrSelectorOperation<&String>,
    ) -> bool {
        self.value().attrs.iter().any(|(key, value)| {
            !matches!(*ns, NamespaceConstraint::Specific(url) if *url != key.ns)
                && *local_name == key.local
                && operation.eval_str(value)
        })
    }

    fn match_non_ts_pseudo_class<F>(
        &self,
        _pc: &NonTSPseudoClass,
        _context: &mut matching::MatchingContext<Self::Impl>,
        _flags_setter: &mut F,
    ) -> bool {
        false
    }

    fn match_pseudo_element(
        &self,
        _pe: &PseudoElement,
        _context: &mut matching::MatchingContext<Self::Impl>,
    ) -> bool {
        false
    }

    fn is_link(&self) -> bool {
        self.value().name() == "link"
    }

    fn is_html_slot_element(&self) -> bool {
        true
    }

    fn has_id(&self, id: &LocalName, case_sensitive: CaseSensitivity) -> bool {               
        let case_sensitivity = case_sensitive;
        match self.value().id {
            Some(ref val) => case_sensitivity.eq(id.as_bytes(), val.as_bytes()),
            None => false,
        }
        
    }

    fn has_class(&self, name: &LocalName, case_sensitive: CaseSensitivity) -> bool {
        self.value().has_class(name, case_sensitive)
    }

    fn is_empty(&self) -> bool {
        !self
            .children()
            .any(|child| child.value().is_element() || child.value().is_text())
    }

    fn is_root(&self) -> bool {
        self.parent()
            .map_or(false, |parent| parent.value().is_document())
    }
}
pub struct ShieldedElmRef<'a>(&'a ElementRef<'a>);

impl<'a> ElementRef<'a>{
    pub fn shielded(&self) -> ShieldedElmRef{
        ShieldedElmRef(&self)
    }
}

impl<'a> ShieldedElmRef<'a>{
    fn has_class(&self, name: &LocalName, case_sensitive: bool) -> bool {
        self.0.has_class(name, map_b_to_cs(case_sensitive))
    }

    fn has_id(&self, id: &LocalName, case_sensitive: bool) -> bool{
        self.0.has_id(id, map_b_to_cs(case_sensitive))
    }
}

#[cfg(test)]
mod tests {
    use crate::html::Html;
    use crate::selector::Selector;
    use selectors::attr::CaseSensitivity;
    use selectors::Element;

    use super::map_b_to_cs;

    #[test]
    fn test_has_id() {
        use html5ever::LocalName;

        let html = "<p id='link_id_456'>hey there</p>";
        let fragment = Html::parse_fragment(html);
        let sel = Selector::parse("p").unwrap();

        let element = fragment.select(&sel).next().unwrap();
        assert_eq!(
            true,
            element.has_id(
                &LocalName::from("link_id_456"),
                map_b_to_cs(true)
            )
        );

        let html = "<p>hey there</p>";
        let fragment = Html::parse_fragment(html);
        let element = fragment.select(&sel).next().unwrap();
        assert_eq!(
            false,
            element.has_id(
                &LocalName::from("any_link_id"),
                map_b_to_cs(true)
            )
        );
    }

    #[test]
    fn test_is_link() {
        let html = "<link href='https://www.example.com'>";
        let fragment = Html::parse_fragment(html);
        let sel = Selector::parse("link").unwrap();
        let element = fragment.select(&sel).next().unwrap();
        assert_eq!(true, element.is_link());

        let html = "<p>hey there</p>";
        let fragment = Html::parse_fragment(html);
        let sel = Selector::parse("p").unwrap();
        let element = fragment.select(&sel).next().unwrap();
        assert_eq!(false, element.is_link());
    }

    #[test]
    fn test_has_class() {
        use html5ever::LocalName;
        let html = "<p class='my_class'>hey there</p>";
        let fragment = Html::parse_fragment(html);
        let sel = Selector::parse("p").unwrap();
        let element = fragment.select(&sel).next().unwrap();
        assert_eq!(
            true,
            element.has_class(&LocalName::from("my_class"), map_b_to_cs(true))
        );

        let html = "<p>hey there</p>";
        let fragment = Html::parse_fragment(html);
        let sel = Selector::parse("p").unwrap();
        let element = fragment.select(&sel).next().unwrap();
        assert_eq!(
            false,
            element.has_class(&LocalName::from("my_class"), map_b_to_cs(true))
        );
    }
}
