// crates/wsman-amt/src/general/service.rs
use super::types::{GeneralSettings, GeneralSettingsRequest, GetResponse, PullResponseEnvelope};
use wsman_core::client::Client;
use wsman_core::error::WsmanError;
use wsman_core::response::EnumerateResponse;
use wsman_core::schema::Namespace;
use wsman_core::service::{ResponseBody, WsmanService};

pub struct GeneralSettingsClass;

impl ResponseBody for GeneralSettingsClass {
    const CLASS: &'static str = "AMT_GeneralSettings";
    const NAMESPACE: Namespace = Namespace::Amt;
    type Get = GetResponse;
    type Pull = PullResponseEnvelope;
    type Put = GetResponse; // AMT Put echoes the updated resource
}

/// Service wrapper for `AMT_GeneralSettings`. Named `Settings` (not
/// `GeneralSettings`) to avoid colliding with the DTO struct of the same
/// name in `types.rs`. Matches Go's `type Settings struct` in `settings.go:15`.
pub struct Settings {
    inner: WsmanService<GeneralSettingsClass>,
}

impl Settings {
    pub fn new(client: Client) -> Self {
        Self {
            inner: WsmanService::new(client),
        }
    }

    pub async fn get(&self) -> Result<GeneralSettings, WsmanError> {
        let env = self.inner.get().await?;
        Ok(env.body.settings)
    }

    pub async fn get_by_instance_id(&self, id: &str) -> Result<GeneralSettings, WsmanError> {
        let env = self.inner.get_by_instance_id(id).await?;
        Ok(env.body.settings)
    }

    pub async fn enumerate(&self) -> Result<EnumerateResponse, WsmanError> {
        self.inner.enumerate().await
    }

    pub async fn pull(&self, context: &str) -> Result<Vec<GeneralSettings>, WsmanError> {
        let env = self.inner.pull(context).await?;
        Ok(env.body.pull.items.map(|i| i.settings).unwrap_or_default())
    }

    pub async fn put(
        &self,
        mut request: GeneralSettingsRequest,
    ) -> Result<GeneralSettings, WsmanError> {
        request.xmlns_h = Namespace::Amt.resource_uri("AMT_GeneralSettings");
        let env = self.inner.put(request).await?;
        Ok(env.body.settings)
    }
}
