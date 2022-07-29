## Stack manipulation
Miden VM stack is a push-down stack of field elements. The stack has a maximum depth of $2^{16}$, but only the top $16$ elements are directly accessible via the instructions listed below.

In addition to the typical stack manipulation instructions such as `drop`, `dup`, `swap` etc., Miden assembly provides several conditional instructions which can be used to manipulate the stack based on some condition - e.g., conditional swap `cswap` or conditional drop `cdrop`.

| Instruction | Stack_input       | Stack_output       | Notes                                      |
| ----------- | ----------------- | ------------------ | ------------------------------------------ |
| drop        | [a, ... ]         | [ ... ]            | Deletes the top stack item.                |
| dropw       | [A, ... ]         | [ ... ]            | Deletes a word (4 elements) from the top of the stack. |
| padw        | [ ... ]           | [0, 0, 0, 0, ... ] | Pushes four $0$ values onto the stack. <br> Note: simple `pad` is not provided because `push.0` does the same thing. |
| dup.*n*     | [ ..., a, ... ]   | [a, ..., a, ... ]  | Pushes a copy of the $n$th stack item onto the stack. `dup` and `dup.0` are the same instruction. Valid for $n \in \{0, ..., 15\}$ |
| dupw.*n*    | [ ..., A, ... ]   | [A, ..., A, ... ]  | Pushes a copy of the $n$th stack word onto the stack. `dupw` and `dupw.0` are the same instruction. Valid for $n \in \{0, 1, 2, 3\}$ |
| swap.*n*    | [a, ..., b, ... ] | [b, ..., a, ... ]  | Swaps the top stack item with the $n$th stack item. `swap` and `swap.1` are the same instruction. Valid for $n \in \{1, ..., 15\}$ |
| swapw.*n*   | [A, ..., B, ... ] | [B, ..., A, ... ]  | Swaps the top stack word with the $n$th stack word. `swapw` and `swapw.1` are the same instruction. Valid for $n \in \{1, 2, 3\}$ |
| movup.*n*   | [ ..., a, ... ]   | [a, ... ]          | Moves the $n$th stack item to the top of the stack. Valid for $n \in \{2, ..., 15\}$ |
| movupw.*n*  | [ ..., A, ... ]   | [A, ... ]          | Moves the $n$th stack word to the top of the stack. Valid for $n \in \{2, 3\}$ |
| movdn.*n*   | [a, ... ]         | [ ..., a, ... ]    | Moves the top stack item to the $n$th position of the stack. Valid for $n \in \{2, ..., 15\}$ |
| movdnw.*n*  | [A, ... ]         | [ ..., A, ... ]     | Moves the top stack word to the $n$th word position of the stack. Valid for $n \in \{2, 3\}$ |

### Conditional manipulation

| Instruction | Stack_input       | Stack_output       | Notes                                      |
| ----------- | ----------------- | ------------------ | ------------------------------------------ |
| cswap       | [c, b, a, ... ]   | [e, d, ... ]       | $d = \begin{cases} a, & \text{if}\ c = 0 \\ b, & \text{if}\ c = 1\ \end{cases}$ $e = \begin{cases} b, & \text{if}\ c = 0 \\ a, & \text{if}\ c = 1\ \end{cases}$  <br> Fails if $c > 1$ |
| cswapw      | [c, B, A, ... ]   | [E, D, ... ]       | $D = \begin{cases} A, & \text{if}\ c = 0 \\ B, & \text{if}\ c = 1\ \end{cases}$ $E = \begin{cases} B, & \text{if}\ c = 0 \\ A, & \text{if}\ c = 1\ \end{cases}$  <br> Fails if $c > 1$ |
| cdrop       | [c, b, a, ... ]   | [d, ... ]          | $d = \begin{cases} a, & \text{if}\ c = 0 \\ b, & \text{if}\ c = 1\ \end{cases}$ <br> Fails if $c > 1$ |
| cdropw      | [c, B, A, ... ]   | [D, ... ]          | $D = \begin{cases} A, & \text{if}\ c = 0 \\ B, & \text{if}\ c = 1\ \end{cases}$ <br> Fails if $c > 1$ |

