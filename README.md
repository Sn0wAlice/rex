# rex
Rex is a multi tool for SOC analyst


# Features
- [x] SSL Dump
- [x] PDF Extractor
- [x] Network realtime logs
- [x] Reg Systemd extract
- [x] Reg Systemd scan (issues)
- [x] Reg Systemd deep scan
- [x] Reg Systemd sshfail

# Installation
```bash
git clone https://github.com/Sn0wAlice/rex
cd rex
cargo build
cp target/debug/rex /usr/local/bin
```

## Linux dependencies
```bash
apt install libssl-dev
apt install libpcap-dev
```