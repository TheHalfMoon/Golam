#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

const MAX_AUTHORITY_BYTES: usize = 253;
const MAX_PROTOCOL_BYTES: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectiveNetworkClass {
    External,
    Private,
    LinkLocal,
    Loopback,
}

impl EffectiveNetworkClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::External => "external",
            Self::Private => "private",
            Self::LinkLocal => "link_local",
            Self::Loopback => "loopback",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectiveDestination {
    authority: String,
    address: IpAddr,
    protocol: String,
    port: u16,
    class: EffectiveNetworkClass,
    resource: String,
}

impl EffectiveDestination {
    pub fn new(
        authority: &str,
        address: IpAddr,
        protocol: &str,
        port: u16,
    ) -> Result<Self, EffectiveDestinationError> {
        let authority = normalize_authority(authority)?;
        let protocol = normalize_protocol(protocol)?;
        if port == 0 {
            return Err(EffectiveDestinationError::InvalidPort);
        }
        let class = classify_address(address)?;
        let resource = match address {
            IpAddr::V4(address) => format!("{protocol}://{address}:{port}"),
            IpAddr::V6(address) => format!("{protocol}://[{address}]:{port}"),
        };
        Ok(Self {
            authority,
            address,
            protocol,
            port,
            class,
            resource,
        })
    }

    pub fn authority(&self) -> &str {
        &self.authority
    }

    pub const fn address(&self) -> IpAddr {
        self.address
    }

    pub fn protocol(&self) -> &str {
        &self.protocol
    }

    pub const fn port(&self) -> u16 {
        self.port
    }

    pub const fn class(&self) -> EffectiveNetworkClass {
        self.class
    }

    pub fn resource(&self) -> &str {
        &self.resource
    }

    pub fn protocol_port(&self) -> String {
        format!("{}:{}", self.protocol, self.port)
    }

    pub fn decision_context(
        &self,
        permit_id: [u8; 16],
        authorized_destination_scope: &str,
    ) -> String {
        let scope_hash = blake3::hash(authorized_destination_scope.as_bytes());
        format!(
            "golam-egress-effective-v1|permit={}|scope={}|authority={}|resource={}|class={}",
            hex_bytes(&permit_id),
            hex_bytes(scope_hash.as_bytes()),
            self.authority,
            self.resource,
            self.class.as_str(),
        )
    }

    pub fn decision_context_hash(
        &self,
        permit_id: [u8; 16],
        authorized_destination_scope: &str,
    ) -> [u8; 32] {
        *blake3::hash(
            self.decision_context(permit_id, authorized_destination_scope)
                .as_bytes(),
        )
        .as_bytes()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectiveDestinationError {
    InvalidAuthority,
    InvalidProtocol,
    InvalidPort,
    UnsupportedAddressClass,
}

impl fmt::Display for EffectiveDestinationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAuthority => f.write_str("effective destination authority is invalid"),
            Self::InvalidProtocol => f.write_str("effective destination protocol is invalid"),
            Self::InvalidPort => f.write_str("effective destination port is invalid"),
            Self::UnsupportedAddressClass => {
                f.write_str("effective destination address class is unsupported")
            }
        }
    }
}

impl Error for EffectiveDestinationError {}

