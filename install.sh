#!/bin/bash
#check root
if [ "$EUID" -ne 0 ]
  then echo "install.sh must be run as root"
  exit
fi

#create user
useradd -r -s /sbin/nologin yarad
groupadd yarad

#create directories
mkdir -p /etc/yarad
mkdir -p /var/run/yarad
mkdir -p /var/lib/yarad/rules

#write CONFIG_TEMPLATE into config.yml
echo """
# log level [error|warn|info|debug|trace]
log_level: warn
# socket for communication with yara daemon
local_socket: /var/run/yarad/yarad.ctl
local_socket_group: yarad
local_socket_mode: 0o666
# yara rule files directory
rules_dir: /var/lib/yarad/rules
# user and working directory for running yarad
user: yarad
working_dir: /var/run/yarad
auto_recompile_rules: true
daemonize: true
"""> /etc/yarad/config.yml
chown yarad:yarad /etc/yarad/config.yml
chmod 640 /etc/yarad/config.yml

