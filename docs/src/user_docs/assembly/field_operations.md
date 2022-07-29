## Field operations
Miden assembly provides a set of instructions which can perform operations with raw field elements. These instructions are described in the tables below.

While most operations place no restrictions on inputs, some operations expect inputs to be binary values, and fail if executed with non-binary inputs.

For instructions where one or more operands can be provided as immediate parameters (e.g., `add` and `add.b`), we provide stack transition diagrams only for the non-immediate version. For the immediate version, it can be assumed that the operand with the specified name is not present on the stack.

### Assertions and tests

| Instruction      | Stack_input | Stack_output  | Notes                         |
| ---------------- | ----------- | ------------- | ----------------------------- |
| assert           | [a, ...]    | [...]         | If $a = 1$, removes it from the stack. <br> Fails if $a \ne 1$ |
| assert.eq        | [b, a, ...] | [...]         | If $a = b$, removes them from the stack. <br> Fails if $a \ne b$ |

### Arithmetic and Boolean operations

| Instruction      | Stack_input | Stack_output  | Notes                         |
| ---------------- | ----------- | ------------- | ----------------------------- |
| add <br> add.*b* | [b, a, ...] | [c, ...]      | $c \leftarrow (a + b) \mod p$          |
| sub <br> sub.*b* | [b, a, ...] | [c, ...]      | $c \leftarrow (a - b) \mod p$          |
| mul <br> mul.*b* | [b, a, ...] | [c, ...]      | $c \leftarrow (a \cdot b) \mod p$      |
| div <br> div.*b* | [b, a, ...] | [c, ...]      | $c \leftarrow (a \cdot b^{-1}) \mod p$ <br> Fails if $b = 0$ |
| neg              | [a, ...]    | [b, ...]      | $b \leftarrow -a \mod p$               |
| inv              | [a, ...]    | [b, ...]      | $b \leftarrow a^{-1} \mod p$ <br> Fails if $a = 0$ |
| checked_pow2     | [a, ...]    | [b, ...]      | $b \leftarrow 2^a$ <br> Fails if $a > 63$ |
| unchecked_pow2   | [a, ...]    | [b, ...]      | $b \leftarrow 2^a$ <br> Undefined if $a > 63$ |
| not              | [a, ...]    | [b, ...]      | $b \leftarrow 1 - a$ <br> Fails if $a > 1$ |
| and              | [b, a, ...] | [c, ...]      | $c \leftarrow a \cdot b$ <br> Fails if $max(a, b) > 1$ |
| or               | [b, a, ...] | [c, ...]      | $c \leftarrow a + b - a \cdot b$ <br> Fails if $max(a, b) > 1$ |
| xor              | [b, a, ...] | [c, ...]      | $c \leftarrow a + b - 2 \cdot a \cdot b$ <br> Fails if $max(a, b) > 1$ |

### Comparison operations

| Instruction      | Stack_input | Stack_output   | Notes                         |
| ---------------- | ----------- | -------------- | ----------------------------- |
| eq <br> eq.*b*   | [b, a, ...] | [c, ...]       | $c \leftarrow \begin{cases} 1, & \text{if}\ a=b \\ 0, & \text{otherwise}\ \end{cases}$ |
| neq <br> neq.*b* | [b, a, ...] | [c, ...]       | $c \leftarrow \begin{cases} 1, & \text{if}\ a \ne b \\ 0, & \text{otherwise}\ \end{cases}$ |
| lt               | [b, a, ...] | [c, ...]       | $c \leftarrow \begin{cases} 1, & \text{if}\ a < b \\ 0, & \text{otherwise}\ \end{cases}$ |
| lte              | [b, a, ...] | [c, ...]       | $c \leftarrow \begin{cases} 1, & \text{if}\ a \le b \\ 0, & \text{otherwise}\ \end{cases}$ |
| gt               | [b, a, ...] | [c, ...]       | $c \leftarrow \begin{cases} 1, & \text{if}\ a > b \\ 0, & \text{otherwise}\ \end{cases}$ |
| gte              | [b, a, ...] | [c, ...]       | $c \leftarrow \begin{cases} 1, & \text{if}\ a \ge b \\ 0, & \text{otherwise}\ \end{cases}$ |
| eqw              | [A, B, ...] | [c, A, B, ...] | $c \leftarrow \begin{cases} 1, & \text{if}\ a_i = b_i \; \forall i \in \{0, 1, 2, 3\} \\ 0, & \text{otherwise}\ \end{cases}$ |


