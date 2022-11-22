use core::cell::RefCell;
use core::fmt::Display;
use embedded_io::blocking::{Read, Write};
use embedded_io::Io;

use embedded_svc::wifi::{AccessPointInfo, AuthMethod, SecondaryChannel};
use enumset::EnumSet;
use smoltcp::iface::{Interface, SocketHandle};
use smoltcp::socket::{Dhcpv4Socket, TcpSocket};
use smoltcp::storage::PacketMetadata;
use smoltcp::time::Instant;
use smoltcp::wire::{IpAddress, IpCidr, IpEndpoint, Ipv4Address};

use crate::current_millis;
use crate::wifi::WifiDevice;

/// An implementation of `embedded-svc`'s wifi trait.
pub struct Wifi<'a> {
    network_interface: Interface<'a, WifiDevice>,
    current_config: embedded_svc::wifi::Configuration,
    network_config: Option<smoltcp::socket::Dhcpv4Config>,
    dhcp_socket_handle: Option<SocketHandle>,
}

impl<'a> Wifi<'a> {
    /// Create a new instance from a `NetworkStack`
    pub fn new(mut network_interface: Interface<'a, WifiDevice>) -> Wifi<'a> {
        let mut dhcp_socket_handle: Option<SocketHandle> = None;

        for (handle, socket) in network_interface.sockets_mut() {
            match socket {
                smoltcp::socket::Socket::Dhcpv4(_) => dhcp_socket_handle = Some(handle),
                _ => {}
            }
        }

        Wifi {
            network_interface,
            current_config: embedded_svc::wifi::Configuration::default(),
            network_config: None,
            dhcp_socket_handle,
        }
    }

    /// Get a mutable reference to the `NetworkStack`
    pub fn network_interface(&mut self) -> &mut Interface<'a, WifiDevice> {
        &mut self.network_interface
    }

