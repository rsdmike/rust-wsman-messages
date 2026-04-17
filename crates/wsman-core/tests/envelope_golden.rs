use pretty_assertions::assert_eq;
use wsman_core::envelope::{build_enumerate, build_get, build_pull, build_put};
use wsman_core::schema::Namespace;

const AMT_GENERAL: &str = "AMT_GeneralSettings";

fn load(name: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{name}"))
        .unwrap_or_else(|_| panic!("missing fixture: {name}"))
}

#[test]
fn get_matches_go() {
    let uri = Namespace::Amt.resource_uri(AMT_GENERAL);
    let actual = build_get(&uri, &[], 0, None);
    assert_eq!(actual, load("envelope_get.xml"));
}

#[test]
fn enumerate_matches_go() {
    let uri = Namespace::Amt.resource_uri(AMT_GENERAL);
    let actual = build_enumerate(&uri, &[], 0, None);
    assert_eq!(actual, load("envelope_enumerate.xml"));
}

#[test]
fn pull_matches_go() {
    let uri = Namespace::Amt.resource_uri(AMT_GENERAL);
    let actual = build_pull(&uri, "AC070000-0000-0000-0000-000000000000", &[], 0, None);
    assert_eq!(actual, load("envelope_pull.xml"));
}

#[test]
fn put_matches_go() {
    let uri = Namespace::Amt.resource_uri(AMT_GENERAL);
    let inner = r#"<h:AMT_GeneralSettings xmlns:h="http://intel.com/wbem/wscim/1/amt-schema/1/AMT_GeneralSettings"><h:HostName>mylaptop</h:HostName></h:AMT_GeneralSettings>"#;
    let actual = build_put(&uri, inner, &[], 0, None);
    assert_eq!(actual, load("envelope_put.xml"));
}
