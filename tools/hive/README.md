# Apply Ethean onto an ethereum/hive checkout

## Preconditions (operator)

1. `gh auth login` (repo + package scopes; needed for the ethereum/hive PR).
2. Public GHCR base image:
   ```powershell
   .\tools\hive\check-ghcr.ps1
   ```
   Anonymous pull of `:devnet5` must succeed. If the package is private, flip
   visibility under GitHub Packages for `ethean-labs/ethean`, or re-run the
   Docker workflow after the “Ensure GHCR package is public” step is allowed.
3. To publish `:devnet5` without a new `v*` tag: Actions → Docker image →
   Run workflow → `extra_tags=devnet5,latest-devnet5`.
4. Docker Desktop (or a Linux Docker daemon) for a local `./hive` smoke.

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

## After apply

```bash
cd /path/to/hive
git checkout -b clients/ethean
git add clients/ethean simulators/lean
git commit -m "clients: add ethean lean participant"
gh repo fork ethereum/hive --remote=true   # once
git push -u origin HEAD
gh pr create --repo ethereum/hive --title "clients: add ethean" --body-file /path/to/ethean/tools/hive/PR_BODY.md
```

Do **not** invent A2 digests / A3 bootnodes; those stay operator-paste.
