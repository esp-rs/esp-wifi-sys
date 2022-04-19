use smoltcp::{
    iface::{InterfaceBuilder, Neighbor, NeighborCache, Route, Routes, SocketStorage},
    socket::{Dhcpv4Socket, TcpSocket, TcpSocketBuffer},
    wire::{EthernetAddress, IpAddress, IpCidr},
};
use smoltcp_nal::NetworkStack;

use crate::wifi::get_sta_mac;

use super::WifiDevice;

#[macro_export]
macro_rules! create_network_stack_storage {
    ($socket_count:literal , $cache_count:literal , $routes_count:literal) => {{
        use smoltcp::iface::{Neighbor, NeighborCache, Route, SocketStorage};
        use smoltcp::wire::{IpAddress, IpCidr, Ipv4Address};

        let mut socket_set_entries: [SocketStorage; $socket_count] = Default::default();
        let mut neighbor_cache_storage: [Option<(IpAddress, Neighbor)>; $cache_count] =
            Default::default();
        let mut routes_storage: [Option<(IpCidr, Route)>; $routes_count] = Default::default();
        let ip_addr = IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 0);
        let mut ip_addrs = [ip_addr];

        (
            socket_set_entries,
            neighbor_cache_storage,
            routes_storage,
            ip_addrs,
        )
    }};
}

#[macro_export]
macro_rules! network_stack_storage {
    ($param:ident) => {{
        (&mut $param.0, &mut $param.1, &mut $param.2, &mut $param.3)
    }};
}

/// Convenient way to create an `embedded-nal` implementation using `smoltcp-nal`
/// You can use the provided macros to create and pass a suitable backing storage.
///
/// Currently `smoltcp-nal` only implements client stacks. If you need server functionality
/// you need to fall back to `smoltcp` itself.
pub fn create_network_stack<'a>(
    storage: (
        &'a mut [SocketStorage<'a>],
        &'a mut [Option<(IpAddress, Neighbor)>],
        &'a mut [Option<(IpCidr, Route)>],
        &'a mut [IpCidr; 1],
    ),
) -> NetworkStack<'a, WifiDevice, WifiClock> {
    let socket_set_entries = storage.0;
    let neighbor_cache_storage = storage.1;
    let routes_storage = storage.2;
    let ip_addrs = storage.3;

    let mut mac = [0u8; 6];
    get_sta_mac(&mut mac);
    let hw_address = EthernetAddress::from_bytes(&mac);

    let device = WifiDevice::new();

    let neighbor_cache = NeighborCache::new(&mut neighbor_cache_storage[..]);
    let routes = Routes::new(&mut routes_storage[..]);

    let sockets_to_add = socket_set_entries.len() - 1;
    let mut ethernet = InterfaceBuilder::new(device, socket_set_entries)
        .hardware_addr(smoltcp::wire::HardwareAddress::Ethernet(hw_address))
        .neighbor_cache(neighbor_cache)
        .ip_addrs(&mut ip_addrs[..])
        .routes(routes)
        .finalize();

    for _ in 0..sockets_to_add {
        let rx_tx_socket1 = {
            static mut TCP_SERVER_RX_DATA: [u8; 2500] = [0; 2500];
            static mut TCP_SERVER_TX_DATA: [u8; 2500] = [0; 2500];

            let tcp_rx_buffer = unsafe { TcpSocketBuffer::new(&mut TCP_SERVER_RX_DATA[..]) };
            let tcp_tx_buffer = unsafe { TcpSocketBuffer::new(&mut TCP_SERVER_TX_DATA[..]) };

            TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer)
        };
        ethernet.add_socket(rx_tx_socket1);
    }

    let dhcp_socket = Dhcpv4Socket::new();
    ethernet.add_socket(dhcp_socket);

    let clock = WifiClock {};
    let network_stack = NetworkStack::new(ethernet, clock);

    network_stack
}

/// The clock used by `smoltcp-nal`
pub struct WifiClock {}

impl embedded_time::Clock for WifiClock {
    type T = u32;

    const SCALING_FACTOR: embedded_time::rate::Fraction =
        embedded_time::rate::Fraction::new(1, (crate::timer::TICKS_PER_SECOND / 1000) as u32);

    fn try_now(&self) -> Result<embedded_time::Instant<Self>, embedded_time::clock::Error> {
        Ok(embedded_time::Instant::new(
            (crate::timer::get_systimer_count() / 1000) as u32,
        ))
    }
}
