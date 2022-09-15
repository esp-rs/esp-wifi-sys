use core::cell::RefCell;
use embedded_io::blocking::Read;
use embedded_io::blocking::Write;
use embedded_io::Io;
use smoltcp::iface::SocketHandle;
use smoltcp::time::Instant;
use smoltcp::wire::Ipv4Address;
use smoltcp::{
    iface::{Interface, InterfaceBuilder, Neighbor, NeighborCache, Route, Routes, SocketStorage},
    socket::{Dhcpv4Socket, TcpSocket, TcpSocketBuffer},
    wire::{EthernetAddress, IpAddress, IpCidr},
};

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

/// Convenient way to create an `smoltcp` ethernet interface
/// You can use the provided macros to create and pass a suitable backing storage.
pub fn create_network_interface<'a>(
    storage: (
        &'a mut [SocketStorage<'a>],
        &'a mut [Option<(IpAddress, Neighbor)>],
        &'a mut [Option<(IpCidr, Route)>],
        &'a mut [IpCidr; 1],
    ),
) -> Interface<WifiDevice> {
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
            static mut TCP_SERVER_RX_DATA: [u8; 1536] = [0; 1536];
            static mut TCP_SERVER_TX_DATA: [u8; 1536] = [0; 1536];

            let tcp_rx_buffer = unsafe { TcpSocketBuffer::new(&mut TCP_SERVER_RX_DATA[..]) };
            let tcp_tx_buffer = unsafe { TcpSocketBuffer::new(&mut TCP_SERVER_TX_DATA[..]) };

            TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer)
        };
        ethernet.add_socket(rx_tx_socket1);
    }

    let dhcp_socket = Dhcpv4Socket::new();
    ethernet.add_socket(dhcp_socket);

    ethernet
}

// Following code is not well tested, yet.
// It's currently more or less just here for the DHCP example.
// Might get replaced or improved in future.

pub struct Network<'a> {
    interface: RefCell<crate::wifi_interface::Wifi<'a>>,
    current_millis_fn: fn() -> u64,
    local_port: RefCell<u16>,
}

impl<'a> Network<'a> {
    pub fn new(
        interface: crate::wifi_interface::Wifi<'a>,
        current_millis_fn: fn() -> u64,
    ) -> Network {
        Self {
            interface: RefCell::new(interface),
            current_millis_fn,
            local_port: RefCell::new(41000),
        }
    }

    fn with_interface<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut crate::wifi_interface::Wifi<'a>) -> R,
    {
        let mut interface = self.interface.borrow_mut();
        f(&mut interface)
    }

    pub fn get_socket<'s>(&'s mut self) -> Socket<'s, 'a>
    where
        'a: 's,
    {
        let socket_handle = self.with_interface(|interface| {
            let (socket_handle, _) = interface.network_interface().sockets_mut().next().unwrap();
            socket_handle
        });

        Socket {
            socket_handle,
            network: self,
        }
    }

    pub fn work(&self) {
        loop {
            self.with_interface(|interface| interface.poll_dhcp().ok());
            if let Ok(false) = self.with_interface(|interface| {
                interface
                    .network_interface()
                    .poll(Instant::from_millis((self.current_millis_fn)() as i64))
            }) {
                break;
            }
        }
    }

    fn next_local_port(&self) -> u16 {
        let mut local_port = self.local_port.borrow_mut();
        *local_port += 1;
        if *local_port == 65535 {
            *local_port = 41000;
        }
        *local_port
    }
}

pub struct Socket<'s, 'n: 's> {
    socket_handle: SocketHandle,
    network: &'s Network<'n>,
}

