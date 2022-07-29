# Miden Standard Library
Miden standard library provides a set of procedures which can be used by any Miden program. These procedures build on the core instruction set of [Miden assembly](../assembly/main.md) expanding the functionality immediately available to the user.

The goals of Miden standard library are:
* Provide highly-optimized and battle-tested implementations of commonly-used primitives.
* Reduce the amount of code that needs to be shared between parties for proving and verifying program execution. 

The second goal can be achieved because calls to procedures in the standard library can always be serialized as 32 bytes, regardless of how large the procedure is.

## Organization and usage
Procedures in Miden standard library are organized into modules, each targeting a narrow set of functionality. Modules are grouped into higher-level namespaces. However, higher-level namespaces do not expose any procedures themselves. For example, `std::math::u64` is a module containing procedures for working with 64-bit unsigned integers. This module is a part of the `std::math` namespace. However, the `std::math` namespace does not expose any procedures.

To invoke a procedure from a standard library module, the module first needs to be imported using a `use` statement. Once a module is imported, procedures from this module can be invoked via the regular `exec` instruction as `exec.<module>::<label>` where `label` is the name of the procedure. An example of this is shown below.

```
use std::math::u64

begin
    push.1.0
    push.2.0
    exec.u64::checked_add
end
```
In the above example we first push two 64-bit integers on the the stack, and then invoke a 64-bit addition procedure from `std::math::u64` module.

## Available modules
CCurrently, Miden standard library contains just a few modules, which are listed below. Over time, we plan to add many more modules which will include various cryptographic primitives, additional numeric data types and operations, and many others.

| Module | Description |
| ------ | ----------- |
| [std::crypto::hashes::blake3](./crypto/hashes.md#blake3) | Contains procedures for computing hashes using BLAKE3 hash function. |
| [std::crypto::hashes::sha265](./crypto/hashes.md#sha256) | Contains procedures for computing hashes using SHA256 hash function. |
| [std::math::u64](./math/u64.md) | Contains procedures for working with 64-bit unsigned integers. |
| [std::sys](./sys.md)            | Contains system-level utility procedures. |