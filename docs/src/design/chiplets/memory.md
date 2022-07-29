# Memory Chiplet

This note assumes some familiarity with [permutation checks](https://hackmd.io/@arielg/ByFgSDA7D).

Miden VM supports linear read-write random access memory. This memory is word-addressable, meaning, four values are located at each address, and we can read and write values to/from memory in batches of four. Each value is a field element in a $64$-bit prime field with modulus $2^{64} - 2^{32} + 1$. Memory address can be any field element.

In this note we describe the rational for selecting the above design and describe AIR constraints needed to support it.

The design makes extensive use of $16$-bit range checks. An efficient way of implementing such range checks is described [here](https://hackmd.io/D-vjBYtHQB2BuOB-HMUG5Q).

## Alternative designs

The simplest (and most efficient) alternative to the above design is contiguous write-once memory. To support such memory, we need to allocate just two trace columns as illustrated below.

![memory_alternative_design](../../assets/design/chiplets/memory/memory_alternative_design.png)

In the above, `addr` column holds memory address, and `value` column holds the field element representing the value stored at this address. Notice that some rows in this table are duplicated. This is because we need one row per memory access (either read or write operation). In the example above, value $b$ was first stored at memory address $1$, and then read from this address.

The AIR constraints for this design are very simple. First, we need to ensure that values in the `addr` column either remain the same or are incremented by $1$ as we move from one row to the next. This can be achieved with the following constraint:

$$
(a' - a) \cdot (a' - a - 1) = 0
$$

where $a$ is the value in `addr` column in the current row, and $a'$ is the value in this column in the next row.

Second, we need to make sure that if the value in the `addr` column didn't change, the value in the `value` column also remained the same (i.e., a value stored in a given address can only be set once). This can be achieved with the following constraint:

$$
(v' - v) \cdot (a' - a - 1) = 0
$$

where $v$ is the value in `value` column at the current row, and $v'$ is the value in this column in the next row.

In addition to the above constraints we would also need to impose constraints needed for permutation checks, but we omit these constraints here because they are needed for all designs described in this note.

As mentioned above, this approach is very efficient: each memory access requires just $2$ trace cells.

### Read-write memory

Write-once memory is tricky to work with, and many developers may need to climb a steep learning curve before they become comfortable working in this model. Thus, ideally, we'd want to support read-write memory. To do this, we need to introduce additional columns as illustrated below.

![memory_read_write](../../assets/design/chiplets/memory/memory_read_write.png)

In the above, we added `clk` column, which keeps track of the clock cycle at which memory access happened. We also need to differentiate between memory reads and writes. To do this, we now use two columns to keep track of the value: `old val` contains the value stored at the address before the operation, and `new val` contains the value after the operation. Thus, if `old val` and `new val` are the same, it was a read operation. If they are different, it was a write operation.

The AIR constraints needed to support the above structure are as follows.

We still need to make sure memory addresses are contiguous:

$$
(a' - a) \cdot (a' - a - 1) = 0
$$

Whenever memory address changes, we want to make sure that `old val` is set to $0$ (i.e., our memory is always initialized to $0$). This can be done with the following constraint:

$$
(a' - a) \cdot v_{old}' = 0
$$

On the other hand, if memory address doesn't change, we want to make sure that `new val` in the current row is the same as `old val` in the next row. This can be done with the following constraint:

$$
(1 + a - a') \cdot (v_{new} - v_{old}') = 0
$$

Lastly, we need to make sure that for the same address values in `clk` column are always increasing. One way to do this is to perform a $16$-bit range check on the value of $(i' - i - 1)$, where $i$ is the reference to `clk` column. However, this would mean that memory operations involving the same address must happen within $65536$ VM cycles from each other. This limitation would be difficult to enforce statically. To remove this limitation, we need to add two more columns as shown below:

![memory_limitation_diagram](../../assets/design/chiplets/memory/memory_limitation_diagram.png)

In the above column `d0` contains the lower $16$ bits of $(i' - i - 1)$ while `d1` contains the upper $16$ bits. The constraint needed to enforces this is as follows:

$$
(1 + a - a') \cdot ((i' - i - 1) - (2^{16} \cdot d_1' + d_0')) = 0
$$

Additionally, we need to apply $16$-bit range checks to columns `d0` and `d1`.

Overall, the cost of reading or writing a single element is now $6$ trace cells and $2$ $16$-bit range-checks.

### Non-contiguous memory

Requiring that memory addresses are contiguous may also be a difficult limitation to impose statically. To remove this limitation, we need to introduce one more column as shown below:

![memory_non_contiguous_memory](../../assets/design/chiplets/memory/memory_non_contiguous_memory.png)

In the above, the prover sets the value in the new column `t` to $0$ when the address doesn't change, and to $1 / (a' - a)$ otherwise. To simplify constraint description, we'll define variable $n$ computed as follows:

$$
n = (a' - a) \cdot t
$$

Then, to make sure the prover sets the value of $t$ correctly, we'll impose the following constraints:

$$
n^2 - n = 0 \\
(1 - n) \cdot  (a' - a) = 0
$$

The above constraints ensure that $n=1$ whenever the address changes, and $n=0$ otherwise. We can then define the following constraints to make sure values in columns `d0` and `d1` contain either the delta between addresses or between clock cycles.

| Condition | Constraint                                      | Comments                                                                                                                                   |
| --------- | ----------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| $n=1$     | $(a' - a) - (2^{16} \cdot d_1' + d_0') = 0$     | When the address changes, columns `d0` and `d1` at the next row should contain the delta between the old and the new address.              |
| $n=0$     | $(i' - i - 1) - (2^{16} \cdot d_1' + d_0') = 0$ | When the address remains the same, columns `d0` and `d1` at the next row should contain the delta between the old and the new clock cycle. |

We can combine the above constraints as follows:

$$
\left(n \cdot (a' - a) + (1 - n) \cdot (i' - i - 1)\right) - (2^{16} \cdot d_1' + d_0') = 0
$$

The above constraint, in combination with $16$-bit range checks against columns `d0` and `d1` ensure that values in `addr` and `clk` columns always increase monotonically, and also that column `addr` may contain duplicates, while values in `clk` column must be unique for a given address.

### Context separation

In many situations it may be desirable to assign memories to different context. For example, when making a cross-contract calls, memories of the caller and the callee should be separate. That is, caller should not be able to access the memory of the callee and vice-versa.

To accommodate this feature, we need to add one more column as illustrated below.

![memory_context_separation](../../assets/design/chiplets/memory/memory_context_separation.png)

This new column `ctx` should behave similarly to the address column: values in it should increase monotonically, and there could be breaks between them. We also need to change how the prover populates column `t`:

- If the context changes, `t` should be set to the inverse $(c' - c)$, where $c$ is a reference to column `ctx`.
- If the context remains the same but the address changes, column `t` should be set to the inverse of $(a' - a)$.
- Otherwise, column `t` should be set to $0$.

To simplify description of constraints, we'll define two variables $n_0$ and $n_1$ as follows:

$$
n_0 = (c' - c) \cdot t' \\
n_1 = (a' - a) \cdot t'
$$

Thus, $n_0 = 1$ when the context changes, and $0$ otherwise. Also, $(1 - n_0) \cdot n_1 = 1$ when context remains the same and address changes, and $0$ otherwise.

To make sure the prover sets the value of column `t` correctly, we'll need to impose the following constraints:

$$
n_0^2 - n_0 = 0 \\
(1 - n_0) \cdot  (c' - c) = 0 \\
(1 - n_0) \cdot (n_1^2 - n_1) = 0 \\
(1 - n_0) \cdot (1 - n_1) \cdot (a' - a) = 0
$$

We can then define the following constraints to make sure values in columns `d0` and `d1` contain the delta between contexts, between addresses, or between clock cycles.

| Condition            | Constraint                                      | Comments                                                                                                                                                         |
| -------------------- | ----------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| $n_0=1$              | $(c' - c) - (2^{16} \cdot d_1' + d_0') = 0$     | When the context changes, columns `d0` and `d1` at the next row should contain the delta between the old and the new contexts.                                   |
| $n_0=0$ <br> $n_1=1$ | $(a' - a) - (2^{16} \cdot d_1' + d_0') = 0$     | When the context remains the same but the address changes, columns `d0` and `d1` at the next row should contain the delta between the old and the new addresses. |
| $n_0=0$ <br> $n_1=0$ | $(i' - i - 1) - (2^{16} \cdot d_1' + d_0') = 0$ | When both the context and the address remain the same, columns `d0` and `d1` at the next row should contain the delta between the old and the new clock cycle.   |

We can combine the above constraints as follows:

$$
\left(n_0 \cdot (c' - c) + (1 - n_0) \cdot \left(n_1 \cdot (a - a') + (1 - n_1) \cdot (i' - i - 1) \right) \right) - (2^{16} \cdot d_1' + d_0') = 0
$$

The above constraint, in combination with $16$-bit range checks against columns `d0` and `d1` ensure that values in `ctx`, `addr`, and `clk` columns always increase monotonically, and also that columns `ctx` and `addr` may contain duplicates, while the values in column `clk` must be unique for a given combination of `ctx` and `addr`.

Notice that the above constraint has degree $5$.

## Miden approach

While the approach described above works, it comes at significant cost. Reading or writing a single value requires $8$ trace cells and $2$ $16$-bit range checks. Assuming a single range check requires roughly $2$ trace cells, the total number of trace cells needed grows to $12$. This is about $6$x worse the simple contiguous write-once memory described earlier.

Miden VM frequently needs to deal with batches of $4$ field elements, which we call _words_. For example, the output of Rescue Prime hash function is a single word. A single 256-bit integer value can be stored as two words (where each element contains one $32$-bit value). Thus, we can optimize for this common use case by making the memory _word-addressable_. That is $4$ field elements are located at each memory address, and we can read and write elements to/from memory in batches of four.

The layout of Miden VM memory table is shown below:

![memory_miden_vm_layout](../../assets/design/chiplets/memory/memory_miden_vm_layout.png)

where:

- `ctx` contains context ID. Values in this column must increase monotonically but there can be gaps between two consecutive values of up to $2^{32}$. Also, two consecutive values can be the same. In AIR constraint description below, we refer to this column as $c$.
- `addr` contains memory address. Values in this column must increase monotonically for a given context but there can be gaps between two consecutive values of up to $2^{32}$. Also, two consecutive values can be the same. In AIR constraint description below, we refer to this column as $a$.
- `clk` contains clock cycle at which the memory operation happened. Values in this column must increase monotonically for a given context and memory address but there can be gaps between two consecutive values of up to $2^{32}$. In AIR constraint description below, we refer to this column as $i$.
- `u0, u1, u2, u3` columns contain field elements stored at a given context/address/clock cycle prior to the memory operation.
- `v0, v1, v2, v3` columns contain field elements stored at a given context/address/clock cycle after the memory operation.
- Columns `d0` and `d1` contain lower and upper $16$ bits of the delta between two consecutive context IDs, addresses, or clock cycles. Specifically:
  - When the context changes, these columns contain $(c' - c)$.
  - When the context remains the same but the address changes, these columns contain $(a' - a)$.
  - When both the context and the address remain the same, these columns contain $(i' - i - 1)$.
- Column `t` contains the inverse of the delta between two consecutive context IDs, addresses, or clock cycles. Specifically:
  - When the context changes, this column contains the inverse of $(c' - c)$.
  - When the context remains the same but the address changes, this column contains the inverse of $(a' - a)$.
  - When both the context and the address remain the same, this column contains the inverse of $(i' - i - 1)$.

For every memory access operation (i.e., read or write), a new row is added to the memory table. For read operations values in `u` columns are equal to the corresponding values in `v` columns. For write operations, the values may be different. Memory values are always initialized to $0$.

The amortized cost of reading or writing a single value is between $4$ and $5$ trace cells (this accounts for the trace cells needed for $16$-bit range checks). Thus, from performance standpoint, this approach is roughly $2.5$x worse than the simple contiguous write-once memory described earlier. However, our view is that this trade-off is worth it given that this approach provides read-write memory, context separation, and eliminates the contiguous memory requirement.

### AIR constraints

To simplify description of constraints, we'll define two variables $n_0$ and $n_1$ as follows:

$$
n_0 = (c' - c) \cdot t \\
n_1 = (a' - a) \cdot t
$$

To make sure the prover sets the value of column `t` correctly, we'll need to impose the following constraints:

$$
n_0^2 - n_0 = 0 \\
(1 - n_0) \cdot  (c' - c) = 0 \\
(1 - n_0) \cdot (n_1^2 - n_1) = 0 \\
(1 - n_0) \cdot (1 - n_1) \cdot (a' - a) = 0
$$

The above constraints guarantee that when context changes $n_0 = 1$, when context remains the same but address changes $(1 - n_0) \cdot n_1 = 1$, and when neither the context nor the address change, $(1 - n_0) \cdot (1 - n_1) = 1$.

To enforce the values of context ID, address, and clock cycle grow monotonically as described in the previous section, we define the following constraint.

$$
\left(n_0 \cdot (c' - c) + (1 - n_0) \cdot \left(n_1 \cdot (a - a') + (1 - n_1) \cdot (i' - i - 1) \right) \right) - (2^{16} \cdot d_1' + d_0') = 0
$$

In addition to this constraint, we also need to make sure that values in registers $d_0$ and $d_1$ are less than $2^{16}$, and this can be done with permutation-based range checks.

Next, we need to make sure that values at a given memory address are always initialized to $0$. This can be done with the following constraint:

$$
(n_0 + (1 - n_0) \cdot n_1) \cdot u_i' = 0
$$

where $i \in \{0, 1, 2, 3\}$. Thus, when either the context changes, or the address changes, values in $u_i$ columns are guaranteed to be zeros.

Lastly, we need to make sure that for the same context/address combination, the $v_i$ columns of the current row are equal to the corresponding $u_i$ columns of the next row. This can be done with the following constraints:

$$
(1 - n_0) \cdot (1 - n_1) \cdot (u_i' - v_i) = 0
$$

where $i \in \{0, 1, 2, 3\}$.

Notice that the maximum degree for all constraints described above is $5$.

#### Memory row value

To use the above table in permutation checks, we need to reduce each row of the memory table to a single value. This can be done like so:

$$
v = \alpha_0 + \alpha_1 \cdot c + \alpha_2 \cdot a + \alpha_3 \cdot i + \sum_{j=0}^3(\alpha_{j+4} \cdot u_j) + \sum_{j=0}^3(\alpha_{j+8} \cdot v_j)
$$

where $\alpha_0$, $\alpha_1$, $\alpha_2$, etc. are random values sent from the verifier to the prover for use in permutation checks.

### Load and store operations

To move elements between the stack and the memory, Miden VM provides two operations `MLOAD` and `MSTORE`. Semantic of these operations are described below.

#### Reading from memory

`MLOAD` operation is used to move values from memory to the top of the stack. This operation works as follows:

1. Pop the top element from the stack and interpret it as memory address.
2. Perform a lookup into the memory table at the specified address and using the current values of context and clock cycle registers. This creates a row in the lookup table corresponding to the load operation.
3. Overwrite the top four stack items with values located at the specified memory address.

Graphically, this looks like so:

![memory_reading_memory](../../assets/design/chiplets/memory/memory_reading_memory.png)

Note that as a result of this operation the stack is shifted to the left by one.

Denoting stack registers as $s_j$, clock cycle register as $i$, and context register as $c$, we can compute the lookup row value as follows:

$$
v = \alpha_0 + \alpha_1 \cdot c + \alpha_2 \cdot s_0 + \alpha_3 \cdot i + \sum_{j=0}^3(\alpha_{j+4} \cdot s_j') + \sum_{j=0}^3(\alpha_{j+8} \cdot s_j')
$$

where $\alpha_0$, $\alpha_1$, $\alpha_2$, etc. are random values sent from the verifier to the prover for use in permutation checks.

Note that the values from the top of the stack are added into the row twice: once for "old" values and once for "new" values. We can do this because old and new values in the memory table row corresponding the load operation are the same.

#### Writing to memory

`MSTORE` operation is used to move values from the top of the stack to the memory. This operation works as follows:

1. Pop the top element from the stack and interpret it as memory address.
2. Perform a lookup into the memory table at the specified address and using the current values of context and clock cycle registers. This creates a row in the lookup table corresponding to the store operation.

Graphically, this looks like so:

![memory_writing_to_memory](../../assets/design/chiplets/memory/memory_writing_to_memory.png)

Note that as a result of this operation the stack is shifted to the left by one, and the values saved to memory remain on the stack.

Denoting stack registers as $s_j$, helper registers as $h_j$, clock cycle register as $i$, and context register as $c$, we can compute the lookup row value as follows:

$$
v = \alpha_0 + \alpha_1 \cdot c + \alpha_2 \cdot s_0 + \alpha_3 \cdot i + \sum_{j=0}^3(\alpha_{j+4} \cdot h_j) + \sum_{j=0}^3(\alpha_{j+8} \cdot s_j')
$$

where $\alpha_0$, $\alpha_1$, $\alpha_2$ etc... are random values sent from the verifier to the prover for use in permutation checks. Values for the helper registers $h_0, ...,  h_3$ are provided by the VM non-deterministically.

We also need to make sure that the saved values remained on the stack. This can be done with the following constraint:

$$
s_i' - s_{i + 1} = 0
$$

where $i \in \{0, 1, 2, 3\}$.
