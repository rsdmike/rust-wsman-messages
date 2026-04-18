use crate::error::HeciError;

/// Byte-level transport over HECI. Implementations: UEFI MMIO, Windows
/// `DeviceIoControl`, Linux `/dev/mei*`, or a test fake.
pub trait HeciTransport {
    /// Send a single HECI message. `me_addr` and `host_addr` identify the
    /// connected client pair; on Windows/Linux the driver ignores them.
    fn send(&mut self, me_addr: u8, host_addr: u8, data: &[u8]) -> Result<(), HeciError>;

    /// Receive a single HECI message into `buf`. Returns `(len, me_addr,
    /// host_addr)`. `me=0, host=0` marks an HBM (bus-management) frame.
    fn recv(&mut self, buf: &mut [u8]) -> Result<(usize, u8, u8), HeciError>;

    fn close(&mut self);

    /// Tear down and re-establish the HECI client connection. Called from
    /// `HeciHooks::reconnect_heci` when ME has dropped the session and the
    /// transport needs to fully re-handshake before retrying. Default:
    /// no-op (Windows/Linux drivers handle reconnect transparently; only
    /// UEFI MMIO targets need to override).
    fn reset(&mut self) -> Result<(), HeciError> {
        Ok(())
    }
}

/// Target-specific escape hatches. Every method has a default no-op;
/// targets override only the ones they need.
pub trait HeciHooks {
    /// Called after sending an `APF_CHANNEL_OPEN`, before waiting for
    /// `APF_CHANNEL_OPEN_CONFIRMATION`. The UEFI target must perform an
    /// actual filesystem write here — see `amt-uefi-boot-app/src/lme/mod.rs`
    /// for the "load-bearing magic" comment that explains why.
    fn post_channel_open_send(&mut self) {}

    /// Called when `ApfSession` detects the ME requested disconnect (e.g.,
    /// HBM_CLIENT_DISCONNECT_REQ on a channel reopen). UEFI target tears
    /// down the HECI session and reconnects the LME client; other targets
    /// let the driver handle it and return `Ok(())`.
    fn reconnect_heci(&mut self, _heci: &mut dyn HeciTransport) -> Result<(), HeciError> {
        Ok(())
    }
}

/// Default hooks with all methods no-op. Use as the `H` generic when the
/// caller doesn't need to hook anything.
#[derive(Default, Debug, Clone, Copy)]
pub struct NoHooks;

impl HeciHooks for NoHooks {}
