# nikkileaks

# Requirements

- You need Oasis CLI. As for now, you can install this with `curl --proto '=https' --tlsv1.2 -sSL https://get.oasis.dev | python`

# Setup

- Run `oasis-chain` on your machine.
- In this directory, do `oasis test`
- You may need to `yarn add -D -W @types/node`

# Contributing
The smart contract is in `services/src/bin/greeter.rs`.
After modifying it, run `oasis build` to regenerate the `app/service-clients/greeter.ts` client.

Then, you can test the client with `yarn test`.
