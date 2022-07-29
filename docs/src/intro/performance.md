# Performance
The benchmarks below should be viewed only as a rough guide for expected future performance. The reasons for this are twofold:
1. Not all constraints have been implemented yet, and we expect that there will be some slowdown once constraint evaluation is completed.
2. Many optimizations have not been applied yet, and we expect that there will be some speedup once we dedicate some time to performance optimizations.

Overall, we don't expect the benchmarks to change significantly, but there will definitely be some deviation from the below numbers in the future.

A few general notes on performance:

* Execution time is dominated by proof generation time. In fact, the time needed to run the program is usually under 1% of the time needed to generate the proof.
* Proof verification time is really fast. In most cases it is under 1 ms, but sometimes gets as high as 2 ms or 3 ms.
* Proof generation process is dynamically adjustable. In general, there is a trade-off between execution time, proof size, and security level (i.e. for a given security level, we can reduce proof size by increasing execution time, up to a point).
* Both proof generation and proof verification times are greatly influenced by the hash function used in the STARK protocol. In the benchmarks below, we use BLAKE3, which is a really fast hash function.

## Single-core prover performance
When executed on a single CPU core, the current version of Miden VM operates at around 10 - 15 KHz. In the benchmarks below, the VM executes a Fibonacci calculator program on Apple M1 Pro CPU in a single thread. The generated proofs have a target security level of 96 bits.

| VM cycles       | Execution time | Proving time | RAM consumed  | Proof size |
| :-------------: | :------------: | :----------: | ------------- | ---------- |
| 2<sup>10</sup>  |  2 ms          | 93 ms        |
| 2<sup>12</sup>  |  4 ms          | 260 ms       |
| 2<sup>14</sup>  |  12 ms         | 1.04 sec     |
| 2<sup>16</sup>  |  40 ms         | 4.3 sec      |
| 2<sup>18</sup>  |  122 ms        | 18.5 sec     |
| 2<sup>20</sup>  |  320 ms        | 81 sec       |

As can be seen from the above, proving time roughly doubles with every doubling in the number of cycles, but proof size grows much slower.

We can also generate proofs at a higher security level. The cost of doing so is roughly doubling of proving time and roughly 40% increase in proof size. In the benchmarks below, the same Fibonacci calculator program was executed on Apple M1 Pro CPU at 128-bit target security level:

| VM cycles       | Execution time | Proving time | RAM consumed  | Proof size |
| :-------------: | -------------- | ------------ | ------------- | ---------- |
| 2<sup>10</sup>  |
| 2<sup>12</sup>  |
| 2<sup>14</sup>  |
| 2<sup>16</sup>  |
| 2<sup>18</sup>  |
| 2<sup>20</sup>  |

## Multi-core prover performance
STARK proof generation is massively parallelizable. Thus, by taking advantage of multiple CPU cores we can dramatically reduce proof generation time. For example, when executed on a high-end 8-core CPU (Apple M1 Pro), the current version of Miden VM operates at around 80 KHz. And when executed on a high-end 64-core CPU (Amazon Graviton 3), the VM operates at around 320 KHz.

In the benchmarks below, the VM executes the same Fibonacci calculator program for 2<sup>20</sup> cycles at 96-bit target security level:

| Machine                        | Execution time | Proving time |
| ------------------------------ | :------------: | :----------: |
| Apple M1 Pro (8 threads)       | 320 ms         | 12.6 sec     |
| Amazon Graviton 3 (64 threads) | 380 ms         | 3 sec        |