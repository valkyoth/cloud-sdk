//! Source-locked active Robot operation inventory exposed by the client.

use cloud_sdk::Method;

/// One active Robot operation exposed through the typed client contracts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RobotClientMethodDescriptor {
    id: &'static str,
    method: Method,
    path: &'static str,
}

impl RobotClientMethodDescriptor {
    const fn new(id: &'static str, method: Method, path: &'static str) -> Self {
        Self { id, method, path }
    }

    /// Returns the source-locked operation identifier.
    #[must_use]
    pub const fn id(self) -> &'static str {
        self.id
    }

    /// Returns the exact HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        self.method
    }

    /// Returns the source template for the canonical request target.
    #[must_use]
    pub const fn path(self) -> &'static str {
        self.path
    }
}

/// Every active non-deprecated operation in the reviewed Robot inventory.
pub const ROBOT_CLIENT_METHODS: &[RobotClientMethodDescriptor] = &[
    RobotClientMethodDescriptor::new("list_servers", Method::Get, "/server"),
    RobotClientMethodDescriptor::new("get_server", Method::Get, "/server/{server-number}"),
    RobotClientMethodDescriptor::new("update_server", Method::Post, "/server/{server-number}"),
    RobotClientMethodDescriptor::new(
        "get_server_cancellation",
        Method::Get,
        "/server/{server-number}/cancellation",
    ),
    RobotClientMethodDescriptor::new(
        "create_server_cancellation",
        Method::Post,
        "/server/{server-number}/cancellation",
    ),
    RobotClientMethodDescriptor::new(
        "delete_server_cancellation",
        Method::Delete,
        "/server/{server-number}/cancellation",
    ),
    RobotClientMethodDescriptor::new("list_ips", Method::Get, "/ip"),
    RobotClientMethodDescriptor::new("get_ip", Method::Get, "/ip/{ip}"),
    RobotClientMethodDescriptor::new("update_ip", Method::Post, "/ip/{ip}"),
    RobotClientMethodDescriptor::new("get_ip_mac", Method::Get, "/ip/{ip}/mac"),
    RobotClientMethodDescriptor::new("set_ip_mac", Method::Put, "/ip/{ip}/mac"),
    RobotClientMethodDescriptor::new("delete_ip_mac", Method::Delete, "/ip/{ip}/mac"),
    RobotClientMethodDescriptor::new("get_ip_cancellation", Method::Get, "/ip/{ip}/cancellation"),
    RobotClientMethodDescriptor::new(
        "create_ip_cancellation",
        Method::Post,
        "/ip/{ip}/cancellation",
    ),
    RobotClientMethodDescriptor::new(
        "delete_ip_cancellation",
        Method::Delete,
        "/ip/{ip}/cancellation",
    ),
    RobotClientMethodDescriptor::new("list_subnets", Method::Get, "/subnet"),
    RobotClientMethodDescriptor::new("get_subnet", Method::Get, "/subnet/{net-ip}"),
    RobotClientMethodDescriptor::new("update_subnet", Method::Post, "/subnet/{net-ip}"),
    RobotClientMethodDescriptor::new("get_subnet_mac", Method::Get, "/subnet/{net-ip}/mac"),
    RobotClientMethodDescriptor::new("set_subnet_mac", Method::Put, "/subnet/{net-ip}/mac"),
    RobotClientMethodDescriptor::new("delete_subnet_mac", Method::Delete, "/subnet/{net-ip}/mac"),
    RobotClientMethodDescriptor::new(
        "get_subnet_cancellation",
        Method::Get,
        "/subnet/{net-ip}/cancellation",
    ),
    RobotClientMethodDescriptor::new(
        "create_subnet_cancellation",
        Method::Post,
        "/subnet/{net-ip}/cancellation",
    ),
    RobotClientMethodDescriptor::new(
        "delete_subnet_cancellation",
        Method::Delete,
        "/subnet/{ip}/cancellation",
    ),
    RobotClientMethodDescriptor::new("list_resets", Method::Get, "/reset"),
    RobotClientMethodDescriptor::new("get_reset", Method::Get, "/reset/{server-number}"),
    RobotClientMethodDescriptor::new("execute_reset", Method::Post, "/reset/{server-number}"),
    RobotClientMethodDescriptor::new("list_failovers", Method::Get, "/failover"),
    RobotClientMethodDescriptor::new("get_failover", Method::Get, "/failover/{failover-ip}"),
    RobotClientMethodDescriptor::new("update_failover", Method::Post, "/failover/{failover-ip}"),
    RobotClientMethodDescriptor::new("delete_failover", Method::Delete, "/failover/{failover-ip}"),
    RobotClientMethodDescriptor::new("get_wol", Method::Get, "/wol/{server-number}"),
    RobotClientMethodDescriptor::new("execute_wol", Method::Post, "/wol/{server-number}"),
    RobotClientMethodDescriptor::new("get_boot", Method::Get, "/boot/{server-number}"),
    RobotClientMethodDescriptor::new("get_rescue", Method::Get, "/boot/{server-number}/rescue"),
    RobotClientMethodDescriptor::new(
        "activate_rescue",
        Method::Post,
        "/boot/{server-number}/rescue",
    ),
    RobotClientMethodDescriptor::new(
        "deactivate_rescue",
        Method::Delete,
        "/boot/{server-number}/rescue",
    ),
    RobotClientMethodDescriptor::new(
        "get_last_rescue",
        Method::Get,
        "/boot/{server-number}/rescue/last",
    ),
    RobotClientMethodDescriptor::new("get_linux", Method::Get, "/boot/{server-number}/linux"),
    RobotClientMethodDescriptor::new(
        "activate_linux",
        Method::Post,
        "/boot/{server-number}/linux",
    ),
    RobotClientMethodDescriptor::new(
        "deactivate_linux",
        Method::Delete,
        "/boot/{server-number}/linux",
    ),
    RobotClientMethodDescriptor::new(
        "get_last_linux",
        Method::Get,
        "/boot/{server-number}/linux/last",
    ),
    RobotClientMethodDescriptor::new("get_vnc", Method::Get, "/boot/{server-number}/vnc"),
    RobotClientMethodDescriptor::new("activate_vnc", Method::Post, "/boot/{server-number}/vnc"),
    RobotClientMethodDescriptor::new(
        "deactivate_vnc",
        Method::Delete,
        "/boot/{server-number}/vnc",
    ),
    RobotClientMethodDescriptor::new("get_windows", Method::Get, "/boot/{server-number}/windows"),
    RobotClientMethodDescriptor::new(
        "activate_windows",
        Method::Post,
        "/boot/{server-number}/windows",
    ),
    RobotClientMethodDescriptor::new(
        "deactivate_windows",
        Method::Delete,
        "/boot/{server-number}/windows",
    ),
    RobotClientMethodDescriptor::new("list_rdns", Method::Get, "/rdns"),
    RobotClientMethodDescriptor::new("get_rdns", Method::Get, "/rdns/{ip}"),
    RobotClientMethodDescriptor::new("set_rdns", Method::Put, "/rdns/{ip}"),
    RobotClientMethodDescriptor::new("update_rdns", Method::Post, "/rdns/{ip}"),
    RobotClientMethodDescriptor::new("delete_rdns", Method::Delete, "/rdns/{ip}"),
    RobotClientMethodDescriptor::new("get_traffic", Method::Post, "/traffic"),
    RobotClientMethodDescriptor::new("list_ssh_keys", Method::Get, "/key"),
    RobotClientMethodDescriptor::new("create_ssh_key", Method::Post, "/key"),
    RobotClientMethodDescriptor::new("get_ssh_key", Method::Get, "/key/{fingerprint}"),
    RobotClientMethodDescriptor::new("update_ssh_key", Method::Post, "/key/{fingerprint}"),
    RobotClientMethodDescriptor::new("delete_ssh_key", Method::Delete, "/key/{fingerprint}"),
    RobotClientMethodDescriptor::new("list_server_products", Method::Get, "/order/server/product"),
    RobotClientMethodDescriptor::new(
        "get_server_product",
        Method::Get,
        "/order/server/product/{product-id}",
    ),
    RobotClientMethodDescriptor::new(
        "list_server_transactions",
        Method::Get,
        "/order/server/transaction",
    ),
    RobotClientMethodDescriptor::new(
        "create_server_transaction",
        Method::Post,
        "/order/server/transaction",
    ),
    RobotClientMethodDescriptor::new(
        "get_server_transaction",
        Method::Get,
        "/order/server/transaction/{id}",
    ),
    RobotClientMethodDescriptor::new(
        "list_server_market_products",
        Method::Get,
        "/order/server_market/product",
    ),
    RobotClientMethodDescriptor::new(
        "get_server_market_product",
        Method::Get,
        "/order/server_market/product/{product-id}",
    ),
    RobotClientMethodDescriptor::new(
        "list_server_market_transactions",
        Method::Get,
        "/order/server_market/transaction",
    ),
    RobotClientMethodDescriptor::new(
        "create_server_market_transaction",
        Method::Post,
        "/order/server_market/transaction",
    ),
    RobotClientMethodDescriptor::new(
        "get_server_market_transaction",
        Method::Get,
        "/order/server_market/transaction/{id}",
    ),
    RobotClientMethodDescriptor::new(
        "list_server_addon_products",
        Method::Get,
        "/order/server_addon/{server-number}/product",
    ),
    RobotClientMethodDescriptor::new(
        "list_server_addon_transactions",
        Method::Get,
        "/order/server_addon/transaction",
    ),
    RobotClientMethodDescriptor::new(
        "create_server_addon_transaction",
        Method::Post,
        "/order/server_addon/transaction",
    ),
    RobotClientMethodDescriptor::new(
        "get_server_addon_transaction",
        Method::Get,
        "/order/server_addon/transaction/{id}",
    ),
    RobotClientMethodDescriptor::new("list_order_currencies", Method::Get, "/order/currency"),
    RobotClientMethodDescriptor::new("get_firewall", Method::Get, "/firewall/{server-id}"),
    RobotClientMethodDescriptor::new("update_firewall", Method::Post, "/firewall/{server-id}"),
    RobotClientMethodDescriptor::new("delete_firewall", Method::Delete, "/firewall/{server-id}"),
    RobotClientMethodDescriptor::new("list_firewall_templates", Method::Get, "/firewall/template"),
    RobotClientMethodDescriptor::new(
        "create_firewall_template",
        Method::Post,
        "/firewall/template",
    ),
    RobotClientMethodDescriptor::new(
        "get_firewall_template",
        Method::Get,
        "/firewall/template/{template-id}",
    ),
    RobotClientMethodDescriptor::new(
        "update_firewall_template",
        Method::Post,
        "/firewall/template/{template-id}",
    ),
    RobotClientMethodDescriptor::new(
        "delete_firewall_template",
        Method::Delete,
        "/firewall/template/{template-id}",
    ),
    RobotClientMethodDescriptor::new("list_vswitches", Method::Get, "/vswitch"),
    RobotClientMethodDescriptor::new("create_vswitch", Method::Post, "/vswitch"),
    RobotClientMethodDescriptor::new("get_vswitch", Method::Get, "/vswitch/{vswitch-id}"),
    RobotClientMethodDescriptor::new("update_vswitch", Method::Post, "/vswitch/{vswitch-id}"),
    RobotClientMethodDescriptor::new("delete_vswitch", Method::Delete, "/vswitch/{vswitch-id}"),
    RobotClientMethodDescriptor::new(
        "add_vswitch_servers",
        Method::Post,
        "/vswitch/{vswitch-id}/server",
    ),
    RobotClientMethodDescriptor::new(
        "remove_vswitch_servers",
        Method::Delete,
        "/vswitch/{vswitch-id}/server",
    ),
];
