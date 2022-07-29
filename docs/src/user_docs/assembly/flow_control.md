## Flow control
As mentioned above, Miden assembly provides high-level constructs to facilitate flow control. These constructs are:

- *if-else* expressions for conditional execution.
- *repeat* expressions for bounded counter-controlled loops.
- *while* expressions for unbounded condition-controlled loops.

### Conditional execution
Conditional execution in Miden VM can be accomplished with *if-else* statements. These statements look like so:
```
if.true
    <instructions>
else
    <instructions>
end
```
where `instructions` can be a sequence of any instructions, including nested control structures; the `else` clause is optional. The above does the following:

1. Pops the top item from the stack.
2. If the value of the item is $1$, instructions in the `if.true` branch are executed.
3. If the value of the item is $0$, instructions in the `else` branch are executed.
4. If the value is not binary, the execution fails.

A note on performance: using *if-else* statements incurs a small, but non-negligible overhead. Thus, for simple conditional statements, it may be more efficient to compute the result of both branches, and then select the result using [conditional drop](#Conditional-manipulation) instructions.

### Counter-controlled loops
Executing a sequence of instructions a predefined number of times can be accomplished with *repeat* statements. These statements look like so:
```
repeat.<count>
    <instructions>
end
```
where:

* `instructions` can be a sequence of any instructions, including nested control structures.
* `count` is the number of times the `instructions` sequence should be repeated (e.g. `repeat.10`). `count` must be an integer greater than $0$.

### Condition-controlled loops
Executing a sequence of instructions zero or more times based on some condition can be accomplished with *while loop* expressions. These expressions look like so:
```
while.true
    <instructions>
end
```
where `instructions` can be a sequence of any instructions, including nested control structures. The above does the following:

1. Pops the top item from the stack.
2. If the value of the item is $1$, `instructions` in the loop body are executed.
    a. After the body is executed, the stack is popped again, and if the popped value is $1$, the body is executed again.
    b. If the popped value is $0$, the loop is exited.
    c. If the popped value is not binary, the execution fails.
3. If the value of the item is $0$, execution of loop body is skipped.
4. If the value is not binary, the execution fails.
