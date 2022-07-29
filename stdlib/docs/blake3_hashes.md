
## std::crypto::hashes::blake3
| Procedure | Description |
| ----------- | ------------- |
| hash.4 |  Blake3 2-to-1 hash function, which takes 64 -bytes input and produces 32 -bytes output digest<br /><br /> Expected stack state:<br /><br /> [msg0, msg1, msg2, msg3, msg4, msg5, msg6, msg7, msg8, msg9, msg10, msg11, msg12, msg13, msg14, msg15]<br /><br /> msg`i` -> 32 -bit message word \| i ∈ [0, 16)<br /><br /> Output stack state:<br /><br /> [dig0, dig1, dig2, dig3, dig4, dig5, dig6, dig7]<br /><br /> dig`i` -> 32 -bit digest word \| i ∈ [0, 8) |
