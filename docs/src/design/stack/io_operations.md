# I/O Operations
In this section we describe the AIR constraint for Miden VM I/O operations.

### PUSH

`PUSH` operation pushes the provided immediate value onto the stack. Currently, it is the only such operation in Miden VM which accepts an immediate value. The operation has been explained in detail [here](https://maticnetwork.github.io/miden/design/decoder/main.html#handling-immediate-values).

The `PUSH` operation will shift the stack to the right by one.

### SDEPTH

`SDEPTH` pushes the current depth of the stack prior to the execution of this operation onto the stack. The diagram below illustrates this graphically.

![sdepth](../../assets/design/stack/io_operations/SDEPTH.png)

The stack transition for this operation must follow the following constraint:

> $$
s_0' - b_0 = 0 \text{ | degree } = 1\\
b_0' - b_0 = 1 \text{ | degree } = 1
$$

where $b_0$ is the stack depth prior to the execution of this operation. 

The `SDEPTH` operation will shift the stack to the right by one. The maximum degree of the above operation is $1$.

### READ

`READ` operation removes the next element from the advice tape and pushes it onto the stack. The diagram below illustrates this graphically.

![read](../../assets/design/stack/io_operations/READ.png)

The stack transition for this operation must follow the following constraint:

> $$
s_i' - s_{i-1} = 0 \space where \space i \in \{1..15\} \text{ | degree } = 1
$$

The `READ` operation will shift the stack to the right by one. The maximum degree of the above operation is $1$.

### READW

`READW` operation removes a word (4 consecutive field elements) from the advice tape and pushes it onto the stack by overwriting the existing elements present in the stack. The diagram below illustrates this graphically.

![readw](../../assets/design/stack/io_operations/READW.png)

The stack transition for this operation must follow the following constraint:

> $$
s_i' - s_{i-4} = 0 \space where \space i \in \{4..15\} \text{ | degree } = 1
$$

The `READW` operation will not change the depth of the stack i.e. the stack doesn't shift while transitioning. The maximum degree of the above operation is $1$.

### MLOAD

`MLOAD` operation loads the first element from the specified memory address onto the stack. The top element in the stack before the execution of this operation is the memory address which is been used to fetch a word(4 consecutive field elements) from the memory and then the *first* element of this word is pushed to the top of the stack. The diagram below illustrates this graphically.

![mload](../../assets/design/stack/io_operations/MLOAD.png)

To facilitate this operation, we will need to perform a lookup into the memory table at the specified address using the current values of context and clock cycle registers as described [here](https://maticnetwork.github.io/miden/design/chiplets/memory.html). 

Let's define a few intermediate variables to simplify constraint description:

$$
v_h = \alpha_0 + \alpha_1 \cdot 8 + \alpha_2 \cdot s_0 + \alpha_3 \cdot clk1
$$

$$
v_{n} = \alpha_4 \cdot s_0' + \sum_{i=0}^2\alpha_{i+5} \cdot m_i
$$

$$
v_{o} = \alpha_8 \cdot s_0' + \sum_{i=0}^2\alpha_{i+9} \cdot m_i
$$

In the above:
- $v_h$ is a _common header_ which is a combination of unique identifier, memory address and clock cycle. The $8$ in the permutation check is the unique identifier of `MEMORY` operation which has been explained [here](../stack/unique_identifier.md#identifiers).
- $clk1$ is the clock cycle and $\alpha_0$, $\alpha_1$, $\alpha_2$ etc... are random values sent from the verifier to the prover for use in permutation checks.
- Values for the helper registers $m_0, m_1,  m_2$ are provided by the VM non-deterministically and represent last 3 elements of the old memory at specified address.
- $v_{n}$ and $v_{o}$ can be thought of as component of new and old memory value (whole word) in the permutation check calculation. 

The lookup in the table can be accomplished by including the value into the lookup product such that it follows the following constraint:

> $$
b_{aux}' \cdot \left(v_h + v_{n} + v_{o}\right) = b_{aux} \text{ | degree } = 2
$$

where $b_{aux}$ is the running product column of auxiliary table. We are not including context in the permutation check as its not yet been implemented. We plan to add context to the memory in $v3.0$.

Although we are only updating one element in the memory we need to include the rest of all elements in the permutation check as the overall memory value(we can loosely relate to a hash of memory elements though it is not appropriate) has changed. Also, note that the value from the top of the stack and helper registers are added twice to the permutation check: once for the old values and the other one for new values. We can do this because old and new values in the memory table row corresponding to the load operation are the same.

The `MLOAD` operation will not change the depth of the stack i.e. the stack doesn't shift while transitioning. The maximum degree of the above operation is $2$.

### MLOADW

`MLOADW` operation loads a word(4 consecutive field elements) from the specified memory address onto the stack. The top element in the stack before the execution of this operation is the memory address which is been used to fetch a word and then the top four elements of the stack are overwritten with values retrieved from the memory. The diagram below illustrates this graphically.

![mloadw](../../assets/design/stack/io_operations/MLOADW.png)

To facilitate this operation, we will need to perform a lookup into the memory table at the specified address using the current values of context and clock cycle registers described [here](https://maticnetwork.github.io/miden/design/chiplets/memory.html). 

Let's define a few intermediate variables to simplify constraint description:

$$
v_h = \alpha_0 + \alpha_1 \cdot 8 + \alpha_2 \cdot s_0 + \alpha_3 \cdot clk1
$$

$$
v_{n} = \sum_{i=0}^3\alpha_{i+4} \cdot s_i'
$$

$$
v_{o} = \sum_{i=0}^3\alpha_{i+8} \cdot s_i'
$$

In the above:
- $v_h$ is a _common header_ which is a combination of unique identifier, memory address and clock cycle. The $8$ in the permutation check is the unique identifier of `MEMORY` operation which has been explained [here](../stack/unique_identifier.md#identifiers).
- $clk1$ is the clock cycle and $\alpha_0$, $\alpha_1$, $\alpha_2$ etc... are random values sent from the verifier to the prover for use in permutation checks.
- $v_{n}$ and $v_{o}$ can be thought of as component of new and old memory value (whole word) in the permutation check calculation. 

The lookup in the table can be accomplished by including the value into the lookup product such that it follows the following constraint:

> $$
b_{aux}' \cdot \left(v_h + v_{n} + v_{o}\right) = b_{aux} \text{ | degree } = 2
$$

where $b_{aux}$ is the running product column of auxiliary table. We are not including context in the permutation check as its not yet been implemented. We plan to add context to the memory in $v3.0$. 

Also, note that the value from the top of the stack are added twice to the permutation check: once for the old values and the other one for new values. We can do this because old and new values in the memory table row corresponding to the load operation are the same.

The `MLOADW` operation will not change the depth of the stack i.e. the stack doesn't shift while transitioning. The maximum degree of the above operation is $2$.

### MSTORE

`MSTORE` operation stores an element from the stack into the first slot at the specified memory address. The top element in the stack before the execution of this operation is the memory address which is been used to fetch a word and then the top stack element is saved into the first element of the word located at the specified memory address. The remaining three elements of the word were not affected. The diagram below illustrates this graphically.

![mstore](../../assets/design/stack/io_operations/MSTORE.png)

To facilitate this operation, we will need to perform a lookup into the memory table at the specified address using the current values of context and clock cycle registers described [here](https://maticnetwork.github.io/miden/design/chiplets/memory.html). 

Let's define a few intermediate variables to simplify constraint description:

$$
v_h = \alpha_0 + \alpha_1 \cdot 8 + \alpha_2 \cdot s_0 + \alpha_3 \cdot clk1
$$

$$
v_{n} = \alpha_4 \cdot s_0' + \sum_{i=1}^3\alpha_{i+4} \cdot m_i
$$

$$
v_{o} = \sum_{i=0}^3\alpha_{i+8} \cdot m_i
$$

In the above:
- $v_h$ is a _common header_ which is a combination of unique identifier, memory address and clock cycle. The $8$ in the permutation check is the unique identifier of `MEMORY` operation which has been explained [here](../stack/unique_identifier.md#identifiers).
- $clk1$ is the clock cycle and $\alpha_0$, $\alpha_1$, $\alpha_2$ etc... are random values sent from the verifier to the prover for use in permutation checks. 
- Values for the helper registers $m_0, m_1,  m_2, m_3$ are provided by the VM non-deterministically and represent old memory at the specified address.
- $v_{n}$ and $v_{o}$ can be thought of as component of new and old memory value (whole word) in the permutation check calculation. 

The lookup in the table can be accomplished by including the value into the lookup product such that it follows the following constraint:

> $$
s_0' - s_1 = 0 \text{ | degree } = 1\\
b_{aux}' \cdot \left(v_h + v_{n} + v_{o}\right) = b_{aux} \text{ | degree } = 2
$$

where $b_{aux}$ is the running product column of auxiliary table. We are not including context in the permutation check as its not yet been implemented. We plan to add context to the memory in $v3.0$. 

Although we are only updating one element in the memory we need to include the rest of all elements in the permutation check as the overall memory value(we can loosely relate to a hash of memory elements though it is not appropriate) has changed. During the lookup we also stored the old memory values in the helper registers. For this instruction, the new memory value will differ from the old value on the *first* element only.

The `MSTORE` operation will shift the stack to left by one. The maximum degree of the above operation is $2$.

### MSTOREW

`MSTOREW` operation stores a word(4 consecutive field elements) from the stack into the specified memory address. The top element in the stack before the execution of this operation is the memory address which is been used to fetch a word and then the top four elements in the stack are saved into the specified memory address. The items are not removed from the stack. The diagram below illustrates this graphically.

![mstorew](../../assets/design/stack/io_operations/MSTOREW.png)

To facilitate this operation, we will need to perform a lookup into the memory table at the specified address using the current values of context and clock cycle registers described [here](https://maticnetwork.github.io/miden/design/chiplets/memory.html). 

Let's define a few intermediate variables to simplify constraint description:

$$
v_h = \alpha_0 + \alpha_1 \cdot 8 + \alpha_2 \cdot s_0 + \alpha_3 \cdot clk1
$$

$$
v_{n} = \sum_{i=0}^3\alpha_{i+4} \cdot s_i'
$$

$$
v_{o} = \sum_{i=0}^3\alpha_{i+8} \cdot m_i
$$

In the above:
- $v_h$ is a _common header_ which is a combination of unique identifier, memory address and clock cycle. The $8$ in the permutation check is the unique identifier of `MEMORY` operation which has been explained [here](../stack/unique_identifier.md#identifiers).
- $clk1$ is the clock cycle and $\alpha_0$, $\alpha_1$, $\alpha_2$ etc... are random values sent from the verifier to the prover for use in permutation checks.
- Values for the helper registers $m_0, m_1, m_2, m_3$ are provided by the VM non-deterministically and represent old memory at the specified address.
- $v_{n}$ and $v_{o}$ can be thought of as component of new and old memory value (whole word) in the permutation check calculation.  

The lookup in the table can be accomplished by including the value into the lookup product such that it follows the following constraint:

> $$
s_j' - s_{j+1} = 0 \space where \space j \in \{0, 1, 2, 3\} \text{ | degree } = 1\\
b_{aux}' \cdot \left(v_h + v_{n} + v_{o}\right) = b_{aux} \text{ | degree } = 2
$$

where $b_{aux}$ is the running product column of auxiliary table. We are not including context in the permutation check as its not yet been implemented. We plan to add context to memory in $v3.0$.

We store the old memory values in the helper registers during the lookup.

The `MSTOREW` operation will shift the stack to left by one. The maximum degree of the above operation is $2$.