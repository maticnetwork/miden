# Miden assembly

> This documentation has some deprecated snippets, this means that should be use just as reference to study purpose. It'll be rewritten in a near future.

This crate contains Miden assembler and description of the Miden assembly language located [here (deprecated documentation)](/../main/assembly/doc/assembly.md).

The purpose of the assembler is to compile Miden assembly source code into a Miden VM program (represented by `Program` struct). The program can then be executed on Miden VM [processor](../processor).

## Compiling assembly code
To compile Miden assembly source code into a program for Miden VM, you can use the `compile()` function exposed by this crate. This function takes the following parameters:

* `source: &str` - a reference to a string containing Miden assembly source code.

The `compile()` function returns `Result<Program, AssemblyError>` which will contain the compiled program if the compilation was successful, or if the source code contained errors, description of the first encountered error.

For example:
```Rust
use miden_assembly::compile;

// the program pushes values 3 and 5 onto the stack and adds them
let program = compile("begin push.3 push.5 add end").unwrap();
```

## License
This project is [MIT licensed](../LICENSE).