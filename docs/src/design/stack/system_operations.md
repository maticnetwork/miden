# System Operations
In this section we describe the AIR constraint for Miden VM system operations.  

## NOOP

`NOOP` advances the cycle counter but doesn't change the state of the user stack. 

The `NOOP` operation will not change the depth of the stack i.e. the stack doesn't shift while transitioning. The maximum degree of this operation is 1.

## ASSERT

Assume $a$ is the top element in the stack. `ASSERT` operation pops $a$ and asserts that it is equal to 1. 

![assert](../../assets/design/stack/system_ops/ASSERT.png)

The stack transition must satisfy the following condition:

> $$
s_0 - 1 = 0 \text{ | degree } = 1
$$

The `ASSERT` operation will shift the stack to the left by one. The maximum degree of this operation is $1$.

## FMPADD

`FMPADD` operation pops the top element of the stack, adds the current value of `fmp` register to it, and pushes this result back onto the stack. The diagram below illustrates this graphically.

![fmpadd](../../assets/design/stack/system_ops/FMPADD.png)

The stack transition for this operation must follow the following constraint:

> $$
s_0' - s_0 - fmp1 = 0 \text{ | degree } = 1
$$

The `FMPADD` operation will not change the depth of the stack i.e. the stack doesn't shift while transitioning. The maximum degree of this operation is $1$.

## FMPUPDATE

`FMPUPDATE` operation pops the top element of the stack and adds it to the current value of `fmp` register. The diagram below illustrates this graphically.

![fmpupdate](../../assets/design/stack/system_ops/FMPUPDATE.png)

The stack transition for this operation must follow the following constraint:

> $$
fmp2 - fmp1 - s_0 = 0 \text{ | degree } = 1
$$

The `FMPUPDATE` operation will shift the stack to the left by one. The maximum degree of this operation is 1.