    /// Convenience function to poll the DHCP socket.
    pub fn poll_dhcp(&mut self) -> Result<(), WifiError> {
        if let Some(dhcp_handle) = self.dhcp_socket_handle {
            let dhcp_socket = self
                .network_interface
                .get_socket::<Dhcpv4Socket>(dhcp_handle);
            let event = dhcp_socket.poll();
            if let Some(event) = event {
                match event {
                    smoltcp::socket::Dhcpv4Event::Deconfigured => {
                        self.network_config = None;
                        self.network_interface
                            .routes_mut()
                            .remove_default_ipv4_route();
                    }
                    smoltcp::socket::Dhcpv4Event::Configured(config) => {
                        self.network_config = Some(config);
                        let address = config.address;
                        self.network_interface.update_ip_addrs(|addrs| {
                            let addr = addrs
                                .iter_mut()
                                .filter(|cidr| match cidr.address() {
                                    IpAddress::Ipv4(_) => true,
                                    _ => false,
                                })
                                .next()
                                .expect("No address");

                            *addr = IpCidr::Ipv4(address);
                        });
                        if let Some(route) = config.router {
                            self.network_interface
                                .routes_mut()
                                .add_default_ipv4_route(route)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum WifiError {
    Unknown(i32),
    SmolTcpError(smoltcp::Error),
    InitializationError(crate::InitializationError),
}

impl From<smoltcp::Error> for WifiError {
    fn from(error: smoltcp::Error) -> Self {
        WifiError::SmolTcpError(error)
    }
}

impl Display for WifiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> embedded_svc::wifi::Wifi for Wifi<'a> {
    type Error = WifiError;

    /// This currently only supports the `Client` capability.
    fn get_capabilities(&self) -> Result<EnumSet<embedded_svc::wifi::Capability>, Self::Error> {
        // for now we only support STA mode
        let mut caps = EnumSet::empty();
        caps.insert(embedded_svc::wifi::Capability::Client);
        Ok(caps)
    }

    /// A blocking wifi network scan.
    fn scan_n<const N: usize>(
        &mut self,
    ) -> Result<(heapless::Vec<AccessPointInfo, N>, usize), Self::Error> {
        crate::wifi::wifi_start_scan();

        let mut scanned = heapless::Vec::<AccessPointInfo, N>::new();
        let mut bss_total: u16 = N as u16;

        unsafe {
            crate::binary::include::esp_wifi_scan_get_ap_num(&mut bss_total);
            if bss_total as usize > N {
                bss_total = N as u16;
            }

            let mut records = [crate::binary::include::wifi_ap_record_t {
                bssid: [0u8; 6],
                ssid: [0u8; 33],
                primary: 0u8,
                second: 0u32,
                rssi: 0i8,
                authmode: 0u32,
                pairwise_cipher: 0u32,
                group_cipher: 0u32,
                ant: 0u32,
                _bitfield_align_1: [0u32; 0],
                _bitfield_1: crate::binary::include::__BindgenBitfieldUnit::new([0u8; 4usize]),
                country: crate::binary::include::wifi_country_t {
                    cc: [0; 3],
                    schan: 0u8,
                    nchan: 0u8,
                    max_tx_power: 0i8,
                    policy: 0u32,
                },
            }; N];

            crate::binary::include::esp_wifi_scan_get_ap_records(
                &mut bss_total,
                &mut records as *mut crate::binary::include::wifi_ap_record_t,
            );

            for i in 0..bss_total {
                let record = records[i as usize];
                let ssid_strbuf = crate::compat::common::StrBuf::from(&record.ssid as *const u8);

                let auth_method = match record.authmode {
                    crate::binary::include::wifi_auth_mode_t_WIFI_AUTH_OPEN => AuthMethod::None,
                    crate::binary::include::wifi_auth_mode_t_WIFI_AUTH_WEP => AuthMethod::WEP,
                    crate::binary::include::wifi_auth_mode_t_WIFI_AUTH_WPA_PSK => AuthMethod::WPA,
                    crate::binary::include::wifi_auth_mode_t_WIFI_AUTH_WPA2_PSK => {
                        AuthMethod::WPA2Personal
                    }
                    crate::binary::include::wifi_auth_mode_t_WIFI_AUTH_WPA_WPA2_PSK => {
                        AuthMethod::WPAWPA2Personal
                    }
                    crate::binary::include::wifi_auth_mode_t_WIFI_AUTH_WPA2_ENTERPRISE => {
                        AuthMethod::WPA2Enterprise
                    }
                    crate::binary::include::wifi_auth_mode_t_WIFI_AUTH_WPA3_PSK => {
                        AuthMethod::WPA3Personal
                    }
                    crate::binary::include::wifi_auth_mode_t_WIFI_AUTH_WPA2_WPA3_PSK => {
                        AuthMethod::WPA2WPA3Personal
                    }
                    crate::binary::include::wifi_auth_mode_t_WIFI_AUTH_WAPI_PSK => {
                        AuthMethod::WAPIPersonal
                    }
                    _ => panic!(),
                };

                let mut ssid = heapless::String::<32>::new();
                ssid.push_str(ssid_strbuf.as_str_ref()).ok();

                let ap_info = AccessPointInfo {
                    ssid: ssid,
                    bssid: record.bssid,
                    channel: record.primary,
                    secondary_channel: match record.second {
                        crate::binary::include::wifi_second_chan_t_WIFI_SECOND_CHAN_NONE => {
                            SecondaryChannel::None
                        }
                        crate::binary::include::wifi_second_chan_t_WIFI_SECOND_CHAN_ABOVE => {
                            SecondaryChannel::Above
                        }
                        crate::binary::include::wifi_second_chan_t_WIFI_SECOND_CHAN_BELOW => {
                            SecondaryChannel::Below
                        }
                        _ => panic!(),
                    },
                    signal_strength: record.rssi,
                    protocols: EnumSet::empty(), // TODO
                    auth_method: auth_method,
                };

                scanned.push(ap_info).ok();
            }
        }

        Ok((scanned, bss_total as usize))
    }

    /// Get the currently used configuration.
    fn get_configuration(&self) -> Result<embedded_svc::wifi::Configuration, Self::Error> {
        Ok(self.current_config.clone())
    }

    /// Set the configuration and start connecting.
    /// Currently only `ssid` and `password` is used. Trying anything but `Configuration::Client` will result in a panic!
    fn set_configuration(
        &mut self,
        conf: &embedded_svc::wifi::Configuration,
    ) -> Result<(), Self::Error> {
        self.current_config = conf.clone();

        match conf {
            embedded_svc::wifi::Configuration::None => panic!(),
            embedded_svc::wifi::Configuration::Client(_) => {}
            embedded_svc::wifi::Configuration::AccessPoint(_) => panic!(),
            embedded_svc::wifi::Configuration::Mixed(_, _) => panic!(),
        };

        Ok(())
    }

    fn start(&mut self) -> Result<(), Self::Error> {
        let res = crate::wifi::wifi_start();
        if res != 0 {
            return Err(WifiError::InitializationError(
                crate::InitializationError::General(res),
            ));
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<(), Self::Error> {
        let res = crate::wifi::wifi_stop();
        if res != 0 {
            return Err(WifiError::InitializationError(
                crate::InitializationError::General(res),
            ));
        }
        Ok(())
    }

    fn connect(&mut self) -> Result<(), Self::Error> {
        if let embedded_svc::wifi::Configuration::Client(config) = &self.current_config {
            let res = crate::wifi::wifi_connect(&config.ssid, &config.password);
            if res != 0 {
                return Err(WifiError::Unknown(res));
            }
        } else {
            panic!();
        }
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), Self::Error> {
        //FIXME: Is there a way to disconnect from Wifi?
        Ok(())
    }

    fn is_started(&self) -> Result<bool, Self::Error> {
        match crate::wifi::get_wifi_state() {
            crate::wifi::WifiState::StaStart => Ok(true),
            //FIXME: Should any enum trigger an error instead of returning false?
            _ => Ok(false),
        }
    }

    fn is_connected(&self) -> Result<bool, Self::Error> {
        match crate::wifi::get_wifi_state() {
            crate::wifi::WifiState::StaConnected => Ok(true),
            //FIXME: Should any enum trigger an error instead of returning false?
            _ => Ok(false),
        }
    }
}

pub fn timestamp() -> Instant {
    Instant::from_millis(current_millis() as i64)
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

    pub fn get_socket<'s>(
        &'s self,
        rx_buffer: &'a mut [u8],
        tx_buffer: &'a mut [u8],
    ) -> Socket<'s, 'a>
    where
        'a: 's,
    {
        let socket = TcpSocket::new(
            smoltcp::socket::TcpSocketBuffer::new(rx_buffer),
            smoltcp::socket::TcpSocketBuffer::new(tx_buffer),
        );

        let socket_handle =
            self.with_interface(|interface| interface.network_interface().add_socket(socket));

        Socket {
            socket_handle,
            network: self,
        }
    }

    pub fn get_udp_socket<'s>(
        &'s self,
        rx_meta: &'a mut [PacketMetadata<IpEndpoint>],
        rx_buffer: &'a mut [u8],
        tx_meta: &'a mut [PacketMetadata<IpEndpoint>],
        tx_buffer: &'a mut [u8],
    ) -> UdpSocket<'s, 'a>
    where
        'a: 's,
    {
        let socket = smoltcp::socket::UdpSocket::new(
            smoltcp::socket::UdpSocketBuffer::new(rx_meta, rx_buffer),
            smoltcp::socket::UdpSocketBuffer::new(tx_meta, tx_buffer),
        );

        let socket_handle =
            self.with_interface(|interface| interface.network_interface().add_socket(socket));

        UdpSocket {
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
            let res = self.network.with_interface(|interface| {
                let (sock, cx) = interface
                    .network_interface()
                    .get_socket_and_context::<TcpSocket>(self.socket_handle);
                let remote_endpoint = (addr, port);
                sock.connect(cx, remote_endpoint, self.network.next_local_port())
            });

            if let Err(err) = res {
                return Err(err.into());
            }
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

impl<'s, 'n: 's> Drop for Socket<'s, 'n> {
    fn drop(&mut self) {
        self.network.with_interface(|interface| {
            interface
                .network_interface
                .remove_socket(self.socket_handle)
        });
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
            let res = self.network.with_interface(|interface| {
                interface
                    .network_interface()
                    .poll(Instant::from_millis(
                        (self.network.current_millis_fn)() as i64
                    ))
            });

            if let Err(err) = res {
                return Err(err.into());
            }

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
            let res = self.network.with_interface(|interface| {
                interface
                    .network_interface()
                    .poll(Instant::from_millis(
                        (self.network.current_millis_fn)() as i64
                    ))
            });

            if let Err(err) = res {
                return Err(err.into());
            }

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

pub struct UdpSocket<'s, 'n: 's> {
    socket_handle: SocketHandle,
    network: &'s Network<'n>,
}

impl<'s, 'n: 's> UdpSocket<'s, 'n> {
    pub fn bind<'i>(&'i mut self, port: u16) -> Result<(), IoError>
    where
        's: 'i,
    {
        self.work();

        {
            let res = self.network.with_interface(|interface| {
                let (sock, _cx) = interface
                    .network_interface()
                    .get_socket_and_context::<smoltcp::socket::UdpSocket>(self.socket_handle);
                sock.bind(port)
            });

            if let Err(err) = res {
                return Err(err.into());
            }
        }

        loop {
            let can_send = self.network.with_interface(|interface| {
                let sock = interface
                    .network_interface()
                    .get_socket::<smoltcp::socket::UdpSocket>(self.socket_handle);
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

    pub fn close(&mut self) {
        self.network.with_interface(|interface| {
            interface
                .network_interface()
                .get_socket::<smoltcp::socket::UdpSocket>(self.socket_handle)
                .close();
        });

        self.work();
    }

    pub fn send(&mut self, addr: Ipv4Address, port: u16, data: &[u8]) -> Result<(), IoError> {
        loop {
            self.work();

            let (can_send, packet_capacity, payload_capacity) =
                self.network.with_interface(|interface| {
                    let sock = interface
                        .network_interface()
                        .get_socket::<smoltcp::socket::UdpSocket>(self.socket_handle);
                    (
                        sock.can_send(),
                        sock.packet_send_capacity(),
                        sock.payload_send_capacity(),
                    )
                });

            if can_send && packet_capacity > 0 && payload_capacity > data.len() {
                break;
            }
        }

        let res = self.network.with_interface(|interface| {
            let endpoint = (addr, port);

            interface
                .network_interface()
                .get_socket::<smoltcp::socket::UdpSocket>(self.socket_handle)
                .send_slice(data, endpoint.into())
        });

        self.work();

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.into()),
        }
    }

    pub fn receive(&mut self, data: &mut [u8]) -> Result<(usize, [u8; 4], u16), IoError> {
        self.work();

        let res = self.network.with_interface(|interface| {
            interface
                .network_interface()
                .get_socket::<smoltcp::socket::UdpSocket>(self.socket_handle)
                .recv_slice(data)
        });

        match res {
            Ok((len, endpoint)) => {
                let addr = match endpoint.addr {
                    IpAddress::Unspecified => todo!(),
                    IpAddress::Ipv4(ipv4) => ipv4,
                    _ => todo!(),
                };
                Ok((len, addr.0, endpoint.port))
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn join_multicast_group(&mut self, addr: Ipv4Address) -> Result<bool, IoError> {
        self.work();

        let res = self.network.with_interface(|interface| {
            interface.network_interface().join_multicast_group(
                addr,
                Instant::from_millis((self.network.current_millis_fn)() as i64),
            )
        });

        self.work();

        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.into()),
        }
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

impl<'s, 'n: 's> Drop for UdpSocket<'s, 'n> {
    fn drop(&mut self) {
        self.network.with_interface(|interface| {
            interface
                .network_interface
                .remove_socket(self.socket_handle)
        });
    }
}
