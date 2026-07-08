# HAL

The hardware abstraction layer defines the contract between the core bootloader code and the hardware-specific implementation. This allows the bootloader to adapt to chip-specific features and workflows without requiring custom code for each.

These are usually defined as function overrides which the core code runs against the HAL implementation.

Examples:
- Flash operations (erase sectors, unlock, etc)
- Hardware security module
- EEPROM
- SRAM
- Chip limitations
