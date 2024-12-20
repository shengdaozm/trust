extern crate tun_tap;

use std::collections::HashMap;
use std::io;
use std::net::Ipv4Addr;
mod tcp;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

fn main() -> io::Result<()> {
    let mut connections: HashMap<Quad, tcp::Connection> = Default::default();
    let mut nic = tun_tap::Iface::new("rust_tun0", tun_tap::Mode::Tun)?;
    let mut buf = [0u8; 1504];
    for _ in 0..20 {
        let nbytes = nic.recv(&mut buf[..])?;
        // let eth_flags = u16::from_be_bytes([buf[0], buf[1]]);
        let eth_proto = u16::from_be_bytes([buf[2], buf[3]]);
        if eth_proto != 0x0800 {
            // not ipv4
            // ipv4 => 0x0800
            // ipv6 => 0x86dd
            continue;
        }

        match etherparse::Ipv4HeaderSlice::from_slice(&buf[4..nbytes]) {
            Ok(iph) => {
                let src = iph.source_addr();
                let dst = iph.destination_addr();
                if iph.protocol() != 0x6 {
                    // not tcp
                    // icmp => 0x1 tcp => 0x6 udp => 0x11
                    continue;
                }

                match etherparse::TcpHeaderSlice::from_slice(&buf[4 + iph.slice().len()..nbytes]) {
                    Ok(tcph) => {
                        use std::collections::hash_map::Entry;
                        let datai = 4 + iph.slice().len() + tcph.slice().len();
                        match connections.entry(Quad {
                            src: (src, tcph.source_port()),
                            dst: (dst, tcph.destination_port()),
                        }) {
                            Entry::Occupied(mut c) => {
                                c.get_mut()
                                    .on_packet(&mut nic, iph, tcph, &buf[datai..nbytes]);
                            }
                            Entry::Vacant(mut e) => {
                                if let Some(c) = tcp::Connection::accept(
                                    &mut nic,
                                    iph,
                                    tcph,
                                    &buf[datai..nbytes],
                                )? {
                                    e.insert(c);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("ignoring weird tcp packet {:?}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("ignoring weird packet {:?}", e);
            }
        }
    }
    Ok(())
}
