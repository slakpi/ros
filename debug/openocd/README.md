OpenOCD
=======

Interface, target, and GDB scripts to debug hardware with OpenOCD. Currently,
only the FTDI FT4232H JTAG GDB server is supported.

Connections
-----------

### Serial (GPIO Alt4 Configuration)

    USB Serial          Raspberry Pi
    --------------------------------
    Black (Ground)      6 (Ground)
    White (RX)          8 (TX)
    Green (TX)          10 (RX)

### JTAG (GPIO Alt4 Configuration)

    FT4232H             Raspberry Pi
    --------------------------------
    CN2:1 - CN2:11      -
    CN2:7               22 (TCK)
    CN2:8               37 (TDI)
    CN2:9               18 (TDO)
    CN2:10              13 (TMS)
    CN2:12              15 (TRST)
    CN3:1 - CN3:3       -
    CN3:4               9 (Ground)
