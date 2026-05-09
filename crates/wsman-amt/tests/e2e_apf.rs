//! End-to-end test: drives AMT_GeneralSettings::Get all the way from
//! wsman-amt through wsman-core and apf, using a scripted FakeHeci
//! that impersonates the ME firmware. Proves the crate integration.

use wsman_amt::general::Settings;
use apf::apf_transport::ApfTransport;
use apf::error::HeciError;
use apf::message::{
    APF_CHANNEL_CLOSE, APF_CHANNEL_DATA, APF_CHANNEL_OPEN, APF_CHANNEL_OPEN_CONFIRMATION,
    APF_CHANNEL_WINDOW_ADJUST, APF_GLOBAL_REQUEST, APF_PROTOCOLVERSION, APF_REQUEST_SUCCESS,
    APF_SERVICE_ACCEPT, APF_SERVICE_PFWD, APF_SERVICE_REQUEST, LME_RX_WINDOW_SIZE, write_be32,
};
use apf::session::ApfSession;
use apf::transport::{HeciTransport, NoHooks};
use wsman_core::client::{Client, Credentials};

const HOST: u8 = 0x07;
const ME: u8 = 0x11;

#[allow(dead_code)]
enum Event {
    ExpectSendStartsWith { me: u8, host: u8, prefix: Vec<u8> },
    ExpectSend { me: u8, host: u8, data: Vec<u8> },
    ReturnRecv { me: u8, host: u8, data: Vec<u8> },
}

struct ScriptedHeci {
    script: Vec<Event>,
}
impl HeciTransport for ScriptedHeci {
    fn send(&mut self, me: u8, host: u8, data: &[u8]) -> Result<(), HeciError> {
        let ev = self.script.remove(0);
        match ev {
            Event::ExpectSend {
                me: em,
                host: eh,
                data: ed,
            } => {
                assert_eq!((me, host), (em, eh));
                assert_eq!(data, ed.as_slice());
                Ok(())
            }
            Event::ExpectSendStartsWith {
                me: em,
                host: eh,
                prefix,
            } => {
                assert_eq!((me, host), (em, eh));
                assert!(data.starts_with(&prefix), "data prefix mismatch");
                Ok(())
            }
            _ => panic!("send called out of order"),
        }
    }
    fn recv(&mut self, buf: &mut [u8]) -> Result<(usize, u8, u8), HeciError> {
        let ev = self.script.remove(0);
        match ev {
            Event::ReturnRecv { me, host, data } => {
                buf[..data.len()].copy_from_slice(&data);
                Ok((data.len(), me, host))
            }
            _ => panic!("recv called out of order"),
        }
    }
    fn close(&mut self) {}
}

fn be32(v: u32) -> [u8; 4] {
    v.to_be_bytes()
}

fn proto_version(major: u32, minor: u32) -> Vec<u8> {
    let mut v = vec![0u8; 93];
    v[0] = APF_PROTOCOLVERSION;
    write_be32(&mut v[1..5], major);
    write_be32(&mut v[5..9], minor);
    v
}

fn service_request(name: &[u8]) -> Vec<u8> {
    let mut v = vec![0u8; 5 + name.len()];
    v[0] = APF_SERVICE_REQUEST;
    write_be32(&mut v[1..5], name.len() as u32);
    v[5..5 + name.len()].copy_from_slice(name);
    v
}

fn service_accept(name: &[u8]) -> Vec<u8> {
    let mut v = vec![0u8; 5 + name.len()];
    v[0] = APF_SERVICE_ACCEPT;
    write_be32(&mut v[1..5], name.len() as u32);
    v[5..5 + name.len()].copy_from_slice(name);
    v
}

fn global_request_fwd(port: u32) -> Vec<u8> {
    let name = b"tcpip-forward";
    let addr = b"0.0.0.0";
    let mut v = vec![APF_GLOBAL_REQUEST];
    v.extend_from_slice(&be32(name.len() as u32));
    v.extend_from_slice(name);
    v.push(1); // want_reply
    v.extend_from_slice(&be32(addr.len() as u32));
    v.extend_from_slice(addr);
    v.extend_from_slice(&be32(port));
    v
}

fn request_success(port: u32) -> Vec<u8> {
    let mut v = vec![0u8; 5];
    v[0] = APF_REQUEST_SUCCESS;
    write_be32(&mut v[1..5], port);
    v
}

fn channel_open_bytes(sender: u32) -> Vec<u8> {
    let mut v = vec![0u8; 66];
    let mut i = 0;
    v[i] = APF_CHANNEL_OPEN;
    i += 1;
    write_be32(&mut v[i..i + 4], 15);
    i += 4;
    v[i..i + 15].copy_from_slice(b"forwarded-tcpip");
    i += 15;
    write_be32(&mut v[i..i + 4], sender);
    i += 4;
    write_be32(&mut v[i..i + 4], LME_RX_WINDOW_SIZE);
    i += 4;
    write_be32(&mut v[i..i + 4], 0xFFFF_FFFF);
    i += 4;
    write_be32(&mut v[i..i + 4], 9);
    i += 4;
    v[i..i + 9].copy_from_slice(b"127.0.0.1");
    i += 9;
    write_be32(&mut v[i..i + 4], 16992);
    i += 4;
    write_be32(&mut v[i..i + 4], 9);
    i += 4;
    v[i..i + 9].copy_from_slice(b"127.0.0.1");
    i += 9;
    write_be32(&mut v[i..i + 4], 16992);
    v
}

