---
title: Browser Client
description: The browser client is punktfunk's and doesn't connect to a Lilynk host.
---

punktfunk's browser client, [client-web](https://github.com/punktfunk/client-web), streams to a
browser tab over WebTransport and WebCodecs. It speaks punktfunk's protocol and doesn't connect to a
Lilynk host. Lilynk has no browser client.

The host's **Browser streaming** setting (UDP 9778, off by default) is the WebTransport listener
for that client ([Ports & firewall](/docs/ports)).
