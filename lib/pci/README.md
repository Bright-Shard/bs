# PCI

PCI identifies all sorts of devices attached to the computer, from hard drives to GPUs. This crate supports discovering and controlling (to some extent) PCI devices.

Note that PCI has been superseded by PCIe. PCIe uses memory-mapped I/O instead of CPU I/O and is vastly more efficient and modern. This crate does not support PCIe, it only supports PCI.

# Sources
- https://www.khoury.northeastern.edu/~pjd/cs7680/homework/pci-enumeration.html
- https://wiki.osdev.org/PCI