fn normalize_authority(value: &str) -> Result<String, EffectiveDestinationError> {
    if value.is_empty()
        || value.len() > MAX_AUTHORITY_BYTES
        || value.trim() != value
        || !value.is_ascii()
        || value.starts_with('.')
        || value.ends_with('.')
        || value.contains("..")
        || value.bytes().any(|byte| {
            !(byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
        })
    {
        return Err(EffectiveDestinationError::InvalidAuthority);
    }
    Ok(value.to_ascii_lowercase())
}

fn normalize_protocol(value: &str) -> Result<String, EffectiveDestinationError> {
    if value.is_empty()
        || value.len() > MAX_PROTOCOL_BYTES
        || value.trim() != value
        || !value.is_ascii()
    {
        return Err(EffectiveDestinationError::InvalidProtocol);
    }
    let normalized = value.to_ascii_lowercase();
    if normalized.bytes().any(|byte| {
        !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    }) {
        return Err(EffectiveDestinationError::InvalidProtocol);
    }
    Ok(normalized)
}

fn classify_address(address: IpAddr) -> Result<EffectiveNetworkClass, EffectiveDestinationError> {
    if address.is_unspecified() || address.is_multicast() {
        return Err(EffectiveDestinationError::UnsupportedAddressClass);
    }
    match address {
        IpAddr::V4(address) => classify_v4(address),
        IpAddr::V6(address) => Ok(classify_v6(address)),
    }
}

fn classify_v4(address: Ipv4Addr) -> Result<EffectiveNetworkClass, EffectiveDestinationError> {
    if address == Ipv4Addr::BROADCAST {
        return Err(EffectiveDestinationError::UnsupportedAddressClass);
    }
    if address.is_loopback() {
        return Ok(EffectiveNetworkClass::Loopback);
    }
    if address.is_link_local() {
        return Ok(EffectiveNetworkClass::LinkLocal);
    }
    if address.is_private() {
        return Ok(EffectiveNetworkClass::Private);
    }
    Ok(EffectiveNetworkClass::External)
}

fn classify_v6(address: Ipv6Addr) -> EffectiveNetworkClass {
    if address.is_loopback() {
        return EffectiveNetworkClass::Loopback;
    }
    if address.is_unicast_link_local() {
        return EffectiveNetworkClass::LinkLocal;
    }
    if address.is_unique_local() {
        return EffectiveNetworkClass::Private;
    }
    EffectiveNetworkClass::External
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(char::from(HEX[(byte >> 4) as usize]));
        value.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_identity_binds_authority_address_protocol_port_class_and_permit_scope() {
        let first = EffectiveDestination::new(
            "Example.Invalid",
            "203.0.113.10".parse().unwrap(),
            "HTTPS",
            443,
        )
        .unwrap();
        assert_eq!(first.authority(), "example.invalid");
        assert_eq!(first.resource(), "https://203.0.113.10:443");
        assert_eq!(first.protocol_port(), "https:443");
        assert_eq!(first.class(), EffectiveNetworkClass::External);

        let same = EffectiveDestination::new(
            "example.invalid",
            "203.0.113.10".parse().unwrap(),
            "https",
            443,
        )
        .unwrap();
        assert_eq!(
            first.decision_context_hash([7; 16], "https://example.invalid"),
            same.decision_context_hash([7; 16], "https://example.invalid")
        );

        let redirect = EffectiveDestination::new(
            "other.invalid",
            "203.0.113.10".parse().unwrap(),
            "https",
            443,
        )
        .unwrap();
        assert_ne!(
            first.decision_context_hash([7; 16], "https://example.invalid"),
            redirect.decision_context_hash([7; 16], "https://example.invalid")
        );

        let rebound = EffectiveDestination::new(
            "example.invalid",
            "203.0.113.11".parse().unwrap(),
            "https",
            443,
        )
        .unwrap();
        assert_ne!(first.resource(), rebound.resource());
        assert_ne!(
            first.decision_context_hash([7; 16], "https://example.invalid"),
            first.decision_context_hash([8; 16], "https://example.invalid")
        );
        assert_ne!(
            first.decision_context_hash([7; 16], "https://example.invalid"),
            first.decision_context_hash([7; 16], "https://other.invalid")
        );
    }

    #[test]
    fn sensitive_address_classes_are_explicit_and_unsupported_classes_deny() {
        assert_eq!(
            EffectiveDestination::new(
                "private.invalid",
                "10.0.0.7".parse().unwrap(),
                "https",
                443,
            )
            .unwrap()
            .class(),
            EffectiveNetworkClass::Private
        );
        assert_eq!(
            EffectiveDestination::new(
                "link.invalid",
                "169.254.10.20".parse().unwrap(),
                "https",
                443,
            )
            .unwrap()
            .class(),
            EffectiveNetworkClass::LinkLocal
        );
        assert_eq!(
            EffectiveDestination::new(
                "loopback.invalid",
                "127.0.0.1".parse().unwrap(),
                "https",
                443,
            )
            .unwrap()
            .class(),
            EffectiveNetworkClass::Loopback
        );
        assert!(matches!(
            EffectiveDestination::new(
                "invalid.invalid",
                "0.0.0.0".parse().unwrap(),
                "https",
                443,
            ),
            Err(EffectiveDestinationError::UnsupportedAddressClass)
        ));
    }
}
