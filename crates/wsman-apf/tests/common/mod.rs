#![allow(dead_code)]

use wsman_apf::error::HeciError;
use wsman_apf::transport::HeciTransport;

#[derive(Debug, Clone)]
pub enum Event {
    ExpectSend { me: u8, host: u8, data: Vec<u8> },
    ReturnRecv { me: u8, host: u8, data: Vec<u8> },
    ReturnRecvErr(HeciError),
    ExpectClose,
}

pub struct FakeHeci {
    script: Vec<Event>,
    idx: usize,
    closed: bool,
}

impl FakeHeci {
    pub fn new(script: Vec<Event>) -> Self {
        Self { script, idx: 0, closed: false }
    }
    pub fn exhausted(&self) -> bool { self.idx == self.script.len() }
}

impl HeciTransport for FakeHeci {
    fn send(&mut self, me: u8, host: u8, data: &[u8]) -> Result<(), HeciError> {
        let ev = self
            .script
            .get(self.idx)
            .cloned()
            .unwrap_or_else(|| panic!("send past end of script"));
        self.idx += 1;
        match ev {
            Event::ExpectSend { me: em, host: eh, data: ed } => {
                assert_eq!((me, host), (em, eh), "send addr mismatch");
                assert_eq!(data, ed.as_slice(), "send bytes mismatch");
                Ok(())
            }
            other => panic!("send called but script expected {other:?}"),
        }
    }
    fn recv(&mut self, buf: &mut [u8]) -> Result<(usize, u8, u8), HeciError> {
        let ev = self
            .script
            .get(self.idx)
            .cloned()
            .unwrap_or_else(|| panic!("recv past end of script"));
        self.idx += 1;
        match ev {
            Event::ReturnRecv { me, host, data } => {
                if buf.len() < data.len() {
                    return Err(HeciError::BufferTooSmall);
                }
                buf[..data.len()].copy_from_slice(&data);
                Ok((data.len(), me, host))
            }
            Event::ReturnRecvErr(e) => Err(e),
            other => panic!("recv called but script expected {other:?}"),
        }
    }
    fn close(&mut self) {
        let ev = self
            .script
            .get(self.idx)
            .cloned()
            .unwrap_or_else(|| panic!("close past end of script"));
        self.idx += 1;
        match ev {
            Event::ExpectClose => self.closed = true,
            other => panic!("close called but script expected {other:?}"),
        }
    }
}
