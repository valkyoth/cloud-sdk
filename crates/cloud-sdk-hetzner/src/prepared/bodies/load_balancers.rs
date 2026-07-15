//! Load Balancer JSON bodies.

use crate::cloud::load_balancers::{
    HttpHealthCheck, HttpServiceConfig, HttpsServiceConfig, LoadBalancerAddServiceRequest,
    LoadBalancerAddTargetRequest, LoadBalancerAttachNetworkRequest,
    LoadBalancerChangeAlgorithmRequest, LoadBalancerChangeDnsPtrRequest,
    LoadBalancerChangeTypeRequest, LoadBalancerCreateRequest, LoadBalancerDeleteServiceRequest,
    LoadBalancerDetachNetworkRequest, LoadBalancerDnsPtrIntent, LoadBalancerHealthCheck,
    LoadBalancerNetworkAddress, LoadBalancerPlacement, LoadBalancerProtectionRequest,
    LoadBalancerRemoveTargetRequest, LoadBalancerService, LoadBalancerServiceProtocol,
    LoadBalancerTarget, LoadBalancerUpdateRequest, LoadBalancerUpdateServiceRequest,
};
use crate::prepared::{HetznerPreparationError, JsonWriter};

body_wire!(LoadBalancerCreateRequest<'_>, request => request.endpoint(), "create_load_balancer", write_create);
body_wire!(LoadBalancerUpdateRequest<'_>, request => request.endpoint(), "update_load_balancer", write_update);
body_component!(
    LoadBalancerAddServiceRequest<'_>,
    "add_load_balancer_service",
    write_add_service
);
body_component!(
    LoadBalancerUpdateServiceRequest<'_>,
    "update_load_balancer_service",
    write_update_service
);
body_component!(
    LoadBalancerDeleteServiceRequest,
    "delete_load_balancer_service",
    write_delete_service
);
body_component!(
    LoadBalancerAttachNetworkRequest<'_>,
    "attach_load_balancer_to_network",
    write_attach_network
);
body_component!(
    LoadBalancerDetachNetworkRequest,
    "detach_load_balancer_from_network",
    write_detach_network
);
body_component!(
    LoadBalancerChangeDnsPtrRequest<'_>,
    "change_load_balancer_dns_ptr",
    write_dns_ptr
);
body_component!(
    LoadBalancerProtectionRequest,
    "change_load_balancer_protection",
    write_protection
);
body_component!(
    LoadBalancerChangeTypeRequest<'_>,
    "change_load_balancer_type",
    write_change_type
);
body_component!(
    LoadBalancerChangeAlgorithmRequest,
    "change_load_balancer_algorithm",
    write_algorithm
);
body_component!(
    LoadBalancerAddTargetRequest<'_>,
    "add_load_balancer_target",
    write_add_target
);
body_component!(
    LoadBalancerRemoveTargetRequest<'_>,
    "remove_load_balancer_target",
    write_remove_target
);

fn write_create(
    request: LoadBalancerCreateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_string(first, "name", request.name().as_str())?;
        writer.field_string(
            first,
            "load_balancer_type",
            request.load_balancer_type().as_str(),
        )?;
        if let Some(algorithm) = request.algorithm() {
            write_algorithm_field(writer, first, algorithm.as_api_str())?;
        }
        if let Some(labels) = request.labels() {
            writer.field_labels(first, "labels", labels)?;
        }
        if let Some(enabled) = request.public_interface() {
            writer.field_bool(first, "public_interface", enabled)?;
        }
        if let Some(network) = request.network() {
            writer.field_u64(first, "network", network.get())?;
        }
        if let Some(placement) = request.placement() {
            match placement {
                LoadBalancerPlacement::Location(location) => {
                    writer.field_string(first, "location", location.as_str())?;
                }
                LoadBalancerPlacement::NetworkZone(zone) => {
                    writer.field_string(first, "network_zone", zone.as_str())?;
                }
            }
        }
        if let Some(services) = request.services() {
            writer.field(first, "services")?;
            writer.begin_array()?;
            let mut item = true;
            for service in services {
                writer.value(&mut item)?;
                write_service(writer, *service)?;
            }
            writer.end_array()?;
        }
        if let Some(targets) = request.targets() {
            writer.field(first, "targets")?;
            writer.begin_array()?;
            let mut item = true;
            for target in targets {
                writer.value(&mut item)?;
                write_target(writer, target.target(), Some(target.use_private_ip()))?;
            }
            writer.end_array()?;
        }
        Ok(())
    })
}

fn write_update(
    request: LoadBalancerUpdateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        if let Some(name) = request.name() {
            writer.field_string(first, "name", name.as_str())?;
        }
        if let Some(labels) = request.labels() {
            writer.field_labels(first, "labels", labels)?;
        }
        Ok(())
    })
}

fn write_add_service(
    request: LoadBalancerAddServiceRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, _| {
        write_service_fields(writer, request.service())
    })
}

