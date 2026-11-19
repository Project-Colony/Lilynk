# Security policy

Lilynk streams a desktop and its games to other devices. A host gives remote control of the
machine it runs on, so a vulnerability in it can hand that machine to someone else.

## Supported versions

Lilynk has no releases yet. The supported version is the current `main` branch. Check that an
issue still reproduces on `main` before you report it. Security fixes land on `main`.

## Reporting a vulnerability

Report a security issue privately through GitHub's
[advisory form](https://github.com/Project-Colony/Lilynk/security/advisories/new). Only the
maintainers can read the report until an advisory is published.

Do not open a public issue or pull request, and do not post about it on Discord. That exposes other
users before a fix exists.

### What to include

- The component and the commit you built (`git rev-parse HEAD`), the host OS, and which client.
- The impact: what an attacker can do, and from what position (same LAN, a local service account,
  admin, a paired client).
- Steps to reproduce, a proof of concept, or a crash or log if you have one.
- A suggested fix or mitigation, if you have one.

## What to expect

Timelines are best effort. The maintainers aim to:

- acknowledge your report within 3 business days;
- give an initial assessment (severity, and whether it reproduces) within about 7 days;
- keep you updated in the advisory, and tell you when the fix lands;
- credit you in the published advisory, unless you prefer to stay anonymous.

Disclosure is coordinated: give the maintainers reasonable time to fix the issue before you publish
details. The aim is to resolve valid issues within 90 days, on a disclosure date agreed with you.

## Scope

In scope is the code in this repository:

- The host (`lilynk-host`), its Windows drivers, and the protocol and crypto core (`lilynk-core`).
- The native clients (Apple, Linux, Windows, Android), the web management console, and the
  management API.

These limits are documented behaviour, not vulnerabilities (see
[Security](https://project-colony.github.io/Lilynk/docs/security), or
`docs-site/content/docs/(reference)/security.md` until the docs site is published):

- An attacker who is already administrator or SYSTEM on the host owns the machine whatever Lilynk
  does. That is out of scope.
- The virtual display is a real monitor. Any process already in the interactive desktop session
  can capture it through the normal OS screen-capture APIs, as it could a physical monitor.
- GameStream and Moonlight compatibility (`--gamestream`) uses legacy encryption. It is opt-in
  and documented as trusted-LAN only.
- Exposing a host to the public internet is unsupported. Keep the host on a trusted LAN or a VPN.

If you are unsure whether something is in scope, report it.

## Verifying what you build

Lilynk publishes no binaries, packages or checksums yet. You build everything from a clone of
`https://github.com/Project-Colony/Lilynk.git`, so what you run is the source you checked out.
Compare `git rev-parse HEAD` with the commit you meant to build.

The Windows drivers (virtual display, virtual gamepads) are signed with a self-signed
code-signing certificate, `CN=lilynk-driver`. A build with no stable certificate configured signs
them with a throwaway certificate made for that build. To install a self-signed driver at all,
`lilynk-host driver install` adds the certificate to the machine's `Root` and `TrustedPublisher`
stores. The signature therefore does not authenticate the files: anyone who can change the bundle
can put their own certificate next to their own driver. `driver install` removes older
`CN=lilynk-driver` certificates before it adds the current one, and `driver uninstall` removes
them all. No driver certificate fingerprint is published yet.

Check the signer of the installed virtual display driver with `Get-AuthenticodeSignature` on
`ly_vdisplay.dll`, and list the trusted driver certificates with
`Get-ChildItem Cert:\LocalMachine\Root | ? Subject -like '*lilynk-driver*'`.

## Safe harbor

Good-faith security research that follows this policy is authorized, and the maintainers will not
take legal action against researchers who:

- make a good-faith effort to avoid privacy violations, data loss and service disruption,
- only test systems they own or have explicit permission to test,
- give reasonable time to fix the issue before public disclosure,
- do not exfiltrate more data than needed to show the issue.
