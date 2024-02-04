# ACPI

ACPI is a power-management and configuration framework. It stands for
"Advanced Configuration and Power Interface".

For some reason, PCIe puts its `MCFG` table in the ACPI tables, so BS has to
parse the ACPI tables to find it and use PCIe devices.
