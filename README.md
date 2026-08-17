# Turborepo x Cloudflare Workers

Citing from myself, in a DM with `g**b***6`:

> [...] among all vendor lock-in services, Cloudflare is the funniest one.

## How to deploy?

- The [`deploy` action](https://github.com/swyrin/turborepo-cfworkers-selfhost/blob/539e3afad93d878dbe7dd831f60a637ea37989cf/.github/workflows/deploy.yml) should have everything you need.
  - Remember to fill `TURBO_TOKEN` to whatever you want.

## How do I know if it works?

[I am literally dogfooding it.](https://github.com/swyrin/turborepo-cfworkers-selfhost/blob/539e3afad93d878dbe7dd831f60a637ea37989cf/.github/workflows/deploy.yml#L13)
