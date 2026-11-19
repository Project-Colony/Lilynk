# @lilynk/host

TypeScript client for the [Lilynk](https://github.com/Project-Colony/Lilynk) host: every
management-API endpoint typed, plus the host's event stream with reconnect and resume. Built on
[Effect](https://effect.website); you don't need to know Effect to use it.

## Install

The package is on the unom npm registry. Point the `@lilynk` scope at it once, in `.npmrc`:

```ini
@lilynk:registry=https://git.unom.io/api/packages/unom/npm/
```

```sh
bun add @lilynk/host effect     # or: npm i @lilynk/host effect
```

`effect` is a peer dependency, so your code and the SDK share one copy.

## Usage

On the host box `connect()` needs no configuration: it reads the URL, token and certificate the
host writes to its config directory.

```ts
import { connect } from "@lilynk/host";

const pf = await connect();
const clients = await pf.api.listPairedClients();
console.log(`${clients.length} paired clients`);

pf.events.on("stream.started", (e) => {
  console.log(`${e.stream.client} started ${e.stream.mode}`);
});
```

`@lilynk/host/effect` exposes the same client as Effect services and streams, and
`@lilynk/host/core` is the Node-free build for a browser.

## Docs

- [Management API](https://project-colony.github.io/Lilynk/docs/developers/management-api): credentials,
  how `connect()` resolves them, the entry points, and the OpenAPI reference.
- [Writing plugins](https://project-colony.github.io/Lilynk/docs/developers/writing-plugins): run your code
  under the host's plugin runner.
- [Host events](https://project-colony.github.io/Lilynk/docs/host-events): every event kind and its fields.
- [`examples/`](./examples): runnable scripts, from tailing events to USB passthrough.

## Development

```sh
bun install
bun run gen        # regenerate src/gen/lilynk.ts from ../api/openapi.json
bun run typecheck
bun test
```

Tag `sdk-vX.Y.Z` (matching `package.json`) to publish.
