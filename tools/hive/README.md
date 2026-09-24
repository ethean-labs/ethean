# Apply Ethean onto an ethereum/hive checkout

## PowerShell (Windows)

```powershell
# From the Ethean repo root:
.\tools\hive\apply-ethean-client.ps1 -HiveRoot G:\path\to\hive
```

## Bash (Linux / WSL)

```bash
./tools/hive/apply-ethean-client.sh /path/to/hive
```

Both are idempotent. They copy `docker/hive/upstream-clients-ethean/` and
patch lean-devnets / client yaml / util.rs / prepare_lean_client_assets.py.

Verified against a shallow `ethereum/hive` clone under local `bazalinacaklar/`
(gitignored). PR text: [PR_BODY.md](./PR_BODY.md).
