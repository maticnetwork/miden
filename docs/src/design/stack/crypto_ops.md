# Cryptographic operations
In this section we describe the AIR constraint for Miden VM cryptographic operations.

Cryptographic operations in Miden VM are performed by the [Hash chiplet](../chiplets/hasher.md). Communication between the stack and the hash chiplet is accomplished via the chiplet bus $b_{chip}$. To make requests to and to read results from the chiplet bus we need to divide its current value by the value representing the request.

Thus, to describe AIR constraints for the cryptographic operations, we need to define how to compute these input and output values within the stack. We do this in the following sections.

## RPPERM
The `RPPERM` operation applies Rescue Prime permutation to the top $12$ elements of the stack. The stack is assumed to be arranged so that the $8$ elements of the rate are at the top of the stack. The capacity word follows, with the number of elements to be hashed at the deepest position in stack. The diagram below illustrates this graphically.

![rpperm](../../assets/design/stack/cryptographic_operations/RPPERM.png)

In the above, $r$ (located in the helper register $h_0$) is the row address from the hash chiplet set by the prover nondeterministically.

For the `RPPERM` operation, we define input and output values as follows:

$$
v_{input} = \alpha_0 + \alpha_1 \cdot op_{linhash} + \alpha_2 \cdot h_0 + \sum_{j=0}^{11} (\alpha_{j+4} \cdot s_{11-i})
$$

$$
v_{output} = \alpha_0 + \alpha_1 \cdot op_{retstate} + \alpha_2 \cdot (h_0 + 7) + \sum_{j=0}^{11} (\alpha_{i+4} \cdot s_{11-i}')
$$

In the above, $op_{linhash}$ and $op_{retstate}$ are the unique [operation label](../chiplets/main.md#operation-labels) for initiating a linear hash and reading the full state of the hasher respectively. Also note that the term for $\alpha_3$ is missing from the above expressions because for Rescue Prime permutation computation the index column is expected to be set to $0$.

Using the above values, we can describe constraint for the chiplet bus column as follows:

>$$
b_{chip}' \cdot v_{input} \cdot v_{output} = b_{chip} \text{ | degree } = 3
$$

The above constrain enforces that the specified input and output rows must be present in the trace of the hash chiplet, and that they must be exactly $7$ rows apart.

The effect of this operation on the rest of the stack is:
* **No change** starting from position $12$.

## MPVERIFY
The `MPVERIFY` operation verifies that a Merkle path from the specified node resolves to the specified root. This operation can be used to prove that the prover knows a path in the specified Merkle tree which starts with the specified node.

Prior to the operation, the stack is expected to be arranged as follows (from the top):
- Depth of the path, 1 element.
- Index of the node, 1 element.
- Value of the node, 4 elements.
- Root of the tree, 4 elements.

The Merkle path itself is expected to be provided by the prover non-deterministically (via the advice provider). If the prover is not able to provide the required path, the operation fails. Otherwise, the state of the stack does not change. The diagram below illustrates this graphically.

![mpverify](../../assets/design/stack/cryptographic_operations/MPVERIFY.png)

In the above, $r$ (located in the helper register $h_0$) is the row address from the hash chiplet set by the prover nondeterministically.

For the `MPVERIFY` operation, we define input and output values as follows:

$$
v_{input} = \alpha_0 + \alpha_1 \cdot op_{mpver} + \alpha_2 \cdot r + \alpha_3 \cdot i + \sum_{j=0}^3 \alpha_{j+8} \cdot s_{6 - j}
$$

$$
v_{output} = \alpha_0 + \alpha_1 \cdot op_{rethash} + \alpha_2 \cdot (h_0 + 8 \cdot s_0 - 1) + \sum_{j=0}^3\alpha_{j + 8} \cdot s_{10 - j}
$$

In the above, $op_{mpver}$ and $op_{rethash}$ are the unique [operation label](../chiplets/main.md#operation-labels) for initiating a Merkle path verification computation and reading the hash result respectively. The sum expression for inputs computes the value of the leaf node, while the sum expression for the output computes the value of the tree root.

Using the above values, we can describe constraint for the chiplet bus column as follows:

>$$
b_{chip}' \cdot v_{input} \cdot v_{output} = b_{chip} \text{ | degree } = 3
$$

The above constrain enforces that the specified input and output rows must be present in the trace of the hash chiplet, and that they must be exactly $8 \cdot d - 1$ rows apart, where $d$ is the depth of the node.

The effect of this operation on the rest of the stack is:
* **No change** starting from position $0$.

## MRUPDATE
The `MRUPDATE` operation computes a new root of a Merkle tree where a node at the specified position is updated to the specified value.
    
The stack is expected to be arranged as follows (from the top):
- depth of the node, 1 element
- index of the node, 1 element
- old value of the node, 4 element
- new value of the node, 4 element
- current root of the tree, 4 elements

The Merkle path for the node is expected to be provided by the prover non-deterministically (via advice sets). At the end of the operation, the old node value is replaced with the old root value computed based on the provided path, the new node value is replaced by the new root value computed based on the same path. Everything else on the stack remains the same. The diagram below illustrates this graphically.

![mrupdate](../../assets/design/stack/cryptographic_operations/MRUPDATE.png)

In the above, $r$ (located in the helper register $h_0$) is the row address from the hash chiplet set by the prover nondeterministically.

For the `MRUPDATE` operation, we define input and output values as follows:

$$
v_{inputold} = \alpha_0 + \alpha_1 \cdot op_{mruold} + \alpha_2 \cdot r + \alpha_3 \cdot i + \sum_{j=0}^3\alpha_{j + 8} \cdot s_{5 - j}
$$

$$
v_{outputold} = \alpha_0 + \alpha_1 \cdot op_{rethash} + \alpha_2 \cdot (r + 8 \cdot d - 1) + \sum_{j=0}^3\alpha_{j + 8} \cdot s_{13 - j}
$$

$$
v_{inputnew} = \alpha_0 + \alpha_1 \cdot op_{mrunew} + \alpha_2 \cdot (r + 8 \cdot d) + \alpha_3 \cdot i + \sum_{j=0}^3\alpha_{j + 8} \cdot s_{9 - j}
$$

$$
v_{outputnew} = \alpha_0 + \alpha_1 \cdot op_{rethash} + \alpha_2 \cdot (r + 2 \cdot 8 \cdot d - 1) + \sum_{j=0}^3\alpha_{j + 8} \cdot s_{9 - j}'
$$

In the above, the first two expressions correspond to inputs and outputs for verifying the Merkle path between the old node value and the old tree root, while the last two expressions correspond to inputs and outputs for verifying the Merkle path between the new node value and the new tree root. Hash chiplet ensures the same set of sibling nodes are uses in both of these computations.

The $op_{mruold}$, $op_{mrunew}$, and $op_{rethash}$ are the unique [operation label](../chiplets/main.md#operation-labels) used by the above computations.

> $$
b_{chip}' \cdot v_{inputold} \cdot v_{outputold} \cdot v_{inputnew} \cdot v_{outputnew} = b_{chip} \text{ | degree } = 5
$$

The above constrain enforces that the specified input and output rows for both, the old and the new node/root combinations, must be present in the trace of the hash chiplet, and that they must be exactly $8 \cdot d - 1$ rows apart, where $d$ is the depth of the node. It also ensure that the computation for the old node/root combination is immediately followed by the computation for the new node/root combination.

The effect of this operation on the rest of the stack is:
* **No change** for positions $0$ and $1$.
* **No change** for positions starting from $10$.