fn write_update_service(
    request: LoadBalancerUpdateServiceRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        let update = request.update();
        writer.field_u64(first, "listen_port", u64::from(update.listen_port().get()))?;
        if let Some(protocol) = update.protocol() {
            write_protocol(writer, first, protocol)?;
        }
        if let Some(port) = update.destination_port() {
            writer.field_u64(first, "destination_port", u64::from(port.get()))?;
        }
        if let Some(enabled) = update.proxy_protocol() {
            writer.field_bool(first, "proxyprotocol", enabled)?;
        }
        if let Some(health_check) = update.health_check() {
            writer.field(first, "health_check")?;
            write_health_check(writer, health_check)?;
        }
        Ok(())
    })
}

fn write_delete_service(
    request: LoadBalancerDeleteServiceRequest,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_u64(first, "listen_port", u64::from(request.listen_port().get()))
    })
}

fn write_attach_network(
    request: LoadBalancerAttachNetworkRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_u64(first, "network", request.network().get())?;
        if let Some(address) = request.address() {
            match address {
                LoadBalancerNetworkAddress::Ip(ip) => {
                    writer.field_string(first, "ip", ip.as_str())?;
                }
                LoadBalancerNetworkAddress::IpRange(range) => {
                    writer.field_string(first, "ip_range", range.as_str())?;
                }
                LoadBalancerNetworkAddress::IpInRange { ip, ip_range } => {
                    writer.field_string(first, "ip", ip.as_str())?;
                    writer.field_string(first, "ip_range", ip_range.as_str())?;
                }
            }
        }
        Ok(())
    })
}

fn write_detach_network(
    request: LoadBalancerDetachNetworkRequest,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_u64(first, "network", request.network().get())
    })
}

fn write_dns_ptr(
    request: LoadBalancerChangeDnsPtrRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_string(first, "ip", request.ip().as_str())?;
        match request.dns_ptr() {
            LoadBalancerDnsPtrIntent::Set(value) => {
                writer.field_string(first, "dns_ptr", value.as_str())
            }
            LoadBalancerDnsPtrIntent::Reset => writer.field_null(first, "dns_ptr"),
        }
    })
}

fn write_protection(
    request: LoadBalancerProtectionRequest,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_bool(first, "delete", request.delete())
    })
}

fn write_change_type(
    request: LoadBalancerChangeTypeRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_string(
            first,
            "load_balancer_type",
            request.load_balancer_type().as_str(),
        )
    })
}

fn write_algorithm(
    request: LoadBalancerChangeAlgorithmRequest,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        write_algorithm_field(writer, first, request.algorithm().as_api_str())
    })
}

fn write_add_target(
    request: LoadBalancerAddTargetRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, _| {
        write_target_fields(writer, request.target(), Some(request.use_private_ip()))
    })
}

fn write_remove_target(
    request: LoadBalancerRemoveTargetRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, _| {
        write_target_fields(writer, request.target(), None)
    })
}

fn write_service(
    writer: &mut JsonWriter<'_>,
    service: LoadBalancerService<'_>,
) -> Result<(), HetznerPreparationError> {
    writer.begin_object()?;
    write_service_fields(writer, service)?;
    writer.end_object()
}

fn write_service_fields(
    writer: &mut JsonWriter<'_>,
    service: LoadBalancerService<'_>,
) -> Result<(), HetznerPreparationError> {
    let mut first = true;
    write_protocol(writer, &mut first, service.protocol())?;
    writer.field_u64(
        &mut first,
        "listen_port",
        u64::from(service.listen_port().get()),
    )?;
    writer.field_u64(
        &mut first,
        "destination_port",
        u64::from(service.destination_port().get()),
    )?;
    writer.field_bool(&mut first, "proxyprotocol", service.proxy_protocol())?;
    writer.field(&mut first, "health_check")?;
    write_health_check(writer, service.health_check())
}

fn write_protocol(
    writer: &mut JsonWriter<'_>,
    first: &mut bool,
    protocol: LoadBalancerServiceProtocol<'_>,
) -> Result<(), HetznerPreparationError> {
    match protocol {
        LoadBalancerServiceProtocol::Tcp => writer.field_string(first, "protocol", "tcp"),
        LoadBalancerServiceProtocol::Http(config) => {
            writer.field_string(first, "protocol", "http")?;
            write_http_field(writer, first, config, None)
        }
        LoadBalancerServiceProtocol::Https(config) => {
            writer.field_string(first, "protocol", "https")?;
            write_http_field(writer, first, config.http(), Some(config))
        }
    }
}

fn write_http_field(
    writer: &mut JsonWriter<'_>,
    first: &mut bool,
    config: HttpServiceConfig<'_>,
    https: Option<HttpsServiceConfig<'_>>,
) -> Result<(), HetznerPreparationError> {
    writer.field(first, "http")?;
    writer.begin_object()?;
    let mut item = true;
    if let Some(name) = config.cookie_name() {
        writer.field_string(&mut item, "cookie_name", name.as_str())?;
    }
    if let Some(lifetime) = config.cookie_lifetime() {
        writer.field_u64(&mut item, "cookie_lifetime", u64::from(lifetime))?;
    }
    if let Some(timeout) = config.timeout_idle() {
        writer.field_u64(&mut item, "timeout_idle", u64::from(timeout))?;
    }
    if let Some(https) = https {
        if let Some(certificates) = https.certificates() {
            writer.field(&mut item, "certificates")?;
            writer.begin_array()?;
            let mut certificate = true;
            for id in certificates {
                writer.value(&mut certificate)?;
                writer.u64(id.get())?;
            }
            writer.end_array()?;
        }
        writer.field_bool(&mut item, "redirect_http", https.redirect_http())?;
    }
    writer.field_bool(&mut item, "sticky_sessions", config.sticky_sessions())?;
    writer.end_object()
}

