# esp32c3-rtic-tau

Repository for COMP.CE.340 Dependable Embedded Systems practical work 1 assignment. Repo structure:

- `esp32c3`, code for the target.
- `shared`, library for shared data structures.

## Software requirements

Software components needed for running the practical work 1 examples (**Already satisfied on course VMs**):

- `probe-rs-tools` for flashing and debugging the target
- A Rust toolchain from the `stable` channel (<https://rustup.rs/>).

We flash these examples using `cargo embed`, cargo-subcommand. Obtain the tools by running the following commands according to advice at [probe-rs](https://probe.rs/) site (**Already done on the course VM**):

1. `curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh`
2. Setup udev rules for probe-rs: <https://probe.rs/docs/getting-started/probe-setup/>
3. Refresh udev rules `sudo udevadm control --reload-rules && sudo udevadm trigger`

## Running the examples

ESP32-C3 programs can be run on the target device as follows.

- Change to target directory:
  - `cd esp32c3`
- Use `cargo embed` to build & run an example, e.g.,
  - `cargo embed --example blinky`
