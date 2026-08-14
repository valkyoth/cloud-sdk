# Migrating Source Users To v0.89

v0.89 is an internal cumulative milestone. No crate is published until the
v0.90 checkpoint.

Source users should update the core dependency to `0.89.0`; the Hetzner package
version remains `0.44.0` during the cumulative train.

```toml
[dependencies]
cloud-sdk = "0.89.0"
cloud-sdk-hetzner = { version = "0.44.0", features = ["serde"] }
```

Construct an ordered inline firewall replacement and retain its exact typed
association:

```rust
use cloud_sdk::operation::PreparationStorageGuard;
use cloud_sdk_hetzner::robot::{
    RobotFirewallAction, RobotFirewallReplaceIntent,
    RobotFirewallReplaceRequest, RobotFirewallRule, RobotFirewallRules,
    RobotFirewallStatus, RobotServerNumber,
};

let input = [RobotFirewallRule::new(RobotFirewallAction::Discard)];
let rules = RobotFirewallRules::new(&input, &[])?;
let intent = RobotFirewallReplaceIntent::Inline {
    status: RobotFirewallStatus::Active,
    filter_ipv6: Some(false),
    whitelist_hos: true,
    rules,
};
let server = RobotServerNumber::new(321)?;
let request = RobotFirewallReplaceRequest::new(server, intent);
let mut path = [0_u8; 64];
let mut body = [0_u8; 16_384];
let mut storage = PreparationStorageGuard::new(&mut path, &mut body);
let prepared = storage.prepare_with(|buffers| request.prepare_bound(buffers))?;
# drop(prepared);
// Execute while `storage` remains borrowed. Both complete buffers are cleared
// when the guard drops.
# Ok::<(), Box<dyn core::error::Error>>(())
```

Inline rules and template application are separate intent variants. Do not
reconstruct forms manually: rule order affects policy, and safe preparation
prevents `template_id` from being combined with inline `rules` or
`whitelist_hos`. Firewall and template mutations use the request-bound
strong-digest permit flow; clear and delete require destructive authority.
