# Source file size

Keep authored code files at 300 lines or less. When a module grows past that, split it by concern and re-export from `mod.rs` so the public path stays stable. This is a modularity rule, not a style hobby: large files are where consensus, crypto, and networking accidentally couple.
