#![forbid(unsafe_code)]

use golam_core::ClientId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationRequest<'a> {
    pub principal: ClientId,
    pub action: &'a str,
    pub resource: &'a str,
    pub context: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationDecision {
    Allow,
    Deny,
}

pub trait AuthorizationPolicy {
    fn authorize(&self, request: &AuthorizationRequest<'_>) -> AuthorizationDecision;
}

#[derive(Debug)]
pub struct AuthorityToken {
    _private: (),
}

pub struct KernelApi<P> {
    policy: P,
}

impl<P: AuthorizationPolicy> KernelApi<P> {
    pub fn new(policy: P) -> Self {
        Self { policy }
    }

    pub fn authorize(
        &self,
        request: &AuthorizationRequest<'_>,
    ) -> Result<AuthorityToken, AuthorizationDecision> {
        match self.policy.authorize(request) {
            AuthorizationDecision::Allow => Ok(AuthorityToken { _private: () }),
            AuthorizationDecision::Deny => Err(AuthorizationDecision::Deny),
        }
    }
}

pub struct DenyByDefault;

impl AuthorizationPolicy for DenyByDefault {
    fn authorize(&self, _request: &AuthorizationRequest<'_>) -> AuthorizationDecision {
        AuthorizationDecision::Deny
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_policy_denies() {
        let kernel = KernelApi::new(DenyByDefault);
        let request = AuthorizationRequest {
            principal: ClientId(7),
            action: "session.write",
            resource: "session:1",
            context: "local",
        };
        assert_eq!(
            kernel.authorize(&request).unwrap_err(),
            AuthorizationDecision::Deny
        );
    }
}
