#!/bin/sh
cargo b --release
# if compile failed, exit
ext=$?
if [ $ext -ne 0 ]; then
    exit $ext
fi
sudo setcap cap_net_admin=eip ./target/release/trust
./target/release/trust &

pid=$!

sudo ip addr add 192.168.0.1/24 dev rust_tun0
sudo ip link set up dev rust_tun0

# if kill the sh process, kill the rust process
trap "kill $pid" INT TERM

wait $pid


