# fdb-sim-visualizer

## Generate traces

```bash
fdbserver -r simulation -f /root/logical_db.toml -b on --trace-format json -L ./logs
```

## associated run for combiend_traces 

```
[root@be95159cfa98 ~]# fdbserver -r simulation -f /root/logical_db.toml -b on --trace-format json -L ./logs/
Random seed is 292006968...
DeterministicRandom successfully bound to OpenSSL RNG
Datacenter 0: 3/9 machines, 1/1 coordinators
sim http machines = 2
Datacenter 1: 3/9 machines, 0/1 coordinators
sim http machines = 2
Datacenter 2: 3/9 machines, 0/1 coordinators
sim http machines = 3
Process 2.1.1.4 run http server
SimHTTPServer protecting 2.1.1.4:1:tls
Process 2.2.1.3 run http server
Process 2.0.1.5 run http server
Process 2.2.1.5 run http server
Process 2.2.1.4 run http server
Process 2.0.1.4 run http server
Process 2.1.1.3 run http server
FDBD joined cluster.
FDBD joined cluster.
FDBD joined cluster.
FDBD joined cluster.
FDBD joined cluster.
FDBD joined cluster.
FDBD joined cluster.
FDBD joined cluster.
FDBD joined cluster.
FDBD joined cluster.
FDBD joined cluster.
FDBD joined cluster.
FDBD joined cluster.
FDBD joined cluster.
FDBD joined cluster.
FDBD joined cluster.
startingConfiguration:new backup_worker_enabled:=0 blob_granules_enabled:=0 commit_proxies:=4 encryption_at_rest_mode=disabled grv_proxies:=1 log_engine=ssd-2 log_spill:=1 log_version:=6 logs:=3 perpetual_storage_wiggle:=0 perpetual_storage_wiggle_engine=none proxies:=5 three_data_hall resolvers:=1 storage_engine=memory storage_migration_type=disabled tenant_mode=disabled usable_regions:=1 start
useDB: true
Simulated Cluster Topology:
===========================
    zoneId: 1585c5e2c8e9bac8cec260ae4b36f57b
      machineId: 1585c5e2c8e9bac8cec260ae4b36f57b
        Address: 3.4.3.3:1:tls
          Class: test
          Name: Server
    zoneId: 4612dee32b7ff83efde45b703bc8521a
      machineId: 4612dee32b7ff83efde45b703bc8521a
        Address: 3.4.3.2:1:tls
          Class: test
          Name: Server
    zoneId: 54147ceac841dcd19ebc4248bd5fcc84
      machineId: 387529a9f3956ac569e86bce971892c3
        Address: 1.1.1.1:1
          Class: test
          Name: TestSystem
    zoneId: bf1ce36475b25f43748ac689fa58600c
      machineId: bf1ce36475b25f43748ac689fa58600c
        Address: 3.4.3.6:1:tls
          Class: test
          Name: Server
    zoneId: ce147ab758d105d7db6164509b555f1c
      machineId: ce147ab758d105d7db6164509b555f1c
        Address: 3.4.3.5:1:tls
          Class: test
          Name: Server
    zoneId: d74b5123945df0ea35d83a274391a30d
      machineId: d74b5123945df0ea35d83a274391a30d
        Address: 3.4.3.1:1:tls
          Class: test
          Name: Server
    zoneId: dd761c5085f0bd96158179054d82df1e
      machineId: dd761c5085f0bd96158179054d82df1e
        Address: 3.4.3.4:1:tls
          Class: test
          Name: Server
dcId: 0
  dataHallId: 0
    zoneId: 0f12bbdbf2c49d14bd0388a344101846
      machineId: 25acda3f10d0edab6db5ed5464b34380
        Address: 2.0.1.4:1:tls
          Class: sim_http_server
          Name: Server
    zoneId: 201ce834c40ac491efe00c8047ad8f7e
      machineId: 9035d3efed7cabe757e02a961660698a
        Address: 2.0.1.2:1:tls
          Class: unset
          Name: Server
    zoneId: 20fc497ed5a1efc4ae829f4b2f4486b3
      machineId: e4a5cec0b954157cc11edea9e5e3ee80
        Address: 2.0.1.1:1:tls
          Class: unset
          Name: Server
    zoneId: 2166b709e030c552f8645d2ed3d5c5a6
      machineId: b174414ad2867246962ebde207f6e58d
        Address: 2.0.1.0:1:tls
          Class: storage
          Name: Server
    zoneId: 875664bb2f271e160da7ce3dc8a38d22
      machineId: 58fd19df6885150fc2ce0972bb90b6db
        Address: 2.0.1.5:1:tls
          Class: sim_http_server
          Name: Server
    zoneId: 88275d1a659ad8747ea3adc1b313af9b
      machineId: 40eb69e0d1320418901a02883687acd8
        Address: 2.0.1.3:1:tls
          Class: storage_cache
          Name: Server
dcId: 1
  dataHallId: 1
    zoneId: 3d3261361cbf89068db3d96ca97d2af4
      machineId: 36d5a8751d21f25294f2707515a85606
        Address: 2.1.1.2:1:tls
          Class: unset
          Name: Server
    zoneId: a51d041b5031a7bba614a2577aed7d3f
      machineId: 59cad75a2e8093db9fe40aac67778906
        Address: 2.1.1.0:1:tls
          Class: unset
          Name: Server
    zoneId: b8d86fa3f212f07aa3fa86aefa61e12f
      machineId: 30fc009a2f52e5dc3b9b6a1cf1d7cd84
        Address: 2.1.1.3:1:tls
          Class: sim_http_server
          Name: Server
    zoneId: dc017f137e02580e4526e92faa9564f0
      machineId: ca7d84cbe4e4de55127ea3469ff09a31
        Address: 2.1.1.4:1:tls
          Class: sim_http_server
          Name: Server
    zoneId: f6c615c867374bef693a266cf61f1e6c
      machineId: 73f82561dc6eae8e271f71dd4db3eecc
        Address: 2.1.1.1:1:tls
          Class: transaction
          Name: Server
dcId: 2
  dataHallId: 2
    zoneId: 42ad003e36e2067cbaa8edee0d74a4f1
      machineId: 65dc1b4ea438738b36c108dceca3aad2
        Address: 2.2.1.4:1:tls
          Class: sim_http_server
          Name: Server
    zoneId: 4d88221754c462f2ed68a1729de0b061
      machineId: 5fe8ec5f4b3c1e6017a38b969bc6eca1
        Address: 2.2.1.0:1:tls
          Class: storage
          Name: Server
    zoneId: 5501e69e8a0ea2798cc81402e43b7afd
      machineId: 6007f5de16dd81a0e2a9d642e3775235
        Address: 2.2.1.3:1:tls
          Class: sim_http_server
          Name: Server
    zoneId: af0447e31d137a87afdea1539e414288
      machineId: 5835a37723cc1d589117ac73d898ed97
        Address: 2.2.1.2:1:tls
          Class: transaction
          Name: Server
    zoneId: eb40f6712ff5dea0430fbcbe921ace42
      machineId: b635b71274fa3e812bd16395116067a4
        Address: 2.2.1.5:1:tls
          Class: sim_http_server
          Name: Server
    zoneId: f287d5d194223cc41854d3bea4b41666
      machineId: 7b8793ccadb0f248783c9dbe11cd28f8
        Address: 2.2.1.1:1:tls
          Class: transaction
          Name: Server
```