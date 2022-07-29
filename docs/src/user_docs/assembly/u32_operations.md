## u32 operations
Miden assembly provides a set of instructions which can perform operations on regular 32-bit integers. These instructions are described in the tables below.

Most instructions have _checked_ variants. These variants ensure that input values are 32-bit integers, and fail if that's not the case. All other variants do not perform these checks, and thus, should be used only if the inputs are known to be 32-bit integers. Supplying inputs which are greater than or equal to $2^{32}$ to unchecked operations results in undefined behavior.

The primary benefit of using unchecked operations is performance: they can frequently be executed $2$ or $3$ times faster than their checked counterparts. In general, vast majority of the unchecked operations listed below can be executed in a single VM cycle.

For instructions where one or more operands can be provided as immediate parameters (e.g., `u32checked_add` and `u32checked_add.b`), we provide stack transition diagrams only for the non-immediate version. For the immediate version, it can be assumed that the operand with the specified name is not present on the stack.

### Conversions and tests

| Instruction   | Stack input | Stack output  | Notes                                      |
| ------------- | ----------- | ------------- | ------------------------------------------ |
| u32test       | [a, ...]    | [b, a, ...]   | $b \leftarrow \begin{cases} 1, & \text{if}\ a < 2^{32} \\ 0, & \text{otherwise}\ \end{cases}$ |
| u32testw      | [A, ...]    | [b, A, ...]   | $b \leftarrow \begin{cases} 1, & \text{if}\ \forall\ i \in \{0, 1, 2, 3\}\ a_i < 2^{32} \\ 0, & \text{otherwise}\ \end{cases}$ |
| u32assert <br> u32assert.1 | [a, ...]    | [a, ...]  | Fails if $a \ge 2^{32}$ |
| u32assert.2   | [b, a,...]  | [b, a,...] | Fails if $a \ge 2^{32}$ or $b \ge 2^{32}$ |
| u32assertw    | [A, ...]    | [A, ...]      | Fails if $\exists\ i \in \{0, 1, 2, 3\} \ni a_i \ge 2^{32}$ |
| u32cast       | [a, ...]    | [b, ...]      | $b \leftarrow a \mod 2^{32}$ |
| u32split      | [a, ...]    | [c, b, ...]   | $b \leftarrow a \mod 2^{32}$, $c \leftarrow \lfloor{a / 2^{32}}\rfloor$ |

### Arithmetic operations

| Instruction    | Stack input    | Stack output  | Notes                                      |
| -------------- | -------------- | ------------- | ------------------------------------------ |
| u32checked_add <br> u32checked_add.*b* | [b, a, ...] | [c, ...] | $c \leftarrow a + b$ <br> Fails if $max(a, b, c) \ge 2^{32}$ |
| u32overflowing_add <br> u32overflowing_add.*b* | [b, a, ...] | [d, c, ...] | $c \leftarrow (a + b) \mod 2^{32}$ <br> $d \leftarrow \begin{cases} 1, & \text{if}\ (a + b) \ge 2^{32} \\ 0, & \text{otherwise}\ \end{cases}$ <br> Undefined if $max(a, b) \ge 2^{32}$ |
| u32wrapping_add <br> u32wrapping_add.*b* | [b, a, ...] | [c, ...] |  $c \leftarrow (a + b) \mod 2^{32}$ <br> Undefined if $max(a, b) \ge 2^{32}$ |
| u32overflowing_add3 | [c, b, a, ...] | [e, d, ...]   | $d \leftarrow (a + b + c) \mod 2^{32}$, <br> $e \leftarrow \lfloor (a + b + c) / 2^{32}\rfloor$ <br> Undefined if $max(a, b, c) \ge 2^{32}$ <br> |
| u32checked_sub <br> u32checked_sub.*b* | [b, a, ...] | [c, ...] | $c \leftarrow (a - b)$ <br> Fails if $max(a, b) \ge 2^{32}$ or $a < b$ |
| u32overflowing_sub <br> u32overflowing_sub.*b* | [b, a, ...] | [d, c, ...] | $c \leftarrow (a - b) \mod 2^{32}$ <br> $d \leftarrow \begin{cases} 1, & \text{if}\ a < b \\ 0, & \text{otherwise}\ \end{cases}$ <br> Undefined if $max(a, b) \ge 2^{32}$ |
| u32wrapping_sub <br> u32wrapping_sub.*b* | [b, a, ...] | [c, ...] | $c \leftarrow (a - b) \mod 2^{32}$ <br> Undefined if $max(a, b) \ge 2^{32}$ |
| u32checked_mul <br> u32checked_mul.*b* | [b, a, ...] | [c, ...] | $c \leftarrow a \cdot b$ <br> Fails if $max(a, b, c) \ge 2^{32}$ |
| u32overflowing_mul <br> u32overflowing_mul.*b* | [b, a, ...] | [d, c, ...] | $c \leftarrow (a \cdot b) \mod 2^{32}$ <br> $d \leftarrow \lfloor(a \cdot b) / 2^{32}\rfloor$ <br> Undefined if $max(a, b) \ge 2^{32}$ |
| u32wrapping_mul <br> u32wrapping_mul.*b* | [b, a, ...] | [c, ...] | $c \leftarrow (a \cdot b) \mod 2^{32}$ <br> Undefined if $max(a, b) \ge 2^{32}$ |
| u32overflowing_madd | [b, a, c, ...] | [e, d, ...] | $d \leftarrow (a \cdot b + c) \mod 2^{32}$ <br> $e \leftarrow \lfloor(a \cdot b + c) / 2^{32}\rfloor$ <br> Undefined if $max(a, b, c) \ge 2^{32}$ |
| u32checked_div <br> u32checked_div.*b* | [b, a, ...] | [c, ...] | $c \leftarrow \lfloor a / b\rfloor$ <br> Fails if $max(a, b) \ge 2^{32}$ or $b = 0$ |
| u32unchecked_div <br> u32unchecked_div.*b* | [b, a, ...] | [c, ...] | $c \leftarrow \lfloor a / b\rfloor$ <br> Fails if $b = 0$ <br> Undefined if $max(a, b) \ge 2^{32}$ |
| u32checked_mod <br> u32checked_mod.*b* | [b, a, ...] | [c, ...] | $c \leftarrow a \mod b$ <br> Fails if $max(a, b) \ge 2^{32}$ or $b = 0$ |
| u32unchecked_mod <br> u32unchecked_mod.*b* | [b, a, ...] | [c, ...] | $c \leftarrow a \mod b$ <br> Fails if $b = 0$ <br> Undefined if $max(a, b) \ge 2^{32}$ |
| u32checked_divmod <br> u32checked_divmod.*b* | [b, a, ...] | [d, c, ...] | $c \leftarrow \lfloor a / b\rfloor$ <br> $d \leftarrow a \mod b$ <br> Fails if $max(a, b) \ge 2^{32}$ or $b = 0$ |
| u32unchecked_divmod <br> u32unchecked_divmod.*b* | [b, a, ...] | [d, c, ...] | $c \leftarrow \lfloor a / b\rfloor$ <br> $d \leftarrow a \mod b$ <br> Fails if $b = 0$ <br> Undefined if $max(a, b) \ge 2^{32}$ |

