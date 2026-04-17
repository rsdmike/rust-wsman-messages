#[derive(Debug, Clone)]
pub struct Selector {
    pub name: String,
    pub value: String,
}

impl Selector {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    pub fn render_set(selectors: &[Selector]) -> String {
        if selectors.is_empty() {
            return String::new();
        }
        let mut s = String::from("<w:SelectorSet>");
        for sel in selectors {
            // NOTE: values are injected verbatim; caller must pre-escape if they
            // contain XML-special chars. Matches Go's wsman.go:122 behavior.
            s.push_str(&format!(
                r#"<w:Selector Name="{}">{}</w:Selector>"#,
                sel.name, sel.value
            ));
        }
        s.push_str("</w:SelectorSet>");
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_set_xml_single() {
        let selectors = [Selector::new(
            "InstanceID",
            "Intel(r) AMT: General Settings",
        )];
        assert_eq!(
            Selector::render_set(&selectors),
            r#"<w:SelectorSet><w:Selector Name="InstanceID">Intel(r) AMT: General Settings</w:Selector></w:SelectorSet>"#
        );
    }

    #[test]
    fn selector_set_xml_empty_is_empty_string() {
        let selectors: [Selector; 0] = [];
        assert_eq!(Selector::render_set(&selectors), "");
    }

    #[test]
    fn selector_set_xml_multiple_has_no_separator() {
        let selectors = [
            Selector::new("Name", "eth0"),
            Selector::new("InstanceID", "1"),
        ];
        assert_eq!(
            Selector::render_set(&selectors),
            r#"<w:SelectorSet><w:Selector Name="Name">eth0</w:Selector><w:Selector Name="InstanceID">1</w:Selector></w:SelectorSet>"#
        );
    }
}
