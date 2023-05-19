Rustland Architecture-Dependent Module
======================================

The Rustland Architecture-Dependent Module provides a single interface to
perform architecture-dependent tasks such as page table operations, interrupt
operations, etc.

`mod.rs` simply imports the appropriate architecture files. This means each
architecture has to implement the same functionality.