### Bitwise operations

| Instruction    | Stack input    | Stack output  | Notes                                      |
| -------------- | -------------- | ------------- | ------------------------------------------ |
| u32checked_and | [b, a, ...]    | [c, ...]      | Computes $c$ as a bitwise `AND` of binary representations of $a$ and $b$. <br> Fails if $max(a,b) \ge 2^{32}$ |
| u32checked_or  | [b, a, ...]    | [c, ...]      | Computes $c$ as a bitwise `OR` of binary representations of $a$ and $b$. <br> Fails if $max(a,b) \ge 2^{32}$ |
| u32checked_xor | [b, a, ...]    | [c, ...]      | Computes $c$ as a bitwise `XOR` of binary representations of $a$ and $b$. <br> Fails if $max(a,b) \ge 2^{32}$ |
| u32checked_not | [a, ...]       | [b, ...]      | Computes $b$ as a bitwise `NOT` of binary representation of $a$. <br> Fails if $a \ge 2^{32}$ |
| u32checked_shl <br> u32checked_shl.*b*         | [b, a, ...] | [c, ...]    | $c \leftarrow (a \cdot 2^b) \mod 2^{32}$ <br> Fails if $a \ge 2^{32}$ or $b > 31$ |
| u32unchecked_shl <br> u32unchecked_shl.*b*     | [b, a, ...] | [c, ...]    | $c \leftarrow (a \cdot 2^b) \mod 2^{32}$ <br> Undefined if $a \ge 2^{32}$ or $b > 31$ |
| u32checked_shr <br> u32checked_shr.*b* | [b, a, ...] | [c, ...] | $c \leftarrow \lfloor a/2^b \rfloor$ <br> Fails if $a \ge 2^{32}$ or $b > 31$ |
| u32unchecked_shr <br> u32unchecked_shr.*b* | [b, a, ...] | [c, ...] | $c \leftarrow \lfloor a/2^b \rfloor$ <br> Undefined if $a \ge 2^{32}$ or $b > 31$ |
| u32checked_rotl <br> u32checked_rotl.*b* | [b, a, ...] | [c, ...] | Computes $c$ by rotating a 32-bit representation of $a$ to the left by $b$ bits. <br> Fails if $a \ge 2^{32}$ or $b > 31$ |
| u32unchecked_rotl <br> u32unchecked_rotl.*b* | [b, a, ...] | [c, ...] | Computes $c$ by rotating a 32-bit representation of $a$ to the left by $b$ bits. <br> Undefined if $a \ge 2^{32}$ or $b > 31$ |
| u32checked_rotr <br> u32checked_rotr.*b* | [b, a, ...] | [c, ...] | Computes $c$ by rotating a 32-bit representation of $a$ to the right by $b$ bits. <br> Fails if $a \ge 2^{32}$ or $b > 31$ |
| u32unchecked_rotr <br> u32unchecked_rotr.*b* | [b, a, ...] | [c, ...] | Computes $c$ by rotating a 32-bit representation of $a$ to the right by $b$ bits. <br> Undefined if $a \ge 2^{32}$ or $b > 31$ |

