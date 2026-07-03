# p2p-tunnel

Instantly share a localhost port over P2P using Iroh.

## Features

- Share a local service without manual port forwarding.
- Connect from another machine using a generated ticket.
- Cross-platform release artifacts generated with `cargo-dist`.
- npm installer support for easy install by platform.

## Usage

Build and run from source:

```bash
cargo run -- share 3000
```

Run with `npx` (no global install):

```bash
npx p2p-tunnel share 3000
```

On another machine (or another terminal), connect using the ticket printed by `share`:

```bash
cargo run -- connect <TICKET>
```

Or with `npx`:

```bash
npx p2p-tunnel connect <TICKET>
```

The current connect command binds locally to `127.0.0.1:8080`.

## Installers and Releases

This repository uses `cargo-dist` with npm installers enabled.

Configured npm package name:

- `p2p-tunnel`

Typical release workflow:

```bash
dist build
dist generate-ci
```

## License

MIT. See [LICENSE](LICENSE).
