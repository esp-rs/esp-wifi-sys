use smoltcp::{
    iface::{Interface, Route, SocketStorage, Config, SocketSet},
    socket::dhcpv4::Socket as Dhcpv4Socket,
    wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address, ArpHardware, HardwareAddress}, phy::{Device, Medium},
};

use crate::wifi::get_sta_mac;

use super::WifiDevice;

#[macro_export]
macro_rules! create_network_stack_storage {
    ($socket_count:literal, $routes_count:literal, $multicast_storage_size:literal) => {{
        use smoltcp::iface::{Route, SocketStorage};
        use smoltcp::wire::{IpAddress, IpCidr, Ipv4Address};

        let mut socket_set_entries: [SocketStorage; $socket_count] = Default::default();
        let mut routes_storage: [Option<(IpCidr, Route)>; $routes_count] = Default::default();
        let ip_addr = IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 0);
        let mut ip_addrs = [ip_addr];
        let mut ipv4_multicast_storage = [None; $multicast_storage_size];

        (
            socket_set_entries,
            routes_storage,
            ip_addrs,
            ipv4_multicast_storage,
        )
    }};
}

#[macro_export]
macro_rules! network_stack_storage {
    ($param:ident) => {{
        (
            &mut $param.0,
            &mut $param.1,
            &mut $param.2,
            &mut $param.3,
        )
    }};
}

/// Convenient way to create an `smoltcp` ethernet interface
/// You can use the provided macros to create and pass a suitable backing storage.
pub fn create_network_interface<'a>(
    storage: (
        &'a mut [SocketStorage<'a>],
        &'a mut [Option<(IpCidr, Route)>],
        &'a mut [IpCidr; 1],
        &'a mut [Option<(Ipv4Address, ())>],
    ),
) -> (Interface, WifiDevice, SocketSet<'a>) {
    let socket_set_entries = storage.0;
    let routes_storage = storage.1;
    let ip_addrs = storage.2;
    let ipv4_multicast_groups = storage.3;

    let mut mac = [0u8; 6];
    get_sta_mac(&mut mac);
    let hw_address = EthernetAddress::from_bytes(&mac);

    let mut device = WifiDevice::new();

    let mut config = Config::new();

    if device.capabilities().medium == Medium::Ethernet {
        config.hardware_addr = Some(hw_address.into());
    }

    let iface = Interface::new(config, &mut device);

    let mut socket_set = SocketSet::new(socket_set_entries);

    let dhcp_socket = Dhcpv4Socket::new();
    socket_set.add(dhcp_socket);

    (iface, device, socket_set)
}
