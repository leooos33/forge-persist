import { defineConfig } from 'vitepress'

export default defineConfig({
  title: "forge-persist",
  description: "Persistent mainnet forks in one command. Zero memory leaks.",
  themeConfig: {
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Guide', link: '/guide/introduction' },
      { text: 'CLI Reference', link: '/cli' }
    ],
    sidebar: [
      {
        text: 'Overview',
        items: [
          { text: 'Introduction', link: '/guide/introduction' },
          { text: 'Installation', link: '/guide/installation' },
          { text: 'Core Concepts', link: '/guide/concepts' }
        ]
      },
      {
        text: 'Usage',
        items: [
          { text: 'CLI Reference', link: '/cli' },
          { text: 'Running with Foundry', link: '/guide/foundry' },
          { text: 'Interacting with Explorer', link: '/guide/explorer' }
        ]
      }
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/your-org/forge-persist' }
    ]
  }
})
