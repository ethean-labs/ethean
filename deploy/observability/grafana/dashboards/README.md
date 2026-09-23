# Grafana dashboards

- `ethean-lean-clients.json`, `ethean-node-health.json`: Ethean dashboards
  over the `ethean_*` families.
- `lean-ethereum-client-interop.json`: the shared leanMetrics interop
  dashboard (leanEthereum/leanMetrics `dashboards/`, commit 69f9722) over the
  standard `lean_*` families. Local changes: the datasource UID is remapped to
  the provisioned `prometheus` datasource, `${datasource}` references use the
  dashboard's `Datasource` variable, and `ethean` is added to the `job`
  variable's client filter (upstream lists only the clients already tracked).
