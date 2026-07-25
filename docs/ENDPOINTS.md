## Priority implement for monitoring and teleoperation:
/session/status -> get current session for reauth with session auth

Site Manager group endpoints

/modems/signal/status -> get singal statues of multiple modems
/modems/signal/status/{id} -> get signal status of one modem -> herer it returns data for timestamp


/modems/apns/status -> something related to operator

/data_usage/{interval}/modem/{modem_id}/status -> here is the interval is string data enum

/internet_connection/status

GPS

## For development purposes

/modems/{id}/actions/scan_network -> to scan network
/modems/{id}/actions/reboot -> in case something happened to modem and we can't get the access to it
/modems/{id}/actions/restart_connection -> the same problem
/system/device/status

### Logging:
/events_log/config
/events_log/config/{event_type}
Traffic Logging -> get what info can be logged there
Logging

### For optimization:
QoS
NAT offloading -> method for packet transfer speed increase by offloading from modem cpu to some shit 
OSPF -> dynamic best path traffic finder
DFOTA -> some methods for utra thin data transfer for battery and network optimisation
SQM -> smart queue manager 
VRF -> add to be at prod level when network isolation is required 
Backup

## Groups and endpoints to consider

/modems/scan/status -> operator scan status -> idk if it is really useful
/modems/scan/status/{id}

Wifi scanner -> why not

What the fuck is DATA to SERVER there

/recipients/phone_groups/config (get, post, put, delete) -> when not able to tranfer data via internet network
/recipients/phone_groups/config/{id} (get, post, put, delete) -> when not able to tranfer data via internet network

Messages -> to send messages in case of problem with internet

Interfaces

Speedtest

Date&Time
