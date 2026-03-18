---
layout: home

hero:
  name: "forge-persist"
  text: "Persistent mainnet forks in one command."
  tagline: "Free, self-hosted replacement for Anvil and Tenderly Virtual Testnets."
  actions:
    - theme: brand
      text: Get Started
      link: /guide/introduction
    - theme: alt
      text: View on GitHub
      link: https://github.com/your-org/forge-persist

features:
  - title: Flat Memory Profile
    details: Backed by Reth's native MDBX database. Zero memory leaks out of the box, ensuring month-long up-times for testing indexers or maintaining persistent development state.
  - title: Blazing Fast Local RPC
    details: Achieves <1ms latency locally with fully simulated 1-second block times, replacing extremely expensive and brittle 50-200ms remote testnet calls.
  - title: Built-in Explorer UX
    details: Natively exposes the `ots` RPC namespace. Directly plug and play with standard web-hosted Otterscan to visually trace your local simulated transactions seamlessly.
---
