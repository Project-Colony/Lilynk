// The footer shape comes from @unom/app-ui/footer, shared with the marketing site.
import type { FooterData } from '@unom/app-ui/footer'

export type { FooterData as Footer } from '@unom/app-ui/footer'

// This build reads no CMS: the docs render without the shared footer.
export async function findFooter(): Promise<FooterData | null> {
  return null
}
