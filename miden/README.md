# Miden VM

> This documentation has some deprecated snippets, this means that should be use just as reference to study purpose. It'll be rewritten in a near future.

This crate contains an implementation of Miden VM. It can be used to execute Miden VM programs and to verify correctness of program execution.

## Overview
Miden VM is a simple [stack machine](https://en.wikipedia.org/wiki/Stack_machine). This means all values live on the stack and all operations work with values near the top of the stack. 

### The stack
Currently, Miden VM stack can be up to 32 items deep (this limit will be removed in the future). However, the more stack space a program uses, the longer it will take to execute, and the larger the execution proof will be. So, it pays to use stack space judiciously.

Values on the stack are elements of a [prime field](https://en.wikipedia.org/wiki/Finite_field) with modulus `340282366920938463463374557953744961537` (which can also be written as 2<sup>128</sup> - 45 * 2<sup>40</sup> + 1). This means that all valid values are in the range between `0` and `340282366920938463463374557953744961536` - this covers almost all 128-bit integers.   

All arithmetic operations (e.g., addition, multiplication) happen in the same prime field. This means that overflow happens after a value reaches field modulus. So, for example: `340282366920938463463374557953744961536 + 1 = 0`.

Besides being field elements, values in Miden VM are untyped. However, some operations expect binary values and will fail if you attempt to execute them using non-binary values. Binary values are values which are either `0` or `1`.

### Programs
Programs in Miden VM are structured as an [execution graph (deprecated documentation)](/../main/core/doc/programs.md) of program blocks each consisting of a sequence of VM [instructions (deprecated documentation)](/../main/core/doc/isa.md). There are two ways of constructing such a graph:

1. You can manually build it from blocks of raw Miden VM instructions.
2. You can compile [Miden assembly](../assembly) source code into it.

The latter approach is strongly encouraged because building programs from raw Miden VM instructions is tedious, error-prone, and requires an in-depth understanding of VM internals. All examples throughout these docs use assembly syntax.

### Inputs / outputs
Currently, there are 3 ways to get values onto the stack:

1. You can use `push` operations to push values onto the stack. These values become a part of the program itself, and, therefore, cannot be changed between program executions. You can think of them as constants.
2. You can initialize the stack with a set of public inputs as described [here](#program-inputs). Because these inputs are public, they must be shared with a verifier for them to verify program execution.
3. You can provide unlimited number of secret inputs via input tapes `A` and `B`. Similar to public inputs, these tapes are defined as a part of [program inputs](#program-inputs). To move secret inputs onto the stack, you'll need to use `read` operations.

Values remaining on the stack after a program is executed can be returned as program outputs. You can specify exactly how many values (from the top of the stack) should be returned. Currently, the number of outputs is limited to 8. A way to return a large number of values (hundreds or thousands) is not yet available, but will be provided in the future.

### Memory
Currently, Miden VM has no random access memory - all values live on the stack. However, a memory module will be added in the future to enable saving values to and reading values from RAM.

### Program hash
All Miden programs can be reduced to a single 32-byte value, called program hash. Once a `Program` object is constructed (e.g. by compiling assembly code), you can access this hash via `Program::hash()` method. This hash value is used by a verifier when they verify program execution. This ensure that the verifier verifies execution of a specific program (e.g. a program which the prover had committed to previously). The methodology for computing program hash is described [here (deprecated documentation)](/../main/core/doc/programs.md#Program-hash).

## Usage
Miden crate exposes `prove()` and `verify()` functions which can be used to execute programs and verify their execution. Both are explained below, but you can also take a look at several working examples [here](src/examples) and find instructions for running them via cli [here](#examples).

### Executing a program 
To execute a program on Miden VM, you can use `execute()` function. The function takes the following parameters:

* `program: &Program` - the program to be executed. A program can be constructed manually by building a program execution graph, or compiled from Miden assembly (see [here](#Writing-programs)).
* `inputs: &ProgramInputs` - inputs for the program. These include public inputs used to initialize the stack, as well as secret inputs consumed during program execution (see [here](#Program-inputs)).
* `num_outputs: usize` - number of items on the stack to be returned as program output. Currently, at most 8 outputs can be returned.
* `options: &ProofOptions` - config parameters for proof generation. The default options target 96-bit security level.

If the program is executed successfully, the function returns a tuple with 2 elements:

* `outputs: Vec<u128>` - the outputs generated by the program. The number of elements in the vector will be equal to the `num_outputs` parameter.
* `proof: StarkProof` - proof of program execution. `StarkProof` can be easily serialized and deserialized using `to_bytes()` and `from_bytes()` functions respectively.

#### Program inputs
To provide inputs for a program, you must create a `ProgramInputs` object which can contain the following:

* A list of public inputs which will be used to initialize the stack. Currently, at most 8 public inputs can be provided.
* Two lists of secret inputs. These lists can be thought of as tapes `A` and `B`. You can use `read` operations to read values from these tapes and push them onto the stack.

Besides the `ProgramInputs::new()` function, you can also use `ProgramInputs::from_public()` and `ProgramInputs:none()` convenience functions to construct the inputs object.

#### Program execution example
Here is a simple example of executing a program which pushes two numbers onto the stack and computes their sum:
```Rust
use miden::{assembly, ProgramInputs, ProofOptions};

// this is our program, we compile it from assembly code
let program = assembly::compile("begin push.3 push.5 add end").unwrap();

// let's execute it
let (outputs, proof) = miden::execute(
    &program,
    &ProgramInputs::none(),   // we won't provide any inputs
    1,                        // we'll return one item from the stack
    &ProofOptions::default(), // we'll be using default options
)
.unwrap();

// the output should be 8
assert_eq!(vec![8], outputs);
```

### Verifying program execution
To verify program execution, you can use `verify()` function. The function takes the following parameters:

* `program_hash: &[u8; 32]` - an array of 32 bytes representing a hash of the program to be verified.
* `public_inputs: &[u128]` - a list of public inputs against which the program was executed.
* `outputs: &[u128]` - a list of outputs generated by the program.
* `proof: &StarkProof` - the proof generated during program execution.

The function returns `Result<(), VerifierError>` which will be `Ok(())` if verification passes, or `Err(VerifierError)` if verification fails, with `VerifierError` describing the reason for the failure.

Verifying execution proof of a program basically means the following:

> If a program with the provided hash is executed against some secret inputs and the provided public inputs, it will produce the provided outputs.

Notice how the verifier needs to know only the hash of the program - not what the actual program was.

#### Verifying execution example
Here is a simple example of verifying execution of the program from the previous example:
```Rust
use miden;

let program =   /* value from previous example */;
let proof =     /* value from previous example */;

// let's verify program execution
match miden::verify(*program.hash(), &[], &[8], proof) {
    Ok(_) => println!("Execution verified!"),
    Err(msg) => println!("Something went terribly wrong: {}", msg),
}
```

## Fibonacci calculator
Let's write a simple program for Miden VM (using [Miden assembly](../assembly). Our program will compute the 5-th [Fibonacci number](https://en.wikipedia.org/wiki/Fibonacci_number):

```
push.0      // stack state: 0
push.1      // stack state: 1 0
swap        // stack state: 0 1
dup.2       // stack state: 0 1 0 1
drop        // stack state: 1 0 1
add         // stack state: 1 1
swap        // stack state: 1 1
dup.2       // stack state: 1 1 1 1
drop        // stack state: 1 1 1
add         // stack state: 2 1
swap        // stack state: 1 2
dup.2       // stack state: 1 2 1 2
drop        // stack state: 2 1 2
add         // stack state: 3 2
```
Notice that except for the first 2 operations which initialize the stack, the sequence of `swap dup.2 drop add` operations repeats over and over. In fact, we can repeat these operations an arbitrary number of times to compute an arbitrary Fibonacci number. In Rust, it would like like this (this is actually a simplified version of the example in [fibonacci.rs](src/examples/src/fibonacci.rs)):
```Rust
use miden::{assembly, ProgramInputs, ProofOptions};

// set the number of terms to compute
let n = 50;

// build the program
let mut source = format!("
    begin 
        repeat.{}
            swap dup.2 drop add
        end
    end",
    n - 1
);
let program = assembly::compile(&source).unwrap();

// initialize the stack with values 0 and 1
let inputs = ProgramInputs::from_public(&[1, 0]);

// execute the program
let (outputs, proof) = miden::execute(
    &program,
    &inputs,
    1,                        // top stack item is the output
    &ProofOptions::default(), // use default proof options
)
.unwrap();

// the output should be the 50th Fibonacci number
assert_eq!(vec![12586269025], outputs);
```
Above, we used public inputs to initialize the stack rather than using `push` operations. This makes the program a bit simpler, and also allows us to run the program from arbitrary starting points without changing program hash.

This program is rather efficient: the stack never gets more than 4 items deep. For some benchmarks of executing this program on the VM see [here](../README.md#Performance).

## Crate features
Miden VM can be compiled with the following features:

* `std` - enabled by default and relies on the Rust standard library.
* `concurrent` - implies `std` and also enables multi-threaded proof generation.
* `no_std` does not rely on the Rust standard library and enables compilation to WebAssembly.

To compile with `no_std`, disable default features via `--no-default-features` flag.

### Concurrent proof generation
When compiled with `concurrent` feature enabled, the VM will generate STARK proofs using multiple threads. The number of threads can be configured via `RAYON_NUM_THREADS` environment variable, and usually defaults to the number of logical cores on the machine. For benefits of concurrent proof generation check out these [benchmarks](../README.md#Performance).


## Examples
This crate contains several examples of Miden VM programs and demonstrates how these programs can be executed on Miden VM.

## Running examples
To run examples of executing programs and verifying proofs of their execution, do the following:

First, compile an optimized version of the `miden` binary by running:
```
cargo build --release --features executable
```
Or, if you want to compile the with multi-threaded support enabled, run:
```
cargo build --release --features "executable concurrent"
```

In either case, the binary will be located in `target/release` directory, and you can run it like so:
```
./target/release/miden [FLAGS] [OPTIONS] <SUBCOMMAND>
```
Where each example can be invoked using a distinct subcommand. To view the list of all available options and examples you can look up help like so:

```
./target/release/miden -h
```
This will print out something similar to this:
```
Miden 0.1.0
Miden examples
USAGE:
    miden.exe [OPTIONS] <SUBCOMMAND>
FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
OPTIONS:
    -s, --security <security>    Security level for execution proofs generated by the VM [default: 96bits]
SUBCOMMANDS:
    collatz        Compute a Collatz sequence from the specified starting value
    comparison     If provided value is less than 9, multiplies it by 9; otherwise add 9 to it
    conditional    If provided value is 0, outputs 15; if provided value is 1, outputs 8
    fib            Compute a Fibonacci sequence of the specified length
    help           Prints this message or the help of the given subcommand(s)
    merkle         Computes a root of a randomly generated Merkle branch of the specified depth
    range          Determines how many of the randomly generated values are less than 2^63
```

Currently, the only available option for all examples is `-s` for specifying security level for the generated proofs. This can be set to one of two values:
* *96bits* - for 96-bit security level (the default).
* *128bits* - for 128-bit security level.

For example, to execute Fibonacci calculator at 128-bit security level, you can run the following:
```
./target/release/miden -s 128bits fib
```

### Example-specific options

To view additional options available for specific examples, you can run the following:
```
./target/release/miden help <example name>
```
For example, executing:
```
./target/release/miden help collatz
```
Will print something like this:
```
miden.exe-collatz 0.1.0
Compute a Collatz sequence from the specified starting value
USAGE:
    miden.exe collatz [OPTIONS]
FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
OPTIONS:
    -n <start-value>        Starting value of the Collatz sequence [default: 511]
```

Thus, to compute Collatz sequence with a different starting value, you could execute something like this:
```
./target/release/miden collatz -n 513
```

## License
This project is [MIT licensed](../LICENSE).