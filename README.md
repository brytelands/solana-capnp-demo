# solana-capnp-demo

## Description

This is an example of a Solana Program (non-Anchor) using Cap’n Proto for serialization/deserialization instead of the standard Borsh library that is used. This demo is currently a proof of concept exploring the use of different interchange formats for data. This example uses the bryte-descriptor libraries (https://github.com/brytelands/bryte-descriptor) to store the capn schema (IDL) online in an account in order to be retrieved by other clients.

## Build

#### Install capnp

In order to fully utilize this demo, you'll need to install Cap’n Proto (https://capnproto.org/install.html)

Build the capnp-models crate. This library contains the schema (IDL) used to generate the Rust files used for serialization in the Solana program.
```shell
cd ./capn-models
cargo build
```

Start your local validator
```shell
solana-test-validator --reset
```

Build the Solana program
```shell
cd ./program
cargo build-sbf
```

Deploy the program to your local validator
```shell
solana program deploy ./target/deploy/solana_capnp_demo.so
```

Run the client test
```shell
cd ./client
npm run capnp_demo
```

(Optional) Example using borsh
```shell
cd ./client
npm run borsh_demo
```

### Other tests

In the program/tests/schema_test.rs test file are some examples of retrieving the schema for an account on-chain, then using it to generate source for ser/de. There is also an example of dynamically loading a schema if you generated the schema using:

```shell
capnp compile -o/bin/cat src/person.capnp > src/person_capnp.bin
```

and stored the bin in the descriptor account.

This command will convert the binary schema to json.
```shell
capnp convert binary:json schema.capnp CodeGeneratorRequest < person_capnp.bin > person_capnp.json
```

## Code Overview

### capnp-models

This crate contains a simple capnp schema along with the generated code. Presumably this schema as well as the generated code may be used with other code in your ecosystem or 3rd parties. In this demo, the schema is stored on-chain in a "descriptor account" in order to be retrieved for off-chain deserialization purposes.

### client

This module contains two client examples, one being a demo of capnp and the other borsh. The client simply initializes an account and an account descriptor and stores the schemas on the account descriptor. It will then call an instruction that deserializes the data using either capnp or borsh.

### program

This is a non-Anchor Solana program that demonstrations using capnp serialization and deserialization.