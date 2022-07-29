## Code organization
A Miden assembly program is just a sequence of instructions each describing a specific directive or an operation. You can use any combination of whitespace characters to separate one instruction from another.

In turn, Miden assembly instructions are just keywords which can be parameterized by zero or more parameters. The notation for specifying parameters is *keyword.param1.param2* - i.e., the parameters are separated by periods. For example, `push.123` instruction denotes a `push` operation which is parameterized by value `123`.

Currently, Miden assembly provides two types of code organization blocks: *programs* and *procedures*. In the future, additional blocks such as modules and functions will be added.

### Programs
A *program* block is used to define an executable program. A program must start with a `begin` instruction and terminate with an `end` instruction. For example:
```
begin
    <instructions>
end
```
When Miden assembly code is executed, the execution starts at the first instruction following the `begin` instruction of the program. A program block is expected to be the last block in an assembly file and can be followed only by comment blocks.

### Procedures
A *procedure* block is used to define a frequently-used sequence of instructions. A procedure must start with a `proc.<label>.<number of locals>` instruction and terminate with an `end` instruction. For example:
```
proc.foo.2
    <instructions>
end
```
A procedure label must start with a letter and can contain any combination of numbers, ASCII letters, and underscores (`_`).

The number of locals specifies the number of memory-based local words a procedure can access (via `load.local`, `store.local`, and other instructions). If a procedure doesn't need any memory-based locals, this parameter can be omitted or set to `0`. The number of locals per procedure is not limited, but the total number of locals available to all procedures at runtime must be smaller than $2^{32}$.

To execute a procedure an `exec.<label>` instruction should be used. For example:
```
exec.foo
```
During compilation, procedures are inlined at their call sites. Thus, from the standpoint of the final program, executing procedures is indistinguishable from manually including procedure code in place of the `exec` instruction.

A procedure may execute any other previously defined procedure from the same source, but it cannot execute itself or any of the subsequent procedures. Thus, recursive procedure calls are not possible. For example, the following code block defines a program with two procedures:
```
proc.foo
    <instructions>
end

proc.bar
    <instructions>
    exec.foo
    <instructions>
end

begin
    <instructions>
    exec.bar
    <instructions>
    exec.foo
end
```

In addition to local procedures defined within a program, Miden assembly programs can invoke procedures defined in the standard library as described [here](../stdlib/main.md).

### Comments
Miden assembly allows annotating code with simple comments. Currently, the only supported type of comments is a single-line comment which starts with a `#` (pound) character. For example:
```
# this is a comment
```

When Miden assembly is serialized into binary format, comments are not retained.
