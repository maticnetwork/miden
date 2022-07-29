## Input / output operations
Miden assembly provides a set of instructions for moving data between the stack and several other sources. These sources include:

* **Program code**: values to be moved onto the stack can be hard-coded in a program's source code.
* **Environment**: values can be moved onto the stack from environment variables. Currently, the only available environment variable is *stack_depth* which holds the current depth of the stack. In the future, other environment variables may be added.
* **Advice tape**: values can be moved onto the stack from a non-deterministic advice tape contained within the advice provider. Values are always read from the head of the advice tape, and once a value is read, it is removed from the tape. There is no limit on the number of values in the advice tape.
* **Memory**: values can be moved between the stack and random-access memory. The memory is word-addressable, meaning, four elements are located at each address, and we can read and write elements to/from memory in batches of four. Memory can be accessed via absolute memory references (i.e., via memory addresses) as well as via local procedure references (i.e., local index). The latter approach ensures that a procedure does not access locals of another procedure.

In the future several other sources such as *storage* and *logs* may be added.

### Constant inputs

| Instruction     | Stack_input | Stack_output | Notes                                      |
| --------------- | ----------- | ------------ | ------------------------------------------ |
| push.*a* <br> push.*a*.*b* <br> push.*a*.*b*.*c*... | [...] | [a, ...] <br> [b, a, ...] <br> [c, b, a, ...] | Pushes values $a$, $b$, $c$ etc. onto the stack. Up to $16$ values can be specified. All values must be valid field elements in decimal (e.g., $123$) or hexadecimal (e.g., $0x7b$) representation. |

When specifying values in hexadecimal format, it is possible to omit the periods between individual values as long as total number of specified bytes is a multiple of $8$. That is, the following are semantically equivalent:

```
push.0x1234.0xabcd
push.0x0000000000001234000000000000abcd
```
In both case the values must still encode valid field elements.

### Environment inputs

| Instruction          | Stack_input | Stack_output | Notes                                      |
| -------------------- | ----------- | ------------ | ------------------------------------------ |
| push.env.sdepth      | [...]       | [d, ...]     | $d \leftarrow stack.depth()$ <br> Pushes the current depth of the stack onto the stack. |
| push.env.locaddr.*i* | [...]       | [a, ...]     | $a \leftarrow address\_of(i)$ <br> Pushes the absolute memory address of local memory at index $i$ onto the stack. |

### Non-deterministic inputs

| Instruction    | Stack_input_  | Stack_output | Notes                                      |
| -------------- | --------------- | ------------ | ------------------------------------------ |
| push.adv.*n*   | [ ... ]         | [a, ... ]    | $a \leftarrow tape.next()$ <br> Removes the next $n$ values from advice tape and pushes them onto the stack. Valid for $n \in \{1, ..., 16\}$. <br> Fails if the advice tape has fewer than $n$ values. |
| loadw.adv      | [0, 0, 0, 0, ... ] | [A, ... ] | $A \leftarrow tape.next\_word()$ <br> Removes the next word (4 elements) from the advice tape and overwrites the top four stack elements with it. <br> Fails if the advice tape has fewer than $4$ values. |

### Random access memory

 As mentioned above, there are two ways to access memory in Miden VM. The first way is via memory addresses using the instructions listed below. The addresses are absolute - i.e., they don't depend on the procedure context. Memory addresses can be in the range $[0, 2^{32})$.
 
Memory is guaranteed to be initialized to zeros. Thus, when reading from memory address which hasn't been written to previously, zero elements will be returned.

