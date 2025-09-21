# Experimental embedded Rust (no_std) SDK for RP2040

A minimal, allocation-light, experimental SDK targeting **RP2040**. It focuses on signing and building transactions and interacting with **Gas Stations**. This is a learning project; APIs are intentionally low-level.

- no_std
- Almost allocation-free (only the BCS serialization allocates)
- Transaction creation, signing, and building
- Gas Station support
- Minimal helpers/builders (PTB is assembled manually)
- Referenced `bcs library` was modified to be `no-std` (https://github.com/lmoe/bcs-no-std)

Status: experimental. Not production-ready.

## What this example does

A simple end-to-end scenario:

- Task: “An RP2040-based temperature sensor posts values to a Move contract via a Gas Station.”
- To keep the firmware simple, temperature values are currently hard coded.
- Real sensors:
    - DS18B20 can be added with an existing library.
    - Thermistors can be read via ```Adc``` and processed with a Steinhart calculation.
- A corresponding Move contract (in `./move`) exists to accept temperature values.
- Note: Each push creates a new Temperature object, which is more costly than appending values to on-chain storage.

## Project layout

- ```firmware/``` — embedded firmware
    - ```src/tx_builder.rs``` — example showing manual PTB creation

Helper methods and builder types are intentionally sparse for now; use ```firmware/src/tx_builder.rs``` as a reference for constructing payloads manually.

## Building / Running


* Deploy the `./move` contract. Save the packageID. 
* Create a `./firmware/config.json` by using `./firmware/config.default.json` as reference, fill in the gaps.  
* Build the firmware from its subdirectory:

```sh
cd firmware

cargo build --release
<or>
cargo run --release

```

Tip: Add ```--release``` for optimized builds.

## Limitations

- Minimal error handling: some operations return errors, others may panic.
- Few abstractions: expect to write boilerplate for PTBs and flows.
- Costs: the demo creates a new on-chain object per temperature submission.

