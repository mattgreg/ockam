use crate::nodes::service::Alias;
use ockam::identity::Identifier;
use ockam::identity::{SecureChannel, SecureChannelListener};
use ockam::remote::RemoteForwarderInfo;
use ockam_core::compat::collections::BTreeMap;
use ockam_core::{Address, Route};
use std::fmt::Display;
use std::net::SocketAddr;

#[derive(Default)]
pub(crate) struct SecureChannelRegistry {
    channels: Vec<SecureChannelInfo>,
}

impl SecureChannelRegistry {
    pub fn get_by_addr(&self, addr: &Address) -> Option<&SecureChannelInfo> {
        self.channels
            .iter()
            .find(|&x| x.sc.encryptor_address() == addr)
    }

    pub fn insert(
        &mut self,
        route: Route,
        sc: SecureChannel,
        authorized_identifiers: Option<Vec<Identifier>>,
    ) {
        self.channels
            .push(SecureChannelInfo::new(route, sc, authorized_identifiers))
    }

    pub fn remove_by_addr(&mut self, addr: &Address) {
        self.channels.retain(|x| x.sc().encryptor_address() != addr)
    }

    pub fn list(&self) -> &[SecureChannelInfo] {
        &self.channels
    }
}

#[derive(Clone)]
pub struct SecureChannelInfo {
    // Target route of the channel
    route: Route,
    sc: SecureChannel,
    authorized_identifiers: Option<Vec<Identifier>>,
}

impl SecureChannelInfo {
    pub fn new(
        route: Route,
        sc: SecureChannel,
        authorized_identifiers: Option<Vec<Identifier>>,
    ) -> Self {
        Self {
            route,
            sc,
            authorized_identifiers,
        }
    }

    pub fn route(&self) -> &Route {
        &self.route
    }

    pub fn sc(&self) -> &SecureChannel {
        &self.sc
    }

    pub fn authorized_identifiers(&self) -> Option<&Vec<Identifier>> {
        self.authorized_identifiers.as_ref()
    }
}

#[derive(Clone)]
pub(crate) struct SecureChannelListenerInfo {
    listener: SecureChannelListener,
}

impl SecureChannelListenerInfo {
    pub fn new(listener: SecureChannelListener) -> Self {
        Self { listener }
    }

    pub fn listener(&self) -> &SecureChannelListener {
        &self.listener
    }
}

#[derive(Default)]
pub(crate) struct IdentityServiceInfo {}

#[derive(Default)]
pub(crate) struct AuthenticatedServiceInfo {}

#[derive(Default)]
pub(crate) struct OktaIdentityProviderServiceInfo {}

#[derive(Default)]
pub(crate) struct UppercaseServiceInfo {}

#[derive(Default)]
pub(crate) struct EchoerServiceInfo {}

#[derive(Default)]
pub(crate) struct HopServiceInfo {}

#[derive(Default)]
pub(crate) struct VerifierServiceInfo {}

#[derive(Default)]
pub(crate) struct CredentialsServiceInfo {}

#[derive(Eq, PartialEq)]
pub(crate) enum KafkaServiceKind {
    Consumer,
    Producer,
    Outlet,
    Direct,
}

impl Display for KafkaServiceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KafkaServiceKind::Consumer => write!(f, "consumer"),
            KafkaServiceKind::Producer => write!(f, "producer"),
            KafkaServiceKind::Outlet => write!(f, "outlet"),
            KafkaServiceKind::Direct => write!(f, "direct"),
        }
    }
}

pub(crate) struct KafkaServiceInfo {
    kind: KafkaServiceKind,
}

impl KafkaServiceInfo {
    pub fn new(kind: KafkaServiceKind) -> Self {
        Self { kind }
    }

    pub fn kind(&self) -> &KafkaServiceKind {
        &self.kind
    }
}

#[derive(Clone)]
pub(crate) struct InletInfo {
    pub(crate) bind_addr: String,
    pub(crate) worker_addr: Address,
    pub(crate) outlet_route: Route,
}

impl InletInfo {
    pub(crate) fn new(
        bind_addr: &str,
        worker_addr: Option<&Address>,
        outlet_route: &Route,
    ) -> Self {
        let worker_addr = match worker_addr {
            Some(addr) => addr.clone(),
            None => Address::from_string(""),
        };
        Self {
            bind_addr: bind_addr.to_owned(),
            worker_addr,
            outlet_route: outlet_route.to_owned(),
        }
    }
}

#[derive(Clone)]
pub struct OutletInfo {
    pub(crate) socket_addr: SocketAddr,
    pub(crate) worker_addr: Address,
}

impl OutletInfo {
    pub(crate) fn new(socket_addr: &SocketAddr, worker_addr: Option<&Address>) -> Self {
        let worker_addr = match worker_addr {
            Some(addr) => addr.clone(),
            None => Address::from_string(""),
        };
        Self {
            socket_addr: *socket_addr,
            worker_addr,
        }
    }
}

#[derive(Default)]
pub(crate) struct Registry {
    pub(crate) secure_channels: SecureChannelRegistry,
    pub(crate) secure_channel_listeners: BTreeMap<Address, SecureChannelListenerInfo>,
    pub(crate) authenticated_services: BTreeMap<Address, AuthenticatedServiceInfo>,
    pub(crate) uppercase_services: BTreeMap<Address, UppercaseServiceInfo>,
    pub(crate) echoer_services: BTreeMap<Address, EchoerServiceInfo>,
    pub(crate) kafka_services: BTreeMap<Address, KafkaServiceInfo>,
    pub(crate) hop_services: BTreeMap<Address, HopServiceInfo>,
    pub(crate) credentials_services: BTreeMap<Address, CredentialsServiceInfo>,

    // FIXME: wow this is a terrible way to store data
    pub(crate) forwarders: BTreeMap<String, RemoteForwarderInfo>,
    pub(crate) inlets: BTreeMap<Alias, InletInfo>,
    pub(crate) outlets: BTreeMap<Alias, OutletInfo>,
}
