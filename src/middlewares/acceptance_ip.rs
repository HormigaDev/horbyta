//! La IP de una aceptación procede del socket o de un proxy explícitamente confiable.
use axum::http::HeaderMap;
use std::net::IpAddr;

pub fn acceptance_ip(peer: IpAddr, headers: &HeaderMap, trusted: &str) -> (IpAddr, &'static str) {
    if trusted
        .split(',')
        .filter_map(|s| s.trim().parse::<IpAddr>().ok())
        .any(|ip| ip == peer)
    {
        if let Some(ip) = headers
            .get("x-real-ip")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
        {
            return (ip, "trusted_proxy");
        }
    }
    (peer, "socket")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn no_confia_en_cabeceras_de_un_peer_desconocido() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", "203.0.113.9".parse().unwrap());
        let peer = "192.0.2.8".parse().unwrap();
        assert_eq!(acceptance_ip(peer, &headers, "127.0.0.1"), (peer, "socket"));
    }
    #[test]
    fn proxy_explicito_ipv6_y_fallback_real() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", "2001:db8::1".parse().unwrap());
        assert_eq!(
            acceptance_ip("::1".parse().unwrap(), &headers, "127.0.0.1,::1"),
            ("2001:db8::1".parse().unwrap(), "trusted_proxy")
        );
        headers.insert("x-real-ip", "falso".parse().unwrap());
        assert_eq!(
            acceptance_ip("::1".parse().unwrap(), &headers, "::1"),
            ("::1".parse().unwrap(), "socket")
        );
    }
}
