use alloc::string::String;

#[derive(Debug, Clone)]
pub struct Selector<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

impl<'a> Selector<'a> {
    pub fn new(name: &'a str, value: &'a str) -> Self {
        Self { name, value }
    }

    /// Renders the `<w:SelectorSet>...</w:SelectorSet>` fragment for the
    /// given selectors. Returns an empty string when `sel` is empty so the
    /// envelope builder can unconditionally splice it in.
    pub fn render_set(sel: &[Selector<'_>]) -> String {
        if sel.is_empty() {
            return String::new();
        }
        let mut s = String::from("<w:SelectorSet>");
        for Selector { name, value } in sel {
            s.push_str("<w:Selector Name=\"");
            s.push_str(name);
            s.push_str("\">");
            s.push_str(value);
            s.push_str("</w:Selector>");
        }
        s.push_str("</w:SelectorSet>");
        s
    }
}
