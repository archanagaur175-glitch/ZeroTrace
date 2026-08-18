//! Local interface enumeration via GetAdaptersAddresses.

use crate::core::ZtError;
use crate::trust::models::LocalInterface;
use windows::Win32::NetworkManagement::IpHelper::{
    GetAdaptersAddresses, IP_ADAPTER_ADDRESSES_LH, IP_ADAPTER_UNICAST_ADDRESS_LH,
};

pub fn local_interfaces() -> crate::core::Result<Vec<LocalInterface>> {
    unsafe {
        let mut size: u32 = 0;
        let rc = GetAdaptersAddresses(0, Default::default(), None, None, &mut size);
        if rc != 0 && rc != 111 {
            return Err(ZtError::new(format!("GetAdaptersAddresses rc={rc}")));
        }
        let mut buf = vec![0u8; size as usize];
        let rc = GetAdaptersAddresses(
            0,
            Default::default(),
            None,
            Some(buf.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH),
            &mut size,
        );
        if rc != 0 {
            return Err(ZtError::new(format!("GetAdaptersAddresses rc={rc}")));
        }

        let mut out = Vec::new();
        let mut cur = buf.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;
        while !cur.is_null() {
            let adapt = &*cur;
            let name = read_pwstr(adapt.FriendlyName);
            let mut addresses = Vec::new();
            let mut ua = adapt.FirstUnicastAddress;
            while !ua.is_null() {
                let unicast = &*ua;
                addresses.push(addr_from_socket(unicast));
                ua = unicast.Next;
            }
            out.push(LocalInterface {
                name,
                description: format!("iftype {}", adapt.IfType),
                addresses,
            });
            cur = adapt.Next;
        }
        Ok(out)
    }
}

fn read_pwstr(p: windows::core::PWSTR) -> String {
    if p.is_null() {
        return String::new();
    }
    unsafe {
        let len = (0..).take_while(|&i| *p.as_ptr().add(i) != 0).count();
        let slice = std::slice::from_raw_parts(p.as_ptr(), len);
        String::from_utf16_lossy(slice)
    }
}

fn addr_from_socket(unicast: &IP_ADAPTER_UNICAST_ADDRESS_LH) -> String {
    let sa = &unicast.Address;
    unsafe {
        if sa.lpSockaddr.is_null() {
            return String::new();
        }
        let bytes =
            std::slice::from_raw_parts(sa.lpSockaddr as *const u8, sa.iSockaddrLength as usize);
        if bytes.len() < 2 {
            return String::new();
        }
        let family = u16::from_ne_bytes([bytes[0], bytes[1]]);
        match family {
            2 => {
                // AF_INET — address at offset 4.
                let mut a = [0u8; 4];
                if bytes.len() >= 8 {
                    a.copy_from_slice(&bytes[4..8]);
                }
                format!("{}.{}.{}.{}", a[0], a[1], a[2], a[3])
            }
            23 => {
                // AF_INET6 — address at offset 8.
                let mut b = [0u8; 16];
                if bytes.len() >= 24 {
                    b.copy_from_slice(&bytes[8..24]);
                }
                crate::network::sockets::fmt_ipv6(&b)
            }
            _ => String::new(),
        }
    }
}
