use anyhow::{Result, Context};
use pcap::{Capture, Device};

pub fn logs_network() -> Result<()> {
    let device = Device::lookup()
        .context("Failed to look up network device")?
        .context("No network device found")?;
    let device_name = device.name;

    let mut cap = Capture::from_device(device_name.as_str())
        .context("Failed to create capture from device")?
        .promisc(true)
        .open()
        .context("Failed to open capture")?;

    println!("Listening for packets...");

    while let Ok(packet) = cap.next_packet() {
        let data = packet.data;

        if data.len() < 14 {
            continue;
        }

        let ethertype = u16::from_be_bytes([data[12], data[13]]);

        match ethertype {
            0x0806 => {
                println!("ARP packet detected");
            }
            0x0800 => {
                let ip_header_start = 14;
                if data.len() < ip_header_start + 20 {
                    continue;
                }

                let protocol = data[ip_header_start + 9];
                let src_ip = format!(
                    "{}.{}.{}.{}",
                    data[ip_header_start + 12],
                    data[ip_header_start + 13],
                    data[ip_header_start + 14],
                    data[ip_header_start + 15]
                );
                let dst_ip = format!(
                    "{}.{}.{}.{}",
                    data[ip_header_start + 16],
                    data[ip_header_start + 17],
                    data[ip_header_start + 18],
                    data[ip_header_start + 19]
                );

                let ip_header_length = (data[ip_header_start] & 0x0F) * 4;
                let transport_start = ip_header_start + ip_header_length as usize;

                match protocol {
                    1 => {
                        println!("ICMP packet from {} to {}", src_ip, dst_ip);
                    }
                    6 => {
                        if data.len() >= transport_start + 4 {
                            let src_port = u16::from_be_bytes([
                                data[transport_start],
                                data[transport_start + 1],
                            ]);
                            let dst_port = u16::from_be_bytes([
                                data[transport_start + 2],
                                data[transport_start + 3],
                            ]);
                            println!(
                                "TCP packet from {}:{} to {}:{}",
                                src_ip, src_port, dst_ip, dst_port
                            );
                        }
                    }
                    17 => {
                        if data.len() >= transport_start + 4 {
                            let src_port = u16::from_be_bytes([
                                data[transport_start],
                                data[transport_start + 1],
                            ]);
                            let dst_port = u16::from_be_bytes([
                                data[transport_start + 2],
                                data[transport_start + 3],
                            ]);
                            println!(
                                "UDP packet from {}:{} to {}:{}",
                                src_ip, src_port, dst_ip, dst_port
                            );
                        }
                    }
                    _ => {
                        println!("IPv4 packet with unknown protocol ({})", protocol);
                    }
                }
            }
            0x86DD => {
                println!("IPv6 packet (not parsed in detail)");
            }
            _ => {
                println!("Unknown EtherType: 0x{:04x}", ethertype);
            }
        }
    }

    Ok(())
}
