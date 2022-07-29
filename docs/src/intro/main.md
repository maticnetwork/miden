# Introduction
Miden VM is a zero-knowledge virtual machine written in Rust. For any program executed on Miden VM, a STARK-based proof of execution is automatically generated. This proof can then be used by anyone to verify that the program was executed correctly without the need for re-executing the program or even knowing the contents of the program.

## Status and features
Miden VM is currently on v0.2 release. In this release, most of the core features of the VM have been stabilized, and most of the STARK proof generation has been implemented. While we expect to keep making changes to the VM internals, the external interfaces should remain relatively stable, and we will do our best to minimize the amount of breaking changes going forward.

At this point, Miden VM is good enough for experimentation, and even for real-world applications, but it is not yet ready for production use. The codebase has not been audited and contains known and unknown bugs and security flaws.

### Feature highlights
Miden VM is a fully-featured virtual machine. Despite being optimized for zero-knowledge proof generation, it provides all the features one would expect from a regular VM. To highlight a few:

* **Flow control.** Miden VM is Turing-complete and supports familiar flow control structures such as conditional statements and counter/condition-controlled loops. There are no restrictions on the maximum number of loop iterations or the depth of control flow logic.
* **Procedures.** Miden assembly programs can be broken into subroutines called *procedures*. This improves code modularity and helps reduce the size of Miden VM programs.
* **Memory.** Miden VM supports read-write random-access memory. Procedures can reserve portions of global memory for easier management of local variables.
* **u32 operations.** Miden VM supports native operations with 32-bit unsigned integers. This includes basic arithmetic, comparison, and bitwise operations.
* **Cryptographic operations.** Miden assembly provides built-in instructions for computing hashes and verifying Merkle paths. These instructions use Rescue Prime hash function (which is the native hash function of the VM).
* **Standard library.** Miden VM ships with a standard library which expands the core functionality of the VM (e.g., by adding support for 64-bit unsigned integers). Currently, the standard library is quite limited, but we plan to expand it significantly in the future.
* **Nondeterminism**. Unlike traditional virtual machines, Miden VM supports nondeterministic programming. This means a prover may do additional work outside of the VM and then provide execution *hints* to the VM. These hints can be used to dramatically speed up certain types of computations, as well as to supply secret inputs to the VM.

### Planned features
In the coming months we plan to finalize the design of the VM and implement support for the following features:

* **Custom kernels.** It will be possible to instantiate Miden VM with a custom kernel. There will also be full separation between kernel space and user space, and we will introduce a number of opcodes to facilitate safe communication between the two.
* **Persistent storage.** Support for read-write persistent storage will be provided at the kernel level. With support for custom kernels, we will be able to support multiple modes of persistent storage, such as key-value maps, index-based arrays etc.
* **Faulty execution.** Miden VM will support generating proofs for programs with faulty execution (a notoriously complex task in ZK context). That is, it will be possible to prove that execution of some program resulted in an error.

## Structure of this document
This document is meant to provide an in-depth description of Miden VM. It is organized as follows:

* In the introduction, we provide a high-level overview of Miden VM and describe how to run simple programs.
* In the user documentation section, we provide developer-focused documentation useful to those who want to develop on Miden VM or build compilers from higher-level languages to Miden assembly (the native language of Miden VM).
* In the design section, we provide in-depth descriptions of the VM's internals, including all AIR constraints for the proving system. We also provide the rationale for settling on specific design choices.
* Finally, in the background material section, we provide references to materials which could be useful for learning more about STARKs - the proving system behind Miden VM.

## License
Licensed under the [MIT license](http://opensource.org/licenses/MIT).
