//! leanMultisig prover process.
//!
//! Reads length-prefixed requests on stdin, writes responses on stdout and
//! diagnostics on stderr. Requests are served one at a time; the prover
//! parallelises each proof internally. Exits cleanly when stdin closes.

use std::io::{BufReader, BufWriter};
use std::time::Instant;

use ethean_multisig::protocol::{read_frame, write_frame, Request, Response};
use ethean_multisig::prove::{aggregate_type1, init_prover_process, merge_type2, split_type2};
use ethean_multisig::LEANVM_REV;

fn serve(request: Request) -> Response {
    let result = match request {
        Request::Ping => return Response::Pong(LEANVM_REV.to_string()),
        Request::AggregateType1 {
            message,
            slot,
            raw,
            children,
        } => aggregate_type1(&children, &raw, &message, slot),
        Request::MergeType2 { components } => merge_type2(&components),
        Request::SplitType2 {
            message,
            proof,
            public_keys_per_component,
        } => split_type2(&proof, &public_keys_per_component, &message),
    };
    match result {
        Ok(proof) => Response::Proof(proof),
        Err(e) => Response::Error(e.to_string()),
    }
}

fn kind(request: &Request) -> &'static str {
    match request {
        Request::Ping => "ping",
        Request::AggregateType1 { .. } => "aggregate",
        Request::MergeType2 { .. } => "merge",
        Request::SplitType2 { .. } => "split",
    }
}

/// Proof generation recurses deeply; debug builds overflow the default 8 MiB
/// main-thread stack, so the serve loop runs on a thread with this stack.
const SERVE_STACK_BYTES: usize = 256 * 1024 * 1024;
/// Default stack for the worker threads leanVM spawns internally.
const WORKER_STACK_BYTES: usize = 64 * 1024 * 1024;

fn main() {
    if std::env::args().any(|a| a == "--version") {
        println!(
            "ethean-prover {} (leanVM {LEANVM_REV})",
            env!("CARGO_PKG_VERSION")
        );
        return;
    }
    // Rust reads RUST_MIN_STACK once, at the first thread spawn, so set it
    // before any thread exists (unless the operator chose a value).
    if std::env::var_os("RUST_MIN_STACK").is_none() {
        std::env::set_var("RUST_MIN_STACK", WORKER_STACK_BYTES.to_string());
    }
    // Warm the prover (bytecode, DFT twiddles) while the handshake proceeds.
    let _warmup = std::thread::Builder::new()
        .name("prover-warmup".into())
        .stack_size(SERVE_STACK_BYTES)
        .spawn(init_prover_process);
    let server = std::thread::Builder::new()
        .name("prover-serve".into())
        .stack_size(SERVE_STACK_BYTES)
        .spawn(serve_loop)
        .expect("spawn prover serve thread");
    if server.join().is_err() {
        std::process::exit(3);
    }
}

fn serve_loop() {
    let mut input = BufReader::new(std::io::stdin().lock());
    let mut output = BufWriter::new(std::io::stdout().lock());
    loop {
        let frame = match read_frame(&mut input) {
            Ok(Some(frame)) => frame,
            Ok(None) => return,
            Err(e) => {
                eprintln!("ethean-prover: read failed: {e}");
                std::process::exit(2);
            }
        };
        let (id, response) = match Request::decode(&frame) {
            Ok((id, request)) => {
                let started = Instant::now();
                let label = kind(&request);
                let response = serve(request);
                if label != "ping" {
                    let outcome = if matches!(response, Response::Proof(_)) {
                        "ok"
                    } else {
                        "error"
                    };
                    eprintln!(
                        "ethean-prover: {label} {outcome} in {:?}",
                        started.elapsed()
                    );
                }
                (id, response)
            }
            Err(e) => (0, Response::Error(format!("malformed request: {e}"))),
        };
        if let Err(e) = write_frame(&mut output, &response.encode(id)) {
            eprintln!("ethean-prover: write failed: {e}");
            std::process::exit(2);
        }
    }
}