fn write_health_check(
    writer: &mut JsonWriter<'_>,
    health_check: LoadBalancerHealthCheck<'_>,
) -> Result<(), HetznerPreparationError> {
    writer.begin_object()?;
    let mut first = true;
    let (protocol, settings, http) = match health_check {
        LoadBalancerHealthCheck::Tcp(settings) => ("tcp", settings, None),
        LoadBalancerHealthCheck::Http { settings, http } => ("http", settings, Some(http)),
    };
    writer.field_string(&mut first, "protocol", protocol)?;
    writer.field_u64(&mut first, "port", u64::from(settings.port().get()))?;
    writer.field_u64(&mut first, "interval", u64::from(settings.interval()))?;
    writer.field_u64(&mut first, "timeout", u64::from(settings.timeout()))?;
    writer.field_u64(&mut first, "retries", u64::from(settings.retries()))?;
    if let Some(http) = http {
        write_health_http(writer, &mut first, http)?;
    }
    writer.end_object()
}

fn write_health_http(
    writer: &mut JsonWriter<'_>,
    first: &mut bool,
    http: HttpHealthCheck<'_>,
) -> Result<(), HetznerPreparationError> {
    writer.field(first, "http")?;
    writer.begin_object()?;
    let mut item = true;
    match http.domain() {
        Some(domain) => writer.field_string(&mut item, "domain", domain)?,
        None => writer.field_null(&mut item, "domain")?,
    }
    writer.field_string(&mut item, "path", http.path().as_str())?;
    if let Some(response) = http.response() {
        writer.field_string(&mut item, "response", response.as_str())?;
    }
    if let Some(status_codes) = http.status_codes() {
        writer.field(&mut item, "status_codes")?;
        writer.begin_array()?;
        let mut status = true;
        for code in status_codes {
            writer.value(&mut status)?;
            writer.string(code.as_str())?;
        }
        writer.end_array()?;
    }
    writer.field_bool(&mut item, "tls", http.tls())?;
    writer.end_object()
}

fn write_target(
    writer: &mut JsonWriter<'_>,
    target: LoadBalancerTarget<'_>,
    use_private_ip: Option<bool>,
) -> Result<(), HetznerPreparationError> {
    writer.begin_object()?;
    write_target_fields(writer, target, use_private_ip)?;
    writer.end_object()
}

fn write_target_fields(
    writer: &mut JsonWriter<'_>,
    target: LoadBalancerTarget<'_>,
    use_private_ip: Option<bool>,
) -> Result<(), HetznerPreparationError> {
    let mut first = true;
    match target {
        LoadBalancerTarget::Server { id, public_ip } => {
            writer.field_string(&mut first, "type", "server")?;
            writer.field(&mut first, "server")?;
            writer.begin_object()?;
            let mut server = true;
            writer.field_u64(&mut server, "id", id.get())?;
            if let Some(ip) = public_ip {
                writer.field_string(&mut server, "ip", ip.as_str())?;
            }
            writer.end_object()?;
        }
        LoadBalancerTarget::LabelSelector(selector) => {
            writer.field_string(&mut first, "type", "label_selector")?;
            writer.field(&mut first, "label_selector")?;
            writer.begin_object()?;
            let mut label = true;
            writer.field_string(&mut label, "selector", selector.as_str())?;
            writer.end_object()?;
        }
        LoadBalancerTarget::Ip(ip) => {
            writer.field_string(&mut first, "type", "ip")?;
            writer.field(&mut first, "ip")?;
            writer.begin_object()?;
            let mut value = true;
            writer.field_string(&mut value, "ip", ip.as_str())?;
            writer.end_object()?;
        }
    }
    if let Some(use_private_ip) = use_private_ip {
        writer.field_bool(&mut first, "use_private_ip", use_private_ip)?;
    }
    Ok(())
}

fn write_algorithm_field(
    writer: &mut JsonWriter<'_>,
    first: &mut bool,
    algorithm: &str,
) -> Result<(), HetznerPreparationError> {
    writer.field(first, "algorithm")?;
    writer.begin_object()?;
    let mut item = true;
    writer.field_string(&mut item, "type", algorithm)?;
    writer.end_object()
}

fn object<F>(output: &mut [u8], write: F) -> Result<usize, HetznerPreparationError>
where
    F: FnOnce(&mut JsonWriter<'_>, &mut bool) -> Result<(), HetznerPreparationError>,
{
    let mut writer = JsonWriter::new(output);
    writer.begin_object()?;
    let mut first = true;
    write(&mut writer, &mut first)?;
    writer.end_object()?;
    Ok(writer.len())
}
