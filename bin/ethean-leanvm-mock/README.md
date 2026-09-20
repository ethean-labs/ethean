# ethean-leanvm-mock

Dev/CI helper that speaks the Ethean leanVM IPC dialect:

- stdin/stdout: `u32` LE length + ELVM frame (`ethean-crypto`)
- ProveRequest → test-aggregate proof (`ProveResponse`)
- VerifyRequest → statement-bound verify (`VerifyResponse`)

Not a production leanVM / Plonky3 prover. Use only for local spawn round-trips:

```powershell
cargo build -p ethean-leanvm-mock
$env:ETHEAN_LEANVM_PROVER = "$PWD\target\debug\ethean-leanvm-mock.exe"
```
