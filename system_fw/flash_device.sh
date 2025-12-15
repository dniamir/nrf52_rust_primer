#!/bin/bash

SOFTDEVICE_HEX="./system_fw/s140_nrf52_7.3.0/s140_nrf52_7.3.0_softdevice.hex"

CHIP_NAME="nrf52840_xxAA" # Adjust to your specific chip

if [ "$1" == "--ble-program" ]; then
    echo "BLE program detected. Flashing SoftDevice..."
    # Use probe-rs to erase and flash the softdevice
    # The --allow-erase-all is often needed for the first time
    probe-rs erase --chip $CHIP_NAME --allow-erase-all
    probe-rs download --verify --binary-format hex --chip $CHIP_NAME $SOFTDEVICE_HEX
    echo "SoftDevice flashed. Proceeding with application build/run."
else
    echo "Non-BLE program. Skipping SoftDevice flash."
fi
