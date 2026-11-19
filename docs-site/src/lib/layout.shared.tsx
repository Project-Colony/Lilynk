import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared'
import BrandMark from '@/components/BrandMark'
import Wordmark from '@/components/Wordmark'

// Shared chrome (nav title, links) for both the docs layout and the home layout.
export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <>
          <BrandMark className="size-6" />
          <Wordmark className="h-4" />
        </>
      ),
    },
    links: [
      { text: 'Docs', url: '/docs' },
      { text: 'API', url: '/api' },
      { text: 'Source code', url: 'https://github.com/Project-Colony/Lilynk' },
      { text: 'Bugs & ideas', url: 'https://github.com/Project-Colony/Lilynk/issues' },
      { text: 'Discord', url: 'https://discord.gg/tvFApEEVfj' },
    ],
  }
}
