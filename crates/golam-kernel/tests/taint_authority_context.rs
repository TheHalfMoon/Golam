#![forbid(unsafe_code)]

use golam_core::taint::{Provenanced, TaintLabel, TaintSet};
use golam_kernel::AuthorizationContext;

#[test]
fn authority_context_carries_monotonic_taint_without_overloading_scope_identity() {
    let web_context = Provenanced::source(
        AuthorizationContext::local("local-owner"),
        TaintSet::from_labels([TaintLabel::WebUntrusted]),
    );
    let channel_context = Provenanced::source(
        AuthorizationContext::local("local-owner"),
        TaintSet::from_labels([TaintLabel::ChannelUntrusted]),
    );

    let derived = Provenanced::derive(
        AuthorizationContext::local("local-owner"),
        [web_context.taint(), channel_context.taint()],
        TaintSet::from_labels([TaintLabel::ModelGenerated]),
    );

    assert_eq!(derived.value().scope, "local-owner");
    assert!(!derived.value().safety_denied);
    assert!(derived.taint().contains(TaintLabel::WebUntrusted));
    assert!(derived.taint().contains(TaintLabel::ChannelUntrusted));
    assert!(derived.taint().contains(TaintLabel::ModelGenerated));
}

#[test]
fn authority_context_taint_encoding_is_independent_of_source_order() {
    let web = TaintSet::from_labels([TaintLabel::WebUntrusted]);
    let mcp = TaintSet::from_labels([TaintLabel::McpUntrusted]);
    let generated = TaintSet::from_labels([TaintLabel::ModelGenerated]);

    let first = Provenanced::derive(
        AuthorizationContext::local("local-owner"),
        [web, mcp],
        generated,
    );
    let second = Provenanced::derive(
        AuthorizationContext::local("local-owner"),
        [mcp, web],
        generated,
    );

    assert_eq!(first.value(), second.value());
    assert_eq!(first.taint(), second.taint());
    assert_eq!(
        first.taint().canonical_bytes().unwrap(),
        second.taint().canonical_bytes().unwrap()
    );
}
