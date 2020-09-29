# Open Runtime Module Library (ORML) Workshop

This is a workshop project for learning about blockchain runtime development with
[Substrate](https://substrate.dev/),
[FRAME](https://substrate.dev/docs/en/knowledgebase/runtime/frame) and the
[Open Runtime Module Library](https://github.com/open-web3-stack/open-runtime-module-library). This
project implements a simple exchange protocol that is built on top of the ORML
[Currencies](https://github.com/open-web3-stack/open-runtime-module-library/tree/master/currencies)
and [Tokens](https://github.com/open-web3-stack/open-runtime-module-library/tree/master/tokens)
pallets. After completing this workshop, participants should have a better understanding of how to
design and implement a FRAME pallet.

## Add ORML Pallets

Find the implementations for the Currencies and Tokens pallets in
[the runtime](blob/master/runtime/src/lib.rs). Notice that the Tokens pallet is configured with a
`CurrencyId` set that specifies a native token; the Currencies pallet is configured to depend on the
Tokens pallet.

## Define an Exchange Protocol

The Exchange pallet defines a simple interface that depends on the ORML pallets that were configured
in the previous step:

- `submit_order(from_id, from_amt, to_id, to_amt)`
- `take_order(order_id)`
- `cancel_order(order_id)`

## Build & Run

If you need to,
[set up your Substrate development environment](https://substrate.dev/docs/en/knowledgebase/getting-started/#manual-installation).
Then, build and run a development chain:

```shell
$ cargo run -- --dev --tmp
```

Once the node is running, use this link to open the Polkadot JS Apps UI and connect to the Substrate
node: https://polkadot.js.org/apps/#/settings/developer?rpc=ws://127.0.0.1:9944. Use the Settings >
Developer app and the contents of the [`types.json`](blob/master/types.json) file to add the
necessary types to the UI.

## Upstream

This project was forked from the
[Substrate Developer Hub Node Template](https://github.com/substrate-developer-hub/substrate-node-template).
