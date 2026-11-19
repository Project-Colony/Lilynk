# @lilynk/plugin-kit

The framework Lilynk host plugins are built on. It owns what every plugin shares: lifecycle and
shutdown, config and state files, the library sync loop, the console page, the CLI and logging.
Your plugin is its domain logic. Built on [`@lilynk/host`](../sdk) and
[Effect](https://effect.website).

## Install

The package is on the unom npm registry. Point the `@lilynk` scope at it once, in `.npmrc`:

```ini
@lilynk:registry=https://git.unom.io/api/packages/unom/npm/
```

```sh
bun add @lilynk/plugin-kit @lilynk/host effect
```

`effect` and `@lilynk/host` are peer dependencies; `react` is an optional one for
`@lilynk/plugin-kit/react`. Declare the SDK yourself: when an older plugin's kit owns the
host's plugins folder, bun places no SDK for yours, and it fails to start.

## Usage

A library source is a scan function; `defineLibraryPlugin` adds the rest:

```ts
import { Effect, Schema } from "effect";
import { defineLibraryPlugin } from "@lilynk/plugin-kit/library";

export const plugin = defineLibraryPlugin({
  name: "hello",
  configSchema: Schema.Struct({}),
  detect: () => Effect.succeed(true),
  scan: () => Effect.succeed([{ external_id: "1", title: "Hello World" }]),
});

export default plugin.def; // what the runner loads; `plugin.cli()` is the CLI entry
```

An Art & Metadata source fills art and details for games other plugins list. It is its matching
and fetching; `defineMetadataPlugin` reads the library, caches what it found, pushes the result and
serves the console's Choose dialog:

```ts
import { defineMetadataPlugin } from "@lilynk/plugin-kit/metadata";

export const plugin = defineMetadataPlugin({
  name: "covers",
  configSchema: Schema.Struct({}),
  matching: "search", // "exact" if you only match by entry ids
  offers: { art: ["portrait"] },
  match: (entry) => Effect.succeed({ key: entry.title, label: entry.title }),
  fetch: (match) => Effect.succeed({ art: { portrait: `https://example.com/${match.key}.png` } }),
});
```

Any other plugin uses `definePluginKit({ name, layer, main })`, and `serveUi(...)` for a page in the
web console, a settings form (`config`) or a tab on each library entry's page (`game`, with
`handedPath()` for folders the operator types in).

| Import | Holds |
|---|---|
| `@lilynk/plugin-kit` | `definePluginKit`, config and cache stores, the sync engine, `serveUi`, `requestAccess`, the CLI scaffold |
| `@lilynk/plugin-kit/library` | `defineLibraryPlugin`, parsers for VDF, SQLite and the Windows registry, the parity check |
| `@lilynk/plugin-kit/metadata` | `defineMetadataPlugin`, the rules for which entries a source looks up |
| `@lilynk/plugin-kit/wire` | The library wire schemas: `ProviderEntry`, `LaunchSpec` and its launch kinds |
| `@lilynk/plugin-kit/react` | Browser helpers for a plugin's console page |
| `@lilynk/plugin-kit/theme.css` | The console's colours for a plugin's page |

## Docs

[Writing plugins](https://project-colony.github.io/Lilynk/docs/developers/writing-plugins) covers the
manifest, the sandbox, a complete minimal plugin, testing on a host and publishing.
[`lilynk-plugin-steam`](https://git.unom.io/unom/punktfunk-plugin-steam) and
[`lilynk-plugin-rom-manager`](https://git.unom.io/unom/punktfunk-plugin-rom-manager) are real
plugins to copy.

## Development

```sh
bun install
bun run check
bun run typecheck
bun test
```

Tag `plugin-kit-vX.Y.Z` (matching `package.json`) to publish.
