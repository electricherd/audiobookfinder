/// The noise protocol to be used
/// (http://noiseprotocol.org/)
use async_std::{
    io,
    task::{self, Context, Poll},
};
use futures::{future, prelude::*};
use libp2p::{
    identify::{Identify, IdentifyEvent},
    mdns::service::MdnsPeer,
    ping::{self, Ping, PingConfig, PingEvent},
    pnet::{PnetConfig, PreSharedKey},
    secio::SecioConfig,
    swarm::NetworkBehaviourEventProcess,
    yamux::Config as YamuxConfig,
    NetworkBehaviour, Swarm,
};
use libp2p_core::{
    either::EitherTransport,
    identity,
    transport::upgrade::{self, Version},
    Multiaddr, PeerId, StreamMuxer, Transport,
};
use libp2p_noise::{Keypair, NoiseConfig, X25519Spec};
use libp2p_tcp::TcpConfig;

use std::{error::Error, time::Duration};

#[derive(NetworkBehaviour)]
pub struct CustomBehaviour {
    pub identify: Identify,
    pub ping: Ping,
}
impl NetworkBehaviourEventProcess<IdentifyEvent> for CustomBehaviour {
    // Called when `identify` produces an event.
    fn inject_event(&mut self, event: IdentifyEvent) {
        trace!("identify: {:?}", event);
    }
}
impl NetworkBehaviourEventProcess<PingEvent> for CustomBehaviour {
    // Called when `ping` produces an event.
    fn inject_event(&mut self, event: PingEvent) {
        use ping::handler::{PingFailure, PingSuccess};
        match event {
            PingEvent {
                peer,
                result: Result::Ok(PingSuccess::Ping { rtt }),
            } => {
                trace!(
                    "ping: rtt to {} is {} ms",
                    peer.to_base58(),
                    rtt.as_millis()
                );
            }
            PingEvent {
                peer,
                result: Result::Ok(PingSuccess::Pong),
            } => {
                trace!("ping: pong from {}", peer.to_base58());
            }
            PingEvent {
                peer,
                result: Result::Err(PingFailure::Timeout),
            } => {
                trace!("ping: timeout to {}", peer.to_base58());
            }
            PingEvent {
                peer,
                result: Result::Err(PingFailure::Other { error }),
            } => {
                warn!("ping: failure with {}: {}", peer.to_base58(), error);
            }
        }
    }
}

// let local_key = identity::Keypair::generate_ed25519();
pub async fn init(
    psk: PreSharedKey,
    found_peer: &MdnsPeer,
    local_key: &identity::Keypair,
) -> Result<(), Box<dyn Error>> {
    let local_peer_id = PeerId::from(local_key.public());

    // get the transporter
    let transport = build_noise_transport(local_key, Some(psk));

    let mut swarm = {
        let behaviour = CustomBehaviour {
            identify: Identify::new("adbf/0.1.0".into(), "adbf-agent".into(), local_key.public()),
            ping: Ping::new(PingConfig::new()),
        };
        Swarm::new(transport, behaviour, local_peer_id.clone())
    };

    info!("hthaaaatt........................");
    for multi_addr in found_peer.addresses().to_owned() {
        info!("Dialed {:?}", &multi_addr);
        Swarm::dial_addr(&mut swarm, multi_addr)?;
    }

    // needed???
    // Listen on all interfaces and whatever port the OS assigns
    Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse()?)?;

    let mut listening = false;
    task::spawn(futures::future::poll_fn(move |cx: &mut Context| {
        loop {
            match swarm.poll_next_unpin(cx) {
                Poll::Ready(Some(event)) => {
                    info!("{:?}", event);
                }
                Poll::Ready(None) => return Poll::Ready(Ok::<(), ()>(())),
                Poll::Pending => {
                    if !listening {
                        for addr in Swarm::listeners(&swarm) {
                            info!("Address {} - {}", addr, local_peer_id);
                            listening = true;
                        }
                    }
                    break;
                }
            }
        }
        Poll::Pending
    }));
    Ok(())
}

pub fn build_noise_transport(
    key_pair: &identity::Keypair,
    psk: Option<PreSharedKey>,
) -> impl Transport<
    Output = (
        PeerId,
        impl StreamMuxer<
                OutboundSubstream = impl Send,
                Substream = impl Send,
                Error = impl Into<io::Error>,
            > + Send
            + Sync,
    ),
    Error = impl Error + Send,
    Listener = impl Send,
    Dial = impl Send,
    ListenerUpgrade = impl Send,
> + Clone {
    let dh_keys = Keypair::<X25519Spec>::new()
        .into_authentic(&key_pair)
        .unwrap();
    let noise_config = NoiseConfig::xx(dh_keys).into_authenticated();
    let yamux_config = YamuxConfig::default();

    let base_transport = TcpConfig::new().nodelay(true);
    let maybe_encrypted = match psk {
        Some(psk) => EitherTransport::Left(
            base_transport.and_then(move |socket, _| PnetConfig::new(psk).handshake(socket)),
        ),
        None => EitherTransport::Right(base_transport),
    };
    maybe_encrypted
        .upgrade(upgrade::Version::V1)
        .authenticate(noise_config)
        .multiplex(yamux_config)
        .timeout(Duration::from_secs(20))
}
