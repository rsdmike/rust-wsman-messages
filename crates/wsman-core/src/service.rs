use crate::client::Client;
use crate::envelope::{build_enumerate, build_get, build_pull, build_put};
use crate::error::WsmanError;
use crate::response::EnumerateResponse;
use crate::schema::Namespace;
use crate::selector::Selector;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

/// Associates a WS-MAN class with its three class-specific response shapes.
pub trait ResponseBody {
    const CLASS: &'static str;
    const NAMESPACE: Namespace;
    type Get: DeserializeOwned;
    type Pull: DeserializeOwned;
    type Put: DeserializeOwned;

    fn resource_uri() -> String {
        Self::NAMESPACE.resource_uri(Self::CLASS)
    }
}

pub struct WsmanService<T: ResponseBody> {
    client: Client,
    _marker: PhantomData<T>,
}

impl<T: ResponseBody> WsmanService<T> {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            _marker: PhantomData,
        }
    }

    pub async fn get(&self) -> Result<T::Get, WsmanError> {
        self.get_with_selectors(&[]).await
    }

    pub async fn get_by_instance_id(&self, id: &str) -> Result<T::Get, WsmanError> {
        self.get_with_selectors(&[Selector::new("InstanceID", id)])
            .await
    }

    pub async fn get_by_name(&self, name: &str) -> Result<T::Get, WsmanError> {
        self.get_with_selectors(&[Selector::new("Name", name)])
            .await
    }

    async fn get_with_selectors(&self, selectors: &[Selector]) -> Result<T::Get, WsmanError> {
        let uri = T::resource_uri();
        let id = self.client.next_message_id();
        let xml = build_get(&uri, selectors, id, None);
        let response = self.client.execute(&xml).await?;
        parse_body(&response)
    }

    pub async fn enumerate(&self) -> Result<EnumerateResponse, WsmanError> {
        let uri = T::resource_uri();
        let id = self.client.next_message_id();
        let xml = build_enumerate(&uri, &[], id, None);
        let response = self.client.execute(&xml).await?;
        EnumerateResponse::from_envelope(&response)
    }

    pub async fn pull(&self, context: &str) -> Result<T::Pull, WsmanError> {
        let uri = T::resource_uri();
        let id = self.client.next_message_id();
        let xml = build_pull(&uri, context, &[], id, None);
        let response = self.client.execute(&xml).await?;
        parse_body(&response)
    }

    pub async fn put<R: serde::Serialize>(&self, request: R) -> Result<T::Put, WsmanError> {
        let uri = T::resource_uri();
        let id = self.client.next_message_id();
        let body_inner = quick_xml::se::to_string(&request).map_err(|e| {
            WsmanError::BuilderMisuse(Box::leak(format!("serialize: {e}").into_boxed_str()))
        })?;
        let xml = build_put(&uri, &body_inner, &[], id, None);
        let response = self.client.execute(&xml).await?;
        parse_body(&response)
    }
}

/// Extracts the envelope and deserializes into T. T is expected to declare
/// the `<Envelope>` / `<Body>` / class-specific-body wrapper shape via serde
/// renames; see wsman-amt crate for examples.
fn parse_body<T: DeserializeOwned>(envelope_xml: &str) -> Result<T, WsmanError> {
    let de = quick_xml::de::from_str::<T>(envelope_xml)?;
    Ok(de)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::Namespace;

    // Fake class used only to exercise the generic.
    struct FakeClass;
    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct FakeGet {
        #[serde(rename = "HostName")]
        #[allow(dead_code)]
        host_name: String,
    }
    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct FakePull {
        #[serde(rename = "Items")]
        #[allow(dead_code)]
        items: Vec<FakeGet>,
    }
    impl ResponseBody for FakeClass {
        const CLASS: &'static str = "AMT_Fake";
        const NAMESPACE: Namespace = Namespace::Amt;
        type Get = FakeGet;
        type Pull = FakePull;
        type Put = ();
    }

    #[test]
    fn resource_uri_is_derived_from_class() {
        assert_eq!(
            <FakeClass as ResponseBody>::resource_uri(),
            "http://intel.com/wbem/wscim/1/amt-schema/1/AMT_Fake"
        );
    }
}
