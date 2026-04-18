use wsman_core::selector::Selector;

#[test]
fn render_set_empty_returns_empty_string() {
    assert_eq!(Selector::render_set(&[]), "");
}

#[test]
fn render_set_single_selector() {
    let s = Selector::render_set(&[Selector::new("InstanceID", "Intel(r) AMT")]);
    assert_eq!(
        s,
        "<w:SelectorSet><w:Selector Name=\"InstanceID\">Intel(r) AMT</w:Selector></w:SelectorSet>"
    );
}

#[test]
fn render_set_multiple_selectors_preserves_order() {
    let s = Selector::render_set(&[
        Selector::new("Name", "val1"),
        Selector::new("Protocol", "HTTP"),
    ]);
    assert_eq!(
        s,
        "<w:SelectorSet>\
<w:Selector Name=\"Name\">val1</w:Selector>\
<w:Selector Name=\"Protocol\">HTTP</w:Selector>\
</w:SelectorSet>"
    );
}