| Instruction    | Stack_input___ | Stack_output | Notes                                      |
| -------------- | -------------- | ------------ | ------------------------------------------ |
| push.mem <br> push.mem.*a*   | [a, ... ] | [v, ... ] | $a \leftarrow mem[a][0]$ <br> Reads a word (4 elements) from memory at address *a*, and pushes the first element of the word onto the stack. If $a$ is provided via the stack, it is removed from the stack first. <br> Fails if $a \ge 2^{32}$ |
| pushw.mem <br> pushw.mem.*a* | [a, ... ] | [A, ... ] | $A \leftarrow mem[a]$ <br> Reads a word from memory at address $a$ and pushes it onto the stack. If $a$ is provided via the stack, it is removed from the stack first. <br> Fails if $a \ge 2^{32}$ |
| loadw.mem <br> loadw.mem.*a* | [a, 0, 0, 0, 0, ...] | [A, ... ] | $A \leftarrow mem[a]$ <br> Reads a word from memory at address $a$ and overwrites top four stack elements with it. If $a$ is provided via the stack, it is removed from the stack first. <br> Fails if $a \ge 2^{32}$ |
| pop.mem <br> pop.mem.*a*    | [a, v, ...] | [ ... ] | $[v, 0, 0, 0] \rightarrow mem[a]$ <br> Pops an element off the stack and stores it as the first element of the word in memory at address $a$. All other elements of the word are set to $0$. If $a$ is provided via the stack, it is removed from the stack first. <br> Fails if $a \ge 2^{32}$ |
| popw.mem <br> popw.mem.*a* | [a, A, ...] | [ ... ] | $A \rightarrow mem[a]$ <br> Pops the top four elements off the stack and stores them in memory at address $a$. If $a$ is provided via the stack, it is removed from the stack first. <br> Fails if $a \ge 2^{32}$ |
| storew.mem <br> storew.mem.*a* | [a, A, ...] | [A, ... ] | $A \rightarrow mem[a]$ <br> Stores the top four elements of the stack in memory at address $a$. If $a$ is provided via the stack, it is removed from the stack first. <br> Fails if $a \ge 2^{32}$ |

The second way to access memory is via procedure locals using the instructions listed below. These instructions are available only in procedure context. The number of locals available to a given procedure must be specified at [procedure declaration](#Procedures) time, and trying to access more locals than was declared will result in a compile-time error. The number of locals per procedure is not limited, but the total number of locals available to all procedures at runtime must be smaller than $2^{32}$.

| Instruction    | Stack_input_   | Stack_output | Notes                                      |
| -------------- | -------------- | ------------ | ------------------------------------------ |
| push.local.*i* | [ ... ] | [v, ... ] | $v \leftarrow local[i][0]$ <br> Reads a word (4 elements) from local memory at index *i*, and pushes the first element of the word onto the stack. |
| pushw.local.*i* | [...] | [A, ... ] | $A \leftarrow local[i]$ <br> Reads a word from local memory at index $i$ and pushes it onto the stack. |
| loadw.local.*i* | [0, 0, 0, 0, ...] | [A, ... ] | $A \leftarrow local[i]$ <br> Reads a word from local memory at index $i$ and overwrites top four stack elements with it. |
| pop.local.*i*  | [v, ...] | [ ... ] | $[v, 0, 0, 0] \rightarrow local[i]$ <br> Pops an element off the stack and stores it as the first element of the word in local memory at index $i$. All other elements of the word are set to $0$. |
| popw.local.*i* | [A, ...] | [ ... ] | $A \rightarrow local[i]$ <br> Pops the top four elements off the stack and stores them in local memory at index $i$. |
| storew.local.*i* | [A, ...] | [A, ... ] | $A \rightarrow local[i]$ <br> Stores the top four elements of the stack in local memory at index $i$. |

Unlike regular memory, procedure locals are not guaranteed to be initialized to zeros. Thus, when working with locals, one must assume that before a local memory address has been written to, it contains "garbage".

Internally in the VM, procedure locals are stored at memory offset stating at $2^{30}$. Thus, every procedure local has an absolute address in regular memory. The `push.env.locaddr` is provided specifically to map an index of a procedure's local to an absolute address so that it can be passed to downstream procedures, when needed.

