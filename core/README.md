# Miden core 

> This documentation has some deprecated snippets, this means that should be use just as reference to study purpose. It'll be rewritten in a near future.

This crate contains core components used by Miden VM. These components include:

* Instruction set architecture (ISA) defined [here (deprecated code)](/../main/core/src/opcodes.rs) and described [here (deprecated documentation)](/../main/core/doc/isa.md).
* Program structure defined [here (deprecated code)](/../main/core/src/programs/mod.rs) and described [here (deprecated documentation)](/../main/core/doc/programs.md).
* Implementations of Rescue hash function used by the VM.
* Various minor utility functions used by other VM crates.

## License
This project is [MIT licensed](../LICENSE).