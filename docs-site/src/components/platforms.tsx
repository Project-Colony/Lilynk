// Install lines, notices and port facts rendered from the platforms data file, so a docs page never
// restates a repo URL or a port number (Contributing page, "Docs"). The canonical file
// is ../../data/platforms.json at the repo root; src/data/platforms.json is a byte-identical
// snapshot (the Docker build context is docs-site/ alone) that scripts/ci/check-docs-drift.sh
// keeps in sync — same arrangement as public/openapi.json.
import Link from 'fumadocs-core/link'
import { Callout } from 'fumadocs-ui/components/callout'
import { CodeBlock, Pre } from 'fumadocs-ui/components/codeblock'
import platforms from '@/data/platforms.json'

type Platform = (typeof platforms.platforms)[number]

function find(id: string): Platform {
  const p = platforms.platforms.find((x) => x.id === id)
  if (!p) throw new Error(`platforms.json has no platform "${id}"`)
  return p
}

/** Platforms whose `install` lines already build from source. */
const SOURCE_BUILT = ['nixos', 'steamos']

/** `<Install platform="nixos" />`: the platform's install lines, or the build-from-source notice
 * on a platform that has no package yet. */
export function Install({ platform }: { platform: string }) {
  const p = find(platform) // an unknown id still fails the build
  if (!SOURCE_BUILT.includes(platform) || !('install' in p) || !p.install) {
    return (
      <Callout>
        Lilynk publishes no packages yet, so this system has no install command. Build from a clone
        as this step describes. <Link href="/docs/developers/build-from-source">Build from source</Link>{' '}
        lists the tools.
      </Callout>
    )
  }
  return (
    <CodeBlock>
      <Pre>
        <code>
          {p.install.map((line, i) => (
            <span key={i} className="line">
              {line}
            </span>
          ))}
        </code>
      </Pre>
    </CodeBlock>
  )
}

/** `<Installer />`: builds the guided installer from a clone and runs it, from platforms.json. */
export function Installer() {
  return (
    <CodeBlock>
      <Pre>
        <code>
          {platforms.installer.fromSource.map((line, i) => (
            <span key={i} className="line">
              {line}
            </span>
          ))}
        </code>
      </Pre>
    </CodeBlock>
  )
}

/** `<Ports />` — every port the host and console use, with the firewall profile that opens it. */
export function Ports() {
  const { ports, firewall } = platforms
  const rows: [string, string, string, string][] = [
    ['Native control', `UDP ${ports.native.port}`, ports.native.what, firewall.native],
    ['Discovery', `UDP ${ports.mdns.port}`, ports.mdns.what, firewall.native],
    ['Management API', `TCP ${ports.mgmt.port}`, ports.mgmt.what, firewall.native],
    ['Browser streaming', `UDP ${ports.browser.port}`, ports.browser.what, firewall.native],
    ['Web console', `TCP ${[ports.web.port, ...ports.web.also].join(', ')}`, ports.web.what, firewall.web],
    [
      'GameStream (Moonlight)',
      `TCP ${ports.gamestream.tcp.join(', ')} · UDP ${ports.gamestream.udp.join(', ')}`,
      ports.gamestream.what,
      firewall.gamestream,
    ],
  ]
  return (
    <table>
      <thead>
        <tr>
          <th>Plane</th>
          <th>Port</th>
          <th>What</th>
          <th>firewalld service / ufw profile</th>
        </tr>
      </thead>
      <tbody>
        {rows.map(([plane, port, what, svc]) => (
          <tr key={plane}>
            <td>{plane}</td>
            <td>
              <code>{port}</code>
            </td>
            <td>{what}</td>
            <td>{svc === '—' ? svc : <code>{svc}</code>}</td>
          </tr>
        ))}
      </tbody>
    </table>
  )
}
