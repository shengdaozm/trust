use std::io;

pub enum State {
    Closed,
    Listen,
    SynRcvd,
    Estab,
}
impl Default for State {
    fn default() -> Self {
        State::Listen
    }
}

impl State {
    pub fn on_packet<'a>(
        &mut self,
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice<'a>,
        tcph: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) -> io::Result<(usize)> {
        let mut buf = [0u8; 1500];
        match *self {
            State::Closed => {
                return;
            }
            State::Listen => {
                if tcph.syn() {
                    //not expected SYN packet
                    return;
                }

                //need to establish connection
                let mut syn_ack = etherparse::TcpHeader::new(
                    tcph.destination_port(),
                    tcph.source_port(),
                    unimplemented!(),
                    unimplemented!(),
                );
                syn_ack.syn = true;
                syn_ack.ack = true;
                let mut ip = etherparse::Ipv4Header::new(
                    syn_ack.total_length(),
                    64,
                    etherparse::IpTrafficClass::Tcp,
                    iph.destination(),
                    iph.source(),
                );
                // write the packet to the buffer
                let mut unwritten = {
                    let mut unwritten = &mut buf[..];
                    ip.write(unwritten);
                    syn_ack.write(unwritten);
                    unwritten.len()
                };
                nic.send(&buf[..unwritten])?;
            }
            State::SynRcvd => {}
            State::Estab => {}
        }
    }
}