### Comparison operations

| Instruction     | Stack input  | Stack output    | Notes                                      |
| --------------- | ------------ | --------------- | ------------------------------------------ |
| u32checked_eq <br> u32checked_eq.*b* | [b, a, ...] | [c, ...] | $c \leftarrow \begin{cases} 1, & \text{if}\ a=b \\ 0, & \text{otherwise}\ \end{cases}$ <br> Fails if $max(a, b) \ge 2^{32}$ <br> Note: unchecked version is not provided because it is equivalent to simple `eq`. |
| u32checked_neq <br> u32checked_neq.*b* | [b, a, ...] | [c, ...] | $c \leftarrow \begin{cases} 1, & \text{if}\ a \ne b \\ 0, & \text{otherwise}\ \end{cases}$ <br> Fails if $max(a, b) \ge 2^{32}$ <br> Note: unchecked version is not provided because it is equivalent to simple `neq`. |
| u32checked_lt   | [b, a, ...] | [c, ...]         | $c \leftarrow \begin{cases} 1, & \text{if}\ a < b \\ 0, & \text{otherwise}\ \end{cases}$ <br> Fails if $max(a, b) \ge 2^{32}$ |
| u32unchecked_lt | [b, a, ...] | [c, ...]         | $c \leftarrow \begin{cases} 1, & \text{if}\ a < b \\ 0, & \text{otherwise}\ \end{cases}$ <br> Undefined if $max(a, b) \ge 2^{32}$ |
| u32checked_lte  | [b, a, ...] | [c, ...]         | $c \leftarrow \begin{cases} 1, & \text{if}\ a \le b \\ 0, & \text{otherwise}\ \end{cases}$ <br> Fails if $max(a, b) \ge 2^{32}$ |
| u32unchecked_lte | [b, a, ...] | [c, ...]         | $c \leftarrow \begin{cases} 1, & \text{if}\ a \le b \\ 0, & \text{otherwise}\ \end{cases}$ <br> Undefined if $max(a, b) \ge 2^{32}$ |
| u32checked_gt    | [b, a, ...] | [c, ...]         | $c \leftarrow \begin{cases} 1, & \text{if}\ a > b \\ 0, & \text{otherwise}\ \end{cases}$ <br> Fails if $max(a, b) \ge 2^{32}$ |
| u32unchecked_gt  | [b, a, ...] | [c, ...]         | $c \leftarrow \begin{cases} 1, & \text{if}\ a > b \\ 0, & \text{otherwise}\ \end{cases}$ <br> Undefined if $max(a, b) \ge 2^{32}$ |
| u32checked_gte   | [b, a, ...] | [c, ...]         | $c \leftarrow \begin{cases} 1, & \text{if}\ a \ge b \\ 0, & \text{otherwise}\ \end{cases}$ <br> Fails if $max(a, b) \ge 2^{32}$ |
| u32unchecked_gte | [b, a, ...] | [c, ...]         | $c \leftarrow \begin{cases} 1, & \text{if}\ a \ge b \\ 0, & \text{otherwise}\ \end{cases}$ <br> Undefined if $max(a, b) \ge 2^{32}$ |
| u32checked_min   | [b, a, ...] | [c, ...]         | $c \leftarrow \begin{cases} a, & \text{if}\ a < b \\ b, & \text{otherwise}\ \end{cases}$ <br> Fails if $max(a, b) \ge 2^{32}$ |
| u32unchecked_min | [b, a, ...] | [c, ...]         | $c \leftarrow \begin{cases} a, & \text{if}\ a < b \\ b, & \text{otherwise}\ \end{cases}$ <br> Undefined if $max(a, b) \ge 2^{32}$ |
| u32checked_max   | [b, a, ...] | [c, ...]         | $c \leftarrow \begin{cases} a, & \text{if}\ a > b \\ b, & \text{otherwise}\ \end{cases}$ <br> Fails if $max(a, b) \ge 2^{32}$ |
| u32unchecked_max | [b, a, ...] | [c, ...]         | $c \leftarrow \begin{cases} a, & \text{if}\ a > b \\ b, & \text{otherwise}\ \end{cases}$ <br> Undefined if $max(a, b) \ge 2^{32}$ |