impl<'s, 'n: 's> Socket<'s, 'n> {
    pub fn open<'i>(&'i mut self, addr: Ipv4Address, port: u16) -> Result<(), IoError>
    where
        's: 'i,
    {
        {
            self.network.with_interface(|interface| {
                let (sock, cx) = interface
                    .network_interface()
                    .get_socket_and_context::<TcpSocket>(self.socket_handle);
                let remote_endpoint = (addr, port);
                sock.connect(cx, remote_endpoint, self.network.next_local_port())
                    .unwrap();
            });
        }

        loop {
            let can_send = self.network.with_interface(|interface| {
                let sock = interface
                    .network_interface()
                    .get_socket::<TcpSocket>(self.socket_handle);
                if sock.can_send() {
                    true
                } else {
                    false
                }
            });

            if can_send {
                break;
            }

            self.work();
        }

        Ok(())
    }

    pub fn disconnect(&mut self) {
        self.network.with_interface(|interface| {
            interface
                .network_interface()
                .get_socket::<TcpSocket>(self.socket_handle)
                .abort();
        });

        self.work();
    }

    pub fn work(&mut self) {
        loop {
            self.network
                .with_interface(|interface| interface.poll_dhcp().ok());
            if let Ok(false) = self.network.with_interface(|interface| {
                interface
                    .network_interface()
                    .poll(Instant::from_millis(
                        (self.network.current_millis_fn)() as i64
                    ))
            }) {
                break;
            }
        }
    }
}

#[derive(Debug)]
pub enum IoError {
    Other(smoltcp::Error),
    SocketClosed,
}

impl embedded_io::Error for IoError {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl From<smoltcp::Error> for IoError {
    fn from(e: smoltcp::Error) -> Self {
        IoError::Other(e)
    }
}

impl<'s, 'n: 's> Io for Socket<'s, 'n> {
    type Error = IoError;
}

impl<'s, 'n: 's> Read for Socket<'s, 'n> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        loop {
            self.network.with_interface(|interface| {
                interface
                    .network_interface()
                    .poll(Instant::from_millis(
                        (self.network.current_millis_fn)() as i64
                    ))
                    .unwrap();
            });

            let (may_recv, is_open, can_recv) = self.network.with_interface(|interface| {
                let socket = interface
                    .network_interface()
                    .get_socket::<TcpSocket>(self.socket_handle);

                (socket.may_recv(), socket.is_open(), socket.can_recv())
            });
            if may_recv {
                break;
            }

            if !is_open {
                return Err(IoError::SocketClosed);
            }

            if !can_recv {
                return Err(IoError::SocketClosed);
            }
        }

        loop {
            let res = self.network.with_interface(|interface| {
                interface
                    .network_interface()
                    .poll(Instant::from_millis(
                        (self.network.current_millis_fn)() as i64
                    ))
            });

            if let Ok(false) = res {
                break;
            }
        }

        self.network.with_interface(|interface| {
            let socket = interface
                .network_interface()
                .get_socket::<TcpSocket>(self.socket_handle);

            socket.recv_slice(buf).map_err(|e| IoError::Other(e))
        })
    }
}

impl<'s, 'n: 's> Write for Socket<'s, 'n> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        loop {
            self.network.with_interface(|interface| {
                interface
                    .network_interface()
                    .poll(Instant::from_millis(
                        (self.network.current_millis_fn)() as i64
                    ))
                    .unwrap();
            });

            let (may_send, is_open, can_send) = self.network.with_interface(|interface| {
                let socket = interface
                    .network_interface()
                    .get_socket::<TcpSocket>(self.socket_handle);

                (socket.may_send(), socket.is_open(), socket.can_send())
            });

            if may_send {
                break;
            }

            if !is_open {
                return Err(IoError::SocketClosed);
            }

            if !can_send {
                return Err(IoError::SocketClosed);
            }
        }

        loop {
            let res = self.network.with_interface(|interface| {
                interface
                    .network_interface()
                    .poll(Instant::from_millis(
                        (self.network.current_millis_fn)() as i64
                    ))
            });

            if let Ok(false) = res {
                break;
            }
        }

        let res = self.network.with_interface(|interface| {
            let socket = interface
                .network_interface()
                .get_socket::<TcpSocket>(self.socket_handle);

            let mut written = 0;
            loop {
                match socket.send_slice(&buf[written..]) {
                    Ok(len) => {
                        written += len;

                        if written >= buf.len() {
                            break Ok(written);
                        }

                        log::info!("not fully written: {}", len);
                    }
                    Err(err) => break Err(IoError::Other(err)),
                }
            }
        });

        res
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        loop {
            let res = self.network.with_interface(|interface| {
                interface
                    .network_interface()
                    .poll(Instant::from_millis(
                        (self.network.current_millis_fn)() as i64
                    ))
            });

            if let Ok(false) = res {
                break;
            }
        }

        Ok(())
    }
}