fn channel_open_confirmation(recip: u32, sender: u32, window: u32) -> Vec<u8> {
    let mut v = vec![0u8; 17];
    v[0] = APF_CHANNEL_OPEN_CONFIRMATION;
    write_be32(&mut v[1..5], recip);
    write_be32(&mut v[5..9], sender);
    write_be32(&mut v[9..13], window);
    v
}

fn channel_data(recip: u32, data: &[u8]) -> Vec<u8> {
    let mut v = vec![0u8; 9 + data.len()];
    v[0] = APF_CHANNEL_DATA;
    write_be32(&mut v[1..5], recip);
    write_be32(&mut v[5..9], data.len() as u32);
    v[9..9 + data.len()].copy_from_slice(data);
    v
}

fn window_adjust(recip: u32, add: u32) -> Vec<u8> {
    let mut v = vec![0u8; 9];
    v[0] = APF_CHANNEL_WINDOW_ADJUST;
    write_be32(&mut v[1..5], recip);
    write_be32(&mut v[5..9], add);
    v
}

fn channel_close(recip: u32) -> Vec<u8> {
    let mut v = vec![0u8; 5];
    v[0] = APF_CHANNEL_CLOSE;
    write_be32(&mut v[1..5], recip);
    v
}

#[test]
fn e2e_get_general_settings_through_full_stack() {
    let body = b"<a:Envelope xmlns:a=\"http://www.w3.org/2003/05/soap-envelope\" \
xmlns:g=\"http://intel.com/wbem/wscim/1/amt-schema/1/AMT_GeneralSettings\">\
<a:Body><g:AMT_GeneralSettings>\
<g:DigestRealm>Digest:ABC</g:DigestRealm>\
<g:InstanceID>Intel(r) AMT</g:InstanceID>\
<g:HostName>uefi-dev</g:HostName>\
<g:DomainName>lab</g:DomainName>\
<g:NetworkInterfaceEnabled>true</g:NetworkInterfaceEnabled>\
</g:AMT_GeneralSettings></a:Body></a:Envelope>";
    let http_response = {
        let mut v =
            format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len()).into_bytes();
        v.extend_from_slice(body);
        v
    };

    let script = vec![
        // Handshake
        Event::ExpectSend {
            me: ME,
            host: HOST,
            data: proto_version(1, 0),
        },
        Event::ReturnRecv {
            me: ME,
            host: HOST,
            data: proto_version(4, 0),
        },
        Event::ReturnRecv {
            me: ME,
            host: HOST,
            data: service_request(APF_SERVICE_PFWD),
        },
        Event::ExpectSend {
            me: ME,
            host: HOST,
            data: service_accept(APF_SERVICE_PFWD),
        },
        Event::ReturnRecv {
            me: ME,
            host: HOST,
            data: global_request_fwd(16992),
        },
        Event::ExpectSend {
            me: ME,
            host: HOST,
            data: request_success(16992),
        },
        // Channel open
        Event::ExpectSend {
            me: ME,
            host: HOST,
            data: channel_open_bytes(1),
        },
        Event::ReturnRecv {
            me: ME,
            host: HOST,
            data: channel_open_confirmation(1, 50, 8192),
        },
        // HTTP round-trip — match only the prefix; Content-Length depends on envelope length.
        Event::ExpectSendStartsWith {
            me: ME,
            host: HOST,
            prefix: {
                let mut v = vec![APF_CHANNEL_DATA];
                v.extend_from_slice(&be32(50));
                v
            },
        },
        Event::ReturnRecv {
            me: ME,
            host: HOST,
            data: channel_data(1, &http_response),
        },
        Event::ExpectSend {
            me: ME,
            host: HOST,
            data: window_adjust(50, http_response.len() as u32),
        },
        Event::ReturnRecv {
            me: ME,
            host: HOST,
            data: channel_close(1),
        },
        Event::ExpectSend {
            me: ME,
            host: HOST,
            data: channel_close(50),
        },
    ];

    let heci = ScriptedHeci { script };
    let mut apf = ApfSession::new(heci, NoHooks, ME, HOST);
    apf.handshake().unwrap();
    apf.channel_open().unwrap();

    let transport = ApfTransport::new(apf);
    let mut client = Client::new(transport, Credentials::digest("admin", "hunter2"));

    let gs = Settings::new(&mut client).get().unwrap();
    assert_eq!(gs.digest_realm, "Digest:ABC");
    assert_eq!(gs.host_name, "uefi-dev");
    assert_eq!(gs.domain_name, "lab");
    assert!(gs.network_interface_enabled);
}
