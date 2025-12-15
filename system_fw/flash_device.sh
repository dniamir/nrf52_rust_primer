#!/bin/bash

SOFTDEVICE_HEX="./system_fw/s140_nrf52_7.3.0/s140_nrf52_7.3.0_softdevice.hex"

CHIP_NAME="nrf52840_xxAA" # Adjust to your specific chip

SOFTDEVICE_ADDR=0x00001000
EXPECTED_WORD="c8 13 00 20"

# Auto-pick first J-Link probe
PROBE_ID=$(probe-rs list | grep "J-Link" | head -n1 | awk '{print $4}')

if [ -z "$PROBE_ID" ]; then
    echo "No J-Link probe found"
    exit 1
fi

# If memory.x already points to BLE layout, skip
if grep -q "memory_ble.x" memory.x; then
    echo "BLE memory layout already active. Skipping SoftDevice flash."
    exit 0
fi

echo "BLE memory layout not active. Flashing SoftDevice..."
probe-rs erase --probe "$PROBE_ID" --chip $CHIP_NAME --allow-erase-all
probe-rs download --probe "$PROBE_ID" --verify --binary-format hex --chip $CHIP_NAME "$SOFTDEVICE_HEX"  # Memory location is dictated by .hex file
echo "SoftDevice flashed."
