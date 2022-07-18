# AutoRouter

Find out non-ACKed TCP syn request packet in realtime network traffic and add the destination IP address into a ipset.


## build and copy

```shell
apt-get install libnetfilter-queue-dev libnfnetlink-dev -y    # install libs
cargo build --release    # compile application
cp target/release/auto-router config.yml /root/auto-router/    # copy executable and config file to /root/auto-router/
cp auto-router.service /etc/systemd/system/    # copy systemd service config file
```


## setup redis server

Uncomment `notify-keyspace-events` in redis server config file and set the value to `"Ex"`.

Replace `bind 127.0.0.1 ::1` to `0.0.0.0 ::` if required.


## configuration

**config.yml**

Change `redis_server` to redis server, `ipset` to ipset name, `nfq_id` to netfilter queue id.

**auto-router.service**

Change `ExecStart`, `WorkingDirectory`, and other if need.


## reload and start service

```
systemctl daemon-reload
systemctl enable --now auto-router.service
ps -ef | grep auto-router
```


## setup iptables to redirect packet to AutoRouter

```shell
#// replace "ens34" to WAN interface, queue-num to config file.
iptables -I FORWARD -i ens34 -p tcp -m tcp --tcp-flags SYN,ACK SYN,ACK -j NFQUEUE --queue-num 10  # add SYN,ACK in packet to queue
iptables -I FORWARD -o ens34 -p tcp -m tcp --tcp-flags SYN,ACK SYN -j NFQUEUE --queue-num 10  # add SYN out packet to queue
#// If you also want to monitoring local network traffic, add them to INPUT and OUTPUT chain.
#// Now, it is working!
```
