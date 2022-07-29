//! This module is automatically generated during build time and should not be modified manually.

/// An array of modules defined in Miden standard library.
///
/// Entries in the array are tuples containing module namespace and module source code.
#[rustfmt::skip]
pub const MODULES: [(&str, &str); 6] = [
// ----- std::crypto::hashes::blake3 --------------------------------------------------------------
("std::crypto::hashes::blake3", "# Initializes four memory addresses, provided for storing initial 4x4 blake3 
# state matrix ( i.e. 16 elements each of 32 -bit ), for computing blake3 2-to-1 hash
#
# Expected stack state:
#
# [state_0_3_addr, state_4_7_addr, state_8_11_addr, state_12_15_addr]
#
# Note, state_`i`_`j`_addr -> absolute address of {state[i], state[i+1], state[i+2], state[i+3]} in memory | j = i+3
#
# Final stack state:
#
# [...]
#
# Initialized stack state is written back to provided memory addresses.
#
# Functionally this routine is equivalent to https://github.com/itzmeanjan/blake3/blob/f07d32e/include/blake3.hpp#L1709-L1713
proc.initialize
    push.0xA54FF53A.0x3C6EF372.0xBB67AE85.0x6A09E667
    movup.4
    popw.mem

    push.0x5BE0CD19.0x1F83D9AB.0x9B05688C.0x510E527F
    movup.4
    popw.mem

    push.0xA54FF53A.0x3C6EF372.0xBB67AE85.0x6A09E667
    movup.4
    popw.mem

    push.11.64.0.0
    movup.4
    popw.mem
end

# Permutes ordered message words, kept on stack top ( = sixteen 32 -bit BLAKE3 words )
#
# Expected stack top: 
#
# [s0, s1, s2, s3, s4, s5, s6, s7, s8, s9, s10, s11, s12, s13, s14, s15]
#
# After permutation, stack top:
#
# [s2, s6, s3, s10, s7, s0, s4, s13, s1, s11, s12, s5, s9, s14, s15, s8]
#
# See https://github.com/itzmeanjan/blake3/blob/f07d32ec10cbc8a10663b7e6539e0b1dab3e453b/include/blake3.hpp#L1623-L1639
# and https://github.com/maticnetwork/miden/pull/313#discussion_r922627984
proc.permute_msg_words
    movdn.7
    movup.5
    movdn.2
    movup.4
    movdn.7
    swapw.3
    swap
    movdn.7
    swapdw
    movup.2
    movdn.7
    swapw
    swapw.2
    movup.3
    movdn.6
    movdn.5
    movup.3
    swapw
    movup.3
    swapdw
end

# Given blake3 state matrix on stack top ( in order ) as 16 elements ( each of 32 -bit ),
# this routine computes output chaining value i.e. 2-to-1 hashing digest.
#
# Expected stack state:
#
# [state0, state1, state2, state3, state4, state5, state6, state7, state8, state9, state10, state11, state12, state13, state14, state15]
#
# After finalizing, stack should look like
#
# [dig0, dig1, dig2, dig3, dig4, dig5, dig6, dig7]
#
# See https://github.com/BLAKE3-team/BLAKE3/blob/da4c792/reference_impl/reference_impl.rs#L116-L119 ,
# you'll notice I've skipped executing second statement in loop body of above hyperlinked implementation,
# that's because it doesn't dictate what output of 2-to-1 hash will be.
proc.finalize
    movup.8
    u32checked_xor

    swap
    movup.8
    u32checked_xor
    swap

    movup.2
    movup.8
    u32checked_xor
    movdn.2

    movup.3
    movup.8
    u32checked_xor
    movdn.3

    movup.4
    movup.8
    u32checked_xor
    movdn.4

    movup.5
    movup.8
    u32checked_xor
    movdn.5

    movup.6
    movup.8
    u32checked_xor
    movdn.6

    movup.7
    movup.8
    u32checked_xor
    movdn.7
end

# Given blake3 state matrix ( total 16 elements, each of 32 -bit ) and 
# 8 message words ( each of 32 -bit ), this routine performs column-wise mixing
# of message words into blake3 hash state.
#
# Functionality wise this routine is equivalent to https://github.com/BLAKE3-team/BLAKE3/blob/da4c792/reference_impl/reference_impl.rs#L55-L59
#
# Expected stack state:
#
# [state0_3_addr, state4_7_addr, state8_11_addr, state12_15_addr, m0, m1, m2, m3, m4, m5, m6, m7]
#
# Note, state_`i`_`j`_addr -> absolute address of {state[i], state[i+1], state[i+2], state[i+3]} in memory | j = i+3
#
# Meaning four consecutive blake3 state words can be read from memory easily.
#
# Final stack state:
#
# [state0, state1, state2, state3, state4, state5, state6, state7, state8, state9, state10, state11, state12, state13, state14, state15]
#
# i.e. whole blake3 state is placed on stack ( in order ).
proc.columnar_mixing.1
    swapw.2
    swapw

    movup.7
    movup.6
    movup.5
    movup.4

    storew.local.0

    movup.9
    loadw.mem
    movup.8
    pushw.mem

    movup.8
    dup.5
    u32overflowing_add3
    drop

    swap
    movup.8
    dup.6
    u32overflowing_add3
    drop
    swap

    movup.2
    dup.6
    movup.9
    u32overflowing_add3
    drop
    movdn.2

    movup.3
    dup.7
    movup.9
    u32overflowing_add3
    drop
    movdn.3

    movup.9
    pushw.mem

    dup.4
    u32checked_xor
    u32unchecked_rotr.16
    
    swap
    dup.5
    u32checked_xor
    u32unchecked_rotr.16
    swap

    movup.2
    dup.6
    u32checked_xor
    u32unchecked_rotr.16
    movdn.2

    movup.3
    dup.7
    u32checked_xor
    u32unchecked_rotr.16
    movdn.3

    movup.12
    pushw.mem

    dup.4
    u32wrapping_add

    swap
    dup.5
    u32wrapping_add
    swap

    movup.2
    dup.6
    u32wrapping_add
    movdn.2

    movup.3
    dup.7
    u32wrapping_add
    movdn.3

    movupw.3

    dup.4
    u32checked_xor
    u32unchecked_rotr.12
    
    swap
    dup.5
    u32checked_xor
    u32unchecked_rotr.12
    swap

    movup.2
    dup.6
    u32checked_xor
    u32unchecked_rotr.12
    movdn.2

    movup.3
    dup.7
    u32checked_xor
    u32unchecked_rotr.12
    movdn.3

    movupw.3
    pushw.local.0
    swapw

    movup.4
    dup.8
    u32overflowing_add3
    drop

    swap
    movup.4
    dup.8
    u32overflowing_add3
    drop
    swap

    movup.2
    movup.4
    dup.8
    u32overflowing_add3
    drop
    movdn.2

    movup.3
    movup.4
    dup.8
    u32overflowing_add3
    drop
    movdn.3

    movupw.3

    dup.4
    u32checked_xor
    u32unchecked_rotr.8
    
    swap
    dup.5
    u32checked_xor
    u32unchecked_rotr.8
    swap

    movup.2
    dup.6
    u32checked_xor
    u32unchecked_rotr.8
    movdn.2

    movup.3
    dup.7
    u32checked_xor
    u32unchecked_rotr.8
    movdn.3

    movupw.3

    dup.4
    u32wrapping_add

    swap
    dup.5
    u32wrapping_add
    swap

    movup.2
    dup.6
    u32wrapping_add
    movdn.2

    movup.3
    dup.7
    u32wrapping_add
    movdn.3

    movupw.3

    dup.4
    u32checked_xor
    u32unchecked_rotr.7

    swap
    dup.5
    u32checked_xor
    u32unchecked_rotr.7
    swap

    movup.2
    dup.6
    u32checked_xor
    u32unchecked_rotr.7
    movdn.2

    movup.3
    dup.7
    u32checked_xor
    u32unchecked_rotr.7
    movdn.3

    movupw.3
end

# Given blake3 state matrix ( total 16 elements, each of 32 -bit ) and 
# 8 message words ( each of 32 -bit ), this routine performs diagonal-wise mixing
# of message words into blake3 hash state.
#
# Functionality wise this routine is equivalent to https://github.com/BLAKE3-team/BLAKE3/blob/da4c792/reference_impl/reference_impl.rs#L61-L64
#
# Expected stack state:
#
# [state0_3_addr, state4_7_addr, state8_11_addr, state12_15_addr, m0, m1, m2, m3, m4, m5, m6, m7]
#
# Note, state_`i`_`j`_addr -> absolute address of {state[i], state[i+1], state[i+2], state[i+3]} in memory | j = i+3
#
# Meaning four consecutive blake3 state words can be read from memory easily.
#
# Final stack state:
#
# [state0, state1, state2, state3, state4, state5, state6, state7, state8, state9, state10, state11, state12, state13, state14, state15]
#
# i.e. whole blake3 state is placed on stack ( in order ).
proc.diagonal_mixing.1
    swapw.2
    swapw

    movup.7
    movup.6
    movup.5
    movup.4

    storew.local.0

    movup.9
    loadw.mem
    movup.8
    pushw.mem

    movup.8
    dup.6
    u32overflowing_add3
    drop

    swap
    movup.8
    dup.7
    u32overflowing_add3
    drop
    swap

    movup.2
    movup.8
    dup.8
    u32overflowing_add3
    drop
    movdn.2

    movup.3
    movup.8
    dup.5
    u32overflowing_add3
    drop
    movdn.3

    movup.9
    pushw.mem

    movup.3
    dup.4
    u32checked_xor
    u32unchecked_rotr.16
    movdn.3

    dup.5
    u32checked_xor
    u32unchecked_rotr.16

    swap
    dup.6
    u32checked_xor
    u32unchecked_rotr.16
    swap

    movup.2
    dup.7
    u32checked_xor
    u32unchecked_rotr.16
    movdn.2

    movup.12
    pushw.mem

    movup.2
    dup.7
    u32wrapping_add
    movdn.2

    movup.3
    dup.4
    u32wrapping_add
    movdn.3

    dup.5
    u32wrapping_add

    swap
    dup.6
    u32wrapping_add
    swap

    movupw.3

    swap
    dup.6
    u32checked_xor
    u32unchecked_rotr.12
    swap

    movup.2
    dup.7
    u32checked_xor
    u32unchecked_rotr.12
    movdn.2

    movup.3
    dup.4
    u32checked_xor
    u32unchecked_rotr.12
    movdn.3

    dup.5
    u32checked_xor
    u32unchecked_rotr.12

    movupw.3
    pushw.local.0
    swapw

    movup.4
    dup.9
    u32overflowing_add3
    drop

    swap
    movup.4
    dup.9
    u32overflowing_add3
    drop
    swap

    movup.2
    movup.4
    dup.9
    u32overflowing_add3
    drop
    movdn.2

    movup.3
    movup.4
    dup.5
    u32overflowing_add3
    drop
    movdn.3

    movupw.3

    movup.3
    dup.4
    u32checked_xor
    u32unchecked_rotr.8
    movdn.3

    dup.5
    u32checked_xor
    u32unchecked_rotr.8

    swap
    dup.6
    u32checked_xor
    u32unchecked_rotr.8
    swap

    movup.2
    dup.7
    u32checked_xor
    u32unchecked_rotr.8
    movdn.2

    movupw.3

    movup.2
    dup.7
    u32wrapping_add
    movdn.2

    movup.3
    dup.4
    u32wrapping_add
    movdn.3

    dup.5
    u32wrapping_add

    swap
    dup.6
    u32wrapping_add
    swap

    movupw.3

    swap
    dup.6
    u32checked_xor
    u32unchecked_rotr.7
    swap

    movup.2
    dup.7
    u32checked_xor
    u32unchecked_rotr.7
    movdn.2

    movup.3
    dup.4
    u32checked_xor
    u32unchecked_rotr.7
    movdn.3

    dup.5
    u32checked_xor
    u32unchecked_rotr.7

    movupw.3
end

# Given blake3 state matrix ( total 16 elements, each of 32 -bit ) and 
# 16 message words ( each of 32 -bit ), this routine applies single round of mixing
# of message words into hash state i.e. msg_word[0..8] are mixed into hash state using
# columnar mixing while remaining message words ( msg_word[8..16] ) are mixed into hash state
# using diagonal mixing.
#
# Functionality wise this routine is equivalent to https://github.com/BLAKE3-team/BLAKE3/blob/da4c792/reference_impl/reference_impl.rs#L54-L65
#
# Expected stack state:
#
# [state0_3_addr, state4_7_addr, state8_11_addr, state12_15_addr, m0, m1, m2, m3, m4, m5, m6, m7, m8, m9, m10, m11, m12, m13, m14, m15]
#
# Note, state_`i`_`j`_addr -> absolute address of {state[i], state[i+1], state[i+2], state[i+3]} in memory | j = i+3
#
# Meaning four consecutive blake3 state words can be read from memory easily.
#
# Final stack state:
#
# [...]
#
# i.e. mixed state matrix lives in memory addresses {state0_3_addr, state4_7_addr, state8_11_addr, state12_15_addr}, 
# which were provided, on stack top, while invoking this routine.
proc.round.5
    storew.local.0

    exec.columnar_mixing

    popw.local.1
    popw.local.2
    popw.local.3
    popw.local.4

    push.env.locaddr.4
    push.env.locaddr.3
    push.env.locaddr.2
    push.env.locaddr.1

    exec.diagonal_mixing

    pushw.local.0
    swapw
    movup.4
    popw.mem

    repeat.3
        push.0
        movdn.3
        swapw
        movup.4
        popw.mem
    end

    repeat.3
        drop
    end
end

# Given blake3 state matrix ( total 16 elements, each of 32 -bit ) and a message block
# i.e. 16 message words ( each of 32 -bit ), this routine applies 7 rounds of mixing
# of (permuted) message words into hash state.
#
# Functionality wise this routine is equivalent to https://github.com/BLAKE3-team/BLAKE3/blob/da4c792/reference_impl/reference_impl.rs#L75-L114
#
# Expected stack state:
#
# [state0_3_addr, state4_7_addr, state8_11_addr, state12_15_addr, m0, m1, m2, m3, m4, m5, m6, m7, m8, m9, m10, m11, m12, m13, m14, m15]
#
# Note, state_`i`_`j`_addr -> absolute address of {state[i], state[i+1], state[i+2], state[i+3]} in memory | j = i+3
#
# Meaning four consecutive blake3 state words can be read from memory easily.
#
# Final stack state:
#
# [...]
#
# i.e. 7 -round mixed state matrix lives in memory addresses {state0_3_addr, state4_7_addr, state8_11_addr, state12_15_addr}, 
# which were provided, on stack top, while invoking this routine. So updated state matrix can be read by caller routine, by reading
# the content of memory addresses where state was provided as routine input.
proc.compress.1
    popw.local.0

    # apply first 6 rounds of mixing
    repeat.6
        # round `i` | i ∈ [1..7)
        repeat.4
            dupw.3
        end

        pushw.local.0
        exec.round
        exec.permute_msg_words
    end

    # round 7 ( last round, so no message word permutation required )
    pushw.local.0
    exec.round
end

# Blake3 2-to-1 hash function, which takes 64 -bytes input and produces 32 -bytes output digest
#
# Expected stack state:
#
# [msg0, msg1, msg2, msg3, msg4, msg5, msg6, msg7, msg8, msg9, msg10, msg11, msg12, msg13, msg14, msg15]
#
# msg`i` -> 32 -bit message word | i ∈ [0, 16)
#
# Output stack state:
#
# [dig0, dig1, dig2, dig3, dig4, dig5, dig6, dig7]
#
# dig`i` -> 32 -bit digest word | i ∈ [0, 8)
export.hash.4
    push.env.locaddr.3
    push.env.locaddr.2
    push.env.locaddr.1
    push.env.locaddr.0

    exec.initialize

    # Note, chunk compression routine needs to compress only one chunk with one message 
    # block ( = 64 -bytes ) because what we're doing here is 2-to-1 hashing i.e. 64 -bytes 
    # input being converted to 32 -bytes output

    push.env.locaddr.3
    push.env.locaddr.2
    push.env.locaddr.1
    push.env.locaddr.0

    exec.compress

    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.finalize
end
"),
// ----- std::crypto::hashes::keccak256 -----------------------------------------------------------
("std::crypto::hashes::keccak256", "# if stack top has [d, c, b, a], after completion of execution of
# this procedure stack top should look like [a, b, c, d]
proc.rev_4_elements
    swap
    movup.2
    movup.3
end

# given four elements of from each of a, b sets, following procedure computes a[i] ^ b[i] ∀ i = [0, 3]
proc.xor_4_elements
    movup.7
    u32checked_xor

    swap

    movup.6
    u32checked_xor

    movup.2
    movup.5
    u32checked_xor

    movup.4
    movup.4
    u32checked_xor
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's θ function, which is
# implemented in terms of 32 -bit word size;
# see https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L55-L98 for original implementation
proc.theta.7
    popw.local.0
    popw.local.1
    popw.local.2
    popw.local.3

    # --- begin https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L71-L79 ---

    # compute a[0] ^ a[10] ^ a[20] ^ a[30] ^ a[40]
    loadw.local.0
    swap
    drop
    movup.2
    drop

    pushw.mem
    repeat.3
        swap
        drop
    end

    swap
    pushw.mem
    drop
    drop
    swap
    drop

    u32checked_xor

    pushw.local.1
    drop
    swap
    drop

    pushw.mem
    repeat.3
        swap
        drop
    end

    swap
    pushw.mem
    drop
    drop
    swap
    drop

    u32checked_xor
    u32checked_xor

    pushw.local.2
    drop
    drop
    swap
    drop

    pushw.mem
    repeat.3
        swap
        drop
    end

    u32checked_xor

    # stack = [c_0]
    # -----
    # compute a[1] ^ a[11] ^ a[21] ^ a[31] ^ a[41]

    pushw.local.0
    swap
    drop
    movup.2
    drop

    pushw.mem
    drop
    repeat.2
        swap
        drop
    end

    swap
    pushw.mem
    drop
    drop
    drop

    u32checked_xor

    pushw.local.1
    drop
    swap
    drop

    pushw.mem

    drop
    repeat.2
        swap
        drop
    end

    swap

    pushw.mem
    drop
    drop
    drop

    u32checked_xor
    u32checked_xor

    pushw.local.2
    drop
    drop
    swap
    drop

    pushw.mem

    drop
    repeat.2
        swap
        drop
    end

    u32checked_xor

    # stack = [c_1, c_0]
    # -----
    # compute a[2] ^ a[12] ^ a[22] ^ a[32] ^ a[42]

    pushw.local.0
    repeat.2
        swap
        drop
    end

    pushw.mem

    drop
    drop
    swap
    drop

    swap

    pushw.mem

    repeat.3
        swap
        drop
    end

    u32checked_xor

    pushw.local.1

    drop
    repeat.2
        swap
        drop
    end

    pushw.mem

    drop
    drop
    swap
    drop

    u32checked_xor

    pushw.local.2

    swap
    drop
    movup.2
    drop

    pushw.mem

    repeat.3
        swap
        drop
    end

    swap

    pushw.mem

    drop
    drop
    swap
    drop

    u32checked_xor
    u32checked_xor

    # stack = [c_2, c_1, c_0]
    # -----
    # compute a[3] ^ a[13] ^ a[23] ^ a[33] ^ a[43]

    pushw.local.0

    repeat.2
        swap
        drop
    end

    pushw.mem

    drop
    drop
    drop

    swap

    pushw.mem

    drop
    repeat.2
        swap
        drop
    end

    u32checked_xor

    pushw.local.1

    drop
    repeat.2
        swap
        drop
    end

    pushw.mem

    drop
    drop
    drop

    u32checked_xor

    pushw.local.2

    swap
    drop
    movup.2
    drop

    pushw.mem

    drop
    repeat.2
        swap
        drop
    end

    swap

    pushw.mem

    drop
    drop
    drop

    u32checked_xor
    u32checked_xor

    # stack = [c_3, c_2, c_1, c_0]
    # -----
    # compute a[4] ^ a[14] ^ a[24] ^ a[34] ^ a[44]

    pushw.local.0

    drop
    swap
    drop

    pushw.mem

    repeat.3
        swap
        drop
    end

    swap

    pushw.mem

    drop
    drop
    swap
    drop

    u32checked_xor

    pushw.local.1

    drop
    drop
    swap
    drop

    pushw.mem

    repeat.3
        swap
        drop
    end

    u32checked_xor

    pushw.local.2

    repeat.2
        swap
        drop
    end

    pushw.mem

    drop
    drop
    swap
    drop

    swap

    pushw.mem

    repeat.3
        swap
        drop
    end

    u32checked_xor
    u32checked_xor

    # stack = [c_4, c_3, c_2, c_1, c_0]
    # -----
    # compute a[5] ^ a[15] ^ a[25] ^ a[35] ^ a[45]

    pushw.local.0

    drop
    swap
    drop

    pushw.mem

    drop
    repeat.2
        swap
        drop
    end

    swap

    pushw.mem

    drop
    drop
    drop

    u32checked_xor

    pushw.local.1

    drop
    drop
    swap
    drop

    pushw.mem

    drop
    repeat.2
        swap
        drop
    end

    u32checked_xor

    pushw.local.2

    repeat.2
        swap
        drop
    end

    pushw.mem

    drop
    drop
    drop

    swap

    pushw.mem

    drop
    repeat.2
        swap
        drop
    end

    u32checked_xor
    u32checked_xor

    # stack = [c_5, c_4, c_3, c_2, c_1, c_0]
    # -----
    # compute a[6] ^ a[16] ^ a[26] ^ a[36] ^ a[46]

    pushw.local.0

    drop
    repeat.2
        swap
        drop
    end

    pushw.mem

    drop
    drop
    swap
    drop

    pushw.local.1

    swap
    drop
    movup.2
    drop

    pushw.mem

    repeat.3
        swap
        drop
    end

    swap

    pushw.mem

    drop
    drop
    swap
    drop

    u32checked_xor
    u32checked_xor

    pushw.local.2

    drop
    swap
    drop

    pushw.mem

    repeat.3
        swap
        drop
    end

    swap

    pushw.mem

    drop
    drop
    swap
    drop

    u32checked_xor
    u32checked_xor

    # stack = [c_6, c_5, c_4, c_3, c_2, c_1, c_0]
    # -----
    # compute a[7] ^ a[17] ^ a[27] ^ a[37] ^ a[47]

    pushw.local.0

    drop
    repeat.2
        swap
        drop
    end

    pushw.mem

    drop
    drop
    drop

    pushw.local.1

    swap
    drop
    movup.2
    drop

    pushw.mem

    drop
    repeat.2
        swap
        drop
    end

    swap

    pushw.mem

    drop
    drop
    drop

    u32checked_xor
    u32checked_xor

    pushw.local.2

    drop
    swap
    drop

    pushw.mem

    drop
    repeat.2
        swap
        drop
    end

    swap

    pushw.mem

    drop
    drop
    drop

    u32checked_xor
    u32checked_xor

    # stack = [c_7, c_6, c_5, c_4, c_3, c_2, c_1, c_0]
    # -----
    # compute a[8] ^ a[18] ^ a[28] ^ a[38] ^ a[48]

    pushw.local.0

    drop
    drop
    swap
    drop

    pushw.mem

    repeat.3
        swap
        drop
    end

    pushw.local.1

    repeat.2
        swap
        drop
    end

    pushw.mem

    drop
    drop
    swap
    drop

    swap

    pushw.mem

    repeat.3
        swap
        drop
    end

    u32checked_xor
    u32checked_xor

    pushw.local.2

    drop
    repeat.2
        swap
        drop
    end

    pushw.mem

    drop
    drop
    swap
    drop

    u32checked_xor

    pushw.local.3

    repeat.3
        swap
        drop
    end

    pushw.mem

    repeat.3
        swap
        drop
    end

    u32checked_xor

    # stack = [c_8, c_7, c_6, c_5, c_4, c_3, c_2, c_1, c_0]
    # -----
    # compute a[9] ^ a[19] ^ a[29] ^ a[39] ^ a[49]

    pushw.local.0

    drop
    drop
    swap
    drop

    pushw.mem

    drop
    repeat.2
        swap
        drop
    end

    pushw.local.1

    repeat.2
        swap
        drop
    end

    pushw.mem

    drop
    drop
    drop

    swap

    pushw.mem

    drop
    repeat.2
        swap
        drop
    end

    u32checked_xor
    u32checked_xor

    pushw.local.2

    drop
    repeat.2
        swap
        drop
    end

    pushw.mem

    drop
    drop
    drop

    pushw.local.3

    repeat.3
        swap
        drop
    end

    pushw.mem

    drop
    repeat.2
        swap
        drop
    end

    u32checked_xor
    u32checked_xor

    push.0.0

    # stack = [0, 0, c_9, c_8, c_7, c_6, c_5, c_4, c_3, c_2, c_1, c_0]

    exec.rev_4_elements
    popw.local.6 # -> to mem [c8, c9, 0, 0]

    exec.rev_4_elements
    popw.local.5 # -> to mem [c4, c5, c6, c7]

    exec.rev_4_elements
    popw.local.4 # -> to mem [c0, c1, c2, c3]

    # --- end https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L71-L79 ---

    # --- begin https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L81-L91 ---

    pushw.local.6
    movup.3
    drop
    movup.2
    drop

    pushw.local.4
    drop
    drop

    movup.3
    u32checked_xor

    swap
    movup.2
    swap

    u32checked_rotl.1
    u32checked_xor

    # stack = [d0, d1]

    pushw.local.4
    movup.3
    drop
    movup.2
    drop

    pushw.local.5
    movup.3
    drop
    movup.2
    drop

    movup.3
    u32checked_xor

    swap
    u32checked_rotl.1
    movup.2
    u32checked_xor

    # stack = [d2, d3, d0, d1]

    movup.3
    movup.3

    # stack = [d0, d1, d2, d3]

    pushw.local.4
    drop
    drop

    pushw.local.5
    drop
    drop

    movup.3
    u32checked_xor

    swap
    u32checked_rotl.1
    movup.2
    u32checked_xor

    # stack = [d4, d5, d0, d1, d2, d3]

    pushw.local.5
    movup.3
    drop
    movup.2
    drop

    pushw.local.6
    movup.3
    drop
    movup.2
    drop

    movup.3
    u32checked_xor

    swap
    u32checked_rotl.1
    movup.2
    u32checked_xor

    # stack = [d6, d7, d4, d5, d0, d1, d2, d3]

    movup.3
    movup.3

    # stack = [d4, d5, d6, d7, d0, d1, d2, d3]

    pushw.local.5
    drop
    drop

    pushw.local.4
    movup.3
    drop
    movup.2
    drop

    movup.3
    u32checked_xor

    swap
    u32checked_rotl.1
    movup.2
    u32checked_xor

    # stack = [d8, d9, d4, d5, d6, d7, d0, d1, d2, d3]

    push.0.0
    movup.3
    movup.3

    # stack = [d8, d9, 0, 0, d4, d5, d6, d7, d0, d1, d2, d3]

    popw.local.6 # -> to mem [d8, d9, 0, 0]
    popw.local.5 # -> to mem [d4, d5, d6, d7]
    popw.local.4 # -> to mem [d0, d1, d2, d3]

    # --- end https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L81-L91 ---

    pushw.local.0
    dupw

    pushw.mem

    pushw.local.4
    exec.rev_4_elements

    exec.xor_4_elements # compute state[0..4]

    movup.7
    popw.mem

    pushw.mem

    pushw.local.5
    exec.rev_4_elements

    exec.xor_4_elements # compute state[4..8]

    movup.6
    popw.mem

    pushw.mem

    pushw.local.6
    exec.rev_4_elements

    drop
    drop

    pushw.local.4
    exec.rev_4_elements

    drop
    drop

    exec.xor_4_elements # compute state[8..12]

    movup.5
    popw.mem

    pushw.mem

    pushw.local.4
    drop
    drop
    swap

    pushw.local.5
    exec.rev_4_elements

    drop
    drop

    exec.xor_4_elements # compute state[12..16]

    movup.4
    popw.mem

    pushw.local.1
    dupw

    pushw.mem

    pushw.local.5
    drop
    drop
    swap

    pushw.local.6
    exec.rev_4_elements

    drop
    drop

    exec.xor_4_elements # compute state[16..20]

    movup.7
    popw.mem

    pushw.mem

    pushw.local.4
    exec.rev_4_elements

    exec.xor_4_elements # compute state[20..24]

    movup.6
    popw.mem

    pushw.mem

    pushw.local.5
    exec.rev_4_elements

    exec.xor_4_elements # compute state[24..28]

    movup.5
    popw.mem

    pushw.mem

    pushw.local.6
    exec.rev_4_elements

    drop
    drop

    pushw.local.4
    exec.rev_4_elements

    drop
    drop

    exec.xor_4_elements # compute state[28..32]

    movup.4
    popw.mem

    pushw.local.2
    dupw

    pushw.mem

    pushw.local.4
    drop
    drop
    swap

    pushw.local.5
    exec.rev_4_elements

    drop
    drop

    exec.xor_4_elements # compute state[32..36]

    movup.7
    popw.mem

    pushw.mem

    pushw.local.5
    drop
    drop
    swap

    pushw.local.6
    exec.rev_4_elements

    drop
    drop

    exec.xor_4_elements # compute state[36..40]

    movup.6
    popw.mem

    pushw.mem

    pushw.local.4
    exec.rev_4_elements

    exec.xor_4_elements # compute state[40..44]

    movup.5
    popw.mem

    pushw.mem

    pushw.local.5
    exec.rev_4_elements

    exec.xor_4_elements # compute state[44..48]

    movup.4
    popw.mem

    pushw.local.3

    repeat.3
        swap
        drop
    end

    dup
    pushw.mem

    pushw.local.6
    exec.rev_4_elements

    exec.xor_4_elements # compute state[48..50]

    movup.4
    popw.mem
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ρ ( rho ) function, which is
# implemented in terms of 32 -bit word size; see https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L115-L147
proc.rho.4
    popw.local.0
    popw.local.1
    popw.local.2
    popw.local.3

    pushw.local.0
    dupw

    pushw.mem
    exec.rev_4_elements

    u32checked_rotl.1
    swap

    exec.rev_4_elements

    movup.7
    popw.mem # wrote state[0..4]

    pushw.mem

    u32checked_rotl.31
    swap
    u32checked_rotl.31
    swap

    exec.rev_4_elements

    u32checked_rotl.14
    swap
    u32checked_rotl.14
    swap

    exec.rev_4_elements

    movup.6
    popw.mem # wrote state[4..8]

    pushw.mem

    u32checked_rotl.13
    swap
    u32checked_rotl.14

    exec.rev_4_elements

    u32checked_rotl.18
    swap
    u32checked_rotl.18
    swap

    exec.rev_4_elements

    movup.5
    popw.mem # wrote state[8..12]

    pushw.mem

    u32checked_rotl.22
    swap
    u32checked_rotl.22
    swap

    exec.rev_4_elements

    u32checked_rotl.3
    swap
    u32checked_rotl.3
    swap

    exec.rev_4_elements

    movup.4
    popw.mem # wrote state[12..16]

    pushw.local.1
    dupw

    pushw.mem

    u32checked_rotl.27
    swap
    u32checked_rotl.28

    exec.rev_4_elements

    u32checked_rotl.10
    swap
    u32checked_rotl.10
    swap

    exec.rev_4_elements

    movup.7
    popw.mem # wrote state[16..20]

    pushw.mem

    u32checked_rotl.1
    swap
    u32checked_rotl.2

    exec.rev_4_elements

    u32checked_rotl.5
    swap
    u32checked_rotl.5
    swap

    exec.rev_4_elements

    movup.6
    popw.mem # wrote state[20..24]

    pushw.mem

    u32checked_rotl.21
    swap
    u32checked_rotl.22

    exec.rev_4_elements

    u32checked_rotl.13
    swap
    u32checked_rotl.12

    exec.rev_4_elements

    movup.5
    popw.mem # wrote state[24..28]

    pushw.mem

    u32checked_rotl.19
    swap
    u32checked_rotl.20

    exec.rev_4_elements

    u32checked_rotl.21
    swap
    u32checked_rotl.20

    exec.rev_4_elements

    movup.4
    popw.mem # wrote state[28..32]

    pushw.local.2
    dupw

    pushw.mem

    u32checked_rotl.22
    swap
    u32checked_rotl.23

    exec.rev_4_elements

    u32checked_rotl.8
    swap
    u32checked_rotl.7

    exec.rev_4_elements

    movup.7
    popw.mem # wrote state[32..36]

    pushw.mem

    u32checked_rotl.10
    swap
    u32checked_rotl.11

    exec.rev_4_elements

    u32checked_rotl.4
    swap
    u32checked_rotl.4
    swap

    exec.rev_4_elements

    movup.6
    popw.mem # wrote state[36..40]

    pushw.mem

    u32checked_rotl.9
    swap
    u32checked_rotl.9
    swap

    exec.rev_4_elements

    u32checked_rotl.1
    swap
    u32checked_rotl.1
    swap

    exec.rev_4_elements

    movup.5
    popw.mem # wrote state[40..44]

    pushw.mem

    u32checked_rotl.30
    swap
    u32checked_rotl.31

    exec.rev_4_elements

    u32checked_rotl.28
    swap
    u32checked_rotl.28
    swap

    exec.rev_4_elements

    movup.4
    popw.mem # wrote state[44..48]

    pushw.local.3

    repeat.3
        swap
        drop
    end

    dup

    pushw.mem

    u32checked_rotl.7
    swap
    u32checked_rotl.7
    swap

    movup.4
    popw.mem # wrote state[48..50]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's π function, which is
# implemented in terms of 32 -bit word size; see https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L169-L207
proc.pi.17
    popw.local.0
    popw.local.1
    popw.local.2
    popw.local.3

    pushw.local.0
    repeat.2
        swap
        drop
    end

    swap
    pushw.mem

    exec.rev_4_elements

    drop
    drop
    swap

    movup.2
    pushw.mem

    exec.rev_4_elements

    drop
    drop
    swap

    popw.local.4 # wrote state[0..4]

    pushw.local.2

    drop
    repeat.2
        swap
        drop
    end

    pushw.mem

    exec.rev_4_elements

    drop
    drop
    swap

    pushw.local.1

    drop
    drop
    swap
    drop

    pushw.mem

    exec.rev_4_elements

    drop
    drop
    swap

    popw.local.5 # wrote state[4..8]

    pushw.local.0

    drop
    repeat.2
        swap
        drop
    end

    pushw.mem

    drop
    drop

    pushw.local.3

    repeat.3
        swap
        drop
    end

    pushw.mem

    exec.rev_4_elements

    drop
    drop
    swap

    popw.local.6 # wrote state[8..12]

    pushw.local.1

    exec.rev_4_elements

    drop
    drop

    pushw.mem

    exec.rev_4_elements

    drop
    drop
    swap

    movup.2

    pushw.mem

    drop
    drop

    popw.local.7 # wrote state[12..16]

    pushw.local.2

    repeat.2
        swap
        drop
    end

    swap

    pushw.mem

    exec.rev_4_elements

    drop
    drop
    swap

    movup.2

    pushw.mem

    exec.rev_4_elements

    drop
    drop
    swap

    popw.local.8 # wrote state[16..20]

    pushw.local.0

    repeat.2
        swap
        drop
    end

    swap

    pushw.mem

    drop
    drop

    movup.2

    pushw.mem

    drop
    drop

    popw.local.9 # wrote state[20..24]

    pushw.local.2

    drop
    repeat.2
        swap
        drop
    end

    pushw.mem

    drop
    drop

    pushw.local.1

    drop
    drop
    swap
    drop

    pushw.mem

    drop
    drop

    popw.local.10 # wrote state[24..28]

    pushw.local.0

    drop
    drop
    swap
    drop

    pushw.mem

    exec.rev_4_elements

    drop
    drop
    swap

    pushw.local.2

    drop
    drop
    swap
    drop

    pushw.mem

    exec.rev_4_elements

    drop
    drop
    swap

    popw.local.11 # wrote state[28..32]

    pushw.local.1

    drop
    repeat.2
        swap
        drop
    end

    pushw.mem

    drop
    drop

    pushw.local.0

    drop
    drop
    swap
    drop

    pushw.mem

    drop
    drop

    popw.local.12 # wrote state[32..36]

    pushw.local.2

    repeat.2
        swap
        drop
    end

    swap

    pushw.mem

    drop
    drop

    movup.2

    pushw.mem

    drop
    drop

    popw.local.13 # wrote state[36..40]

    pushw.local.1

    repeat.3
        swap
        drop
    end

    pushw.mem

    exec.rev_4_elements

    drop
    drop
    swap

    pushw.local.0

    drop
    repeat.2
        swap
        drop
    end

    pushw.mem

    exec.rev_4_elements

    drop
    drop
    swap

    popw.local.14 # wrote state[40..44]

    pushw.local.1

    drop
    drop
    drop

    pushw.mem

    popw.local.15 # wrote state[44..48]

    pushw.local.2

    drop
    drop
    swap
    drop

    pushw.mem

    drop
    drop
    push.0.0

    exec.rev_4_elements

    swap

    popw.local.16 # wrote state[48..50]

    pushw.local.0

    pushw.local.4
    movup.4
    storew.mem # final write state[0..4]

    loadw.local.5
    movup.4
    storew.mem # final write state[4..8]

    loadw.local.6
    movup.4
    storew.mem # final write state[8..12]

    loadw.local.7
    movup.4
    storew.mem # final write state[12..16]

    loadw.local.1

    pushw.local.8
    movup.4
    storew.mem # final write state[16..20]

    loadw.local.9
    movup.4
    storew.mem # final write state[20..24]

    loadw.local.10
    movup.4
    storew.mem # final write state[24..28]

    loadw.local.11
    movup.4
    storew.mem # final write state[28..32]

    loadw.local.2

    pushw.local.12
    movup.4
    storew.mem # final write state[32..36]

    loadw.local.13
    movup.4
    storew.mem # final write state[36..40]

    loadw.local.14
    movup.4
    storew.mem # final write state[40..44]

    loadw.local.15
    movup.4
    storew.mem # final write state[44..48]

    loadw.local.16

    pushw.local.3
    repeat.3
        swap
        drop
    end

    storew.mem # final write state[48..50]
    dropw
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's χ function, which is
# implemented in terms of 32 -bit word size; see https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L233-L271
proc.chi.7
    popw.local.0
    popw.local.1
    popw.local.2
    popw.local.3

    pushw.local.0

    exec.rev_4_elements

    drop
    drop

    pushw.mem

    exec.rev_4_elements

    drop
    drop
    swap

    movup.2

    pushw.mem

    drop
    drop

    u32checked_not
    swap
    u32checked_not
    swap

    movup.2
    u32checked_and

    swap
    movup.2
    u32checked_and
    swap

    pushw.local.0

    drop
    repeat.2
        swap
        drop
    end

    pushw.mem

    u32checked_not
    swap
    u32checked_not
    swap

    movup.2
    u32checked_and

    swap
    movup.2
    u32checked_and

    exec.rev_4_elements
    swap

    popw.local.4 # write to c[0..4]

    pushw.local.0

    drop
    movup.2
    drop

    swap

    pushw.mem

    exec.rev_4_elements

    drop
    drop
    swap

    movup.2

    pushw.mem

    drop
    drop

    u32checked_not
    swap
    u32checked_not
    swap

    movup.2
    u32checked_and

    swap
    movup.2
    u32checked_and

    pushw.local.0

    swap
    drop
    movup.2
    drop
    swap

    pushw.mem

    exec.rev_4_elements

    drop
    drop

    u32checked_not
    swap
    u32checked_not

    movup.2
    pushw.mem

    exec.rev_4_elements

    drop
    drop

    movup.3
    u32checked_and

    swap
    movup.2
    u32checked_and

    swap
    exec.rev_4_elements

    popw.local.5 # write to c[4..8]

    pushw.local.0

    repeat.3
        swap
        drop
    end

    pushw.mem

    u32checked_not
    swap
    u32checked_not
    swap

    movup.2
    u32checked_and

    swap
    movup.2
    u32checked_and

    push.0.0
    exec.rev_4_elements

    popw.local.6 # write to c[8..10]

    pushw.local.0

    movup.3
    drop

    dup
    pushw.mem
    pushw.local.4

    exec.rev_4_elements
    exec.xor_4_elements

    movup.4

    popw.mem # write to state[0..4]

    dup
    pushw.mem
    pushw.local.5

    exec.rev_4_elements
    exec.xor_4_elements

    movup.4

    popw.mem # write to state[4..8]

    dup
    pushw.mem
    pushw.local.6

    exec.rev_4_elements
    exec.xor_4_elements

    movup.4

    popw.mem # write to state[8..10]

    pushw.local.0

    drop
    drop
    drop

    pushw.mem

    u32checked_not
    swap
    u32checked_not
    swap

    movup.2
    u32checked_and

    swap
    movup.2
    u32checked_and

    swap
    push.0.0

    popw.local.4 # write to c[0..2]

    pushw.local.1

    repeat.3
        swap
        drop
    end

    pushw.mem

    exec.rev_4_elements

    drop
    drop

    pushw.local.0

    drop
    drop
    drop

    pushw.mem

    drop
    drop

    u32checked_not
    swap
    u32checked_not
    swap

    movup.3
    u32checked_and

    swap
    movup.2
    u32checked_and

    pushw.local.1

    repeat.3
        swap
        drop
    end

    pushw.mem

    u32checked_not
    swap
    u32checked_not
    swap

    movup.2
    u32checked_and

    swap
    movup.2
    u32checked_and

    exec.rev_4_elements
    popw.local.5 # write to c[2..6]

    pushw.local.1

    repeat.3
        swap
        drop
    end

    pushw.mem

    drop
    drop

    u32checked_not
    swap
    u32checked_not
    swap

    pushw.local.0

    drop
    drop
    swap
    drop

    pushw.mem

    drop
    drop

    movup.2
    u32checked_and

    swap
    movup.2
    u32checked_and

    pushw.local.0

    drop
    drop

    pushw.mem

    drop
    drop

    u32checked_not
    swap
    u32checked_not
    swap

    movup.2

    pushw.mem

    exec.rev_4_elements

    drop
    drop

    movup.3
    u32checked_and

    swap
    movup.2
    u32checked_and
    swap

    exec.rev_4_elements
    popw.local.6 # write to c[6..10]

    pushw.local.0

    drop
    drop

    dup
    pushw.mem

    pushw.local.4

    exec.rev_4_elements
    exec.xor_4_elements

    movup.4
    popw.mem # write to state[10..12]

    dup
    pushw.mem

    pushw.local.5

    exec.rev_4_elements
    exec.xor_4_elements

    movup.4
    popw.mem # write to state[12..16]

    pushw.local.1

    repeat.3
        swap
        drop
    end

    dup
    pushw.mem

    pushw.local.6

    exec.rev_4_elements
    exec.xor_4_elements

    movup.4
    popw.mem # write to state[16..20]

    pushw.local.1

    drop
    movup.2
    drop

    pushw.mem

    drop
    drop

    u32checked_not
    swap
    u32checked_not
    swap

    movup.2

    pushw.mem

    exec.rev_4_elements

    drop
    drop

    movup.3
    u32checked_and

    swap
    movup.2
    u32checked_and
    swap

    pushw.local.1

    drop
    drop
    swap
    drop

    pushw.mem

    u32checked_not
    swap
    u32checked_not
    swap

    movup.2
    u32checked_and

    swap
    movup.2
    u32checked_and

    exec.rev_4_elements
    popw.local.4 # write to c[0..4]

    pushw.local.1

    drop
    drop

    pushw.mem

    drop
    drop

    u32checked_not
    swap
    u32checked_not
    swap

    movup.2

    pushw.mem

    exec.rev_4_elements

    drop
    drop

    movup.3
    u32checked_and

    swap
    movup.2
    u32checked_and
    swap

    pushw.local.1

    drop
    drop
    drop

    pushw.mem

    exec.rev_4_elements

    drop
    drop

    u32checked_not
    swap
    u32checked_not

    pushw.local.1

    drop
    repeat.2
        swap
        drop
    end

    pushw.mem

    exec.rev_4_elements

    drop
    drop

    movup.3
    u32checked_and

    swap
    movup.2
    u32checked_and
    swap

    exec.rev_4_elements
    popw.local.5 # write to c[4..8]

    pushw.local.1

    drop
    repeat.2
        swap
        drop
    end

    pushw.mem

    u32checked_not
    swap
    u32checked_not
    swap

    movup.2
    u32checked_and

    swap
    movup.2
    u32checked_and

    push.0.0
    exec.rev_4_elements

    popw.local.6 # write to c[8..10]

    pushw.local.1

    drop

    dup
    pushw.mem

    pushw.local.4

    exec.rev_4_elements
    exec.xor_4_elements

    movup.4
    popw.mem # write to state[20..24]

    dup
    pushw.mem

    pushw.local.5

    exec.rev_4_elements
    exec.xor_4_elements

    movup.4
    popw.mem # write to state[24..28]

    dup
    pushw.mem

    pushw.local.6

    exec.rev_4_elements
    exec.xor_4_elements

    movup.4
    popw.mem # write to state[28..30]

    pushw.local.2

    repeat.3
        swap
        drop
    end

    pushw.mem

    u32checked_not
    swap
    u32checked_not
    swap

    movup.2
    u32checked_and

    swap
    movup.2
    u32checked_and
    swap

    push.0.0
    popw.local.4 # write to c[0..2]

    pushw.local.2
    movup.2
    drop
    movup.2
    drop

    pushw.mem

    drop
    drop

    u32checked_not
    swap
    u32checked_not
    swap

    dup.2

    pushw.mem

    exec.rev_4_elements

    drop
    drop

    movup.3
    u32checked_and

    swap
    movup.2
    u32checked_and
    swap

    movup.2
    pushw.mem

    u32checked_not
    swap
    u32checked_not
    swap

    movup.2
    u32checked_and

    swap
    movup.2
    u32checked_and

    exec.rev_4_elements
    popw.local.5 # write to c[2..6]

    pushw.local.2

    drop
    repeat.2
        swap
        drop
    end

    pushw.mem

    drop
    drop

    u32checked_not
    swap
    u32checked_not
    swap

    pushw.local.1

    drop
    drop
    drop

    pushw.mem

    drop
    drop

    movup.2
    u32checked_and

    swap
    movup.2
    u32checked_and

    pushw.local.1

    drop
    drop
    drop

    pushw.mem

    drop
    drop

    u32checked_not
    swap
    u32checked_not
    swap

    pushw.local.2

    repeat.3
        swap
        drop
    end

    pushw.mem

    exec.rev_4_elements

    drop
    drop

    movup.3
    u32checked_and

    swap
    movup.2
    u32checked_and
    swap

    exec.rev_4_elements
    popw.local.6 # write to c[6..10]

    pushw.local.1

    drop
    drop
    drop

    dup

    pushw.mem

    pushw.local.4

    exec.rev_4_elements
    exec.xor_4_elements

    movup.4
    popw.mem # write to state[30..32]

    pushw.local.2

    exec.rev_4_elements

    drop
    drop
    swap

    dup
    pushw.mem

    pushw.local.5

    exec.rev_4_elements
    exec.xor_4_elements

    movup.4
    popw.mem # write to state[32..36]

    dup
    pushw.mem

    pushw.local.6

    exec.rev_4_elements
    exec.xor_4_elements

    movup.4
    popw.mem # write to state[36..40]

    pushw.local.2

    drop
    drop

    pushw.mem

    drop
    drop

    u32checked_not
    swap
    u32checked_not
    swap

    movup.2

    pushw.mem

    exec.rev_4_elements

    drop
    drop

    movup.3
    u32checked_and

    swap
    movup.2
    u32checked_and
    swap

    pushw.local.2

    drop
    drop
    drop

    pushw.mem

    u32checked_not
    swap
    u32checked_not
    swap

    movup.2
    u32checked_and

    swap
    movup.2
    u32checked_and

    exec.rev_4_elements
    popw.local.4 # write to c[0..4]

    pushw.local.2

    drop
    drop
    drop

    pushw.mem

    drop
    drop

    u32checked_not
    swap
    u32checked_not
    swap

    pushw.local.3

    repeat.3
        swap
        drop
    end

    pushw.mem

    exec.rev_4_elements

    drop
    drop

    movup.3
    u32checked_and

    swap
    movup.2
    u32checked_and
    swap

    pushw.local.3

    repeat.3
        swap
        drop
    end

    pushw.mem

    exec.rev_4_elements

    drop
    drop

    u32checked_not
    swap
    u32checked_not

    pushw.local.2

    drop
    drop
    swap
    drop

    pushw.mem

    exec.rev_4_elements

    drop
    drop

    movup.3
    u32checked_and

    swap
    movup.2
    u32checked_and
    swap

    exec.rev_4_elements
    popw.local.5 # write to c[4..8]

    pushw.local.2

    drop
    drop
    swap
    drop

    pushw.mem

    u32checked_not
    swap
    u32checked_not
    swap

    movup.2
    u32checked_and

    swap
    movup.2
    u32checked_and

    push.0.0

    exec.rev_4_elements
    popw.local.6 # write to c[8..10]

    pushw.local.2

    drop
    drop

    dup
    pushw.mem

    pushw.local.4

    exec.rev_4_elements
    exec.xor_4_elements

    movup.4
    popw.mem # write to state[40..44]

    dup
    pushw.mem

    pushw.local.5

    exec.rev_4_elements
    exec.xor_4_elements

    movup.4
    popw.mem # write to state[44..48]

    pushw.local.3

    repeat.3
        swap
        drop
    end

    dup
    pushw.mem

    pushw.local.6

    exec.rev_4_elements
    exec.xor_4_elements

    movup.4
    popw.mem # write to state[48..50]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (1u, 0u) as template arguments
proc.iota_round_1
    dup
    pushw.mem

    push.1
    u32checked_xor

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (0u, 137u) as template arguments
proc.iota_round_2
    dup
    pushw.mem

    swap

    push.137
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (0u, 2147483787u) as template arguments
proc.iota_round_3
    dup
    pushw.mem

    swap

    push.2147483787
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (0u, 2147516544u) as template arguments
proc.iota_round_4
    dup
    pushw.mem

    swap

    push.2147516544
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (1u, 139u) as template arguments
proc.iota_round_5
    dup
    pushw.mem

    push.1
    u32checked_xor

    swap

    push.139
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (1u, 32768u) as template arguments
proc.iota_round_6
    dup
    pushw.mem

    push.1
    u32checked_xor

    swap

    push.32768
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (1u, 2147516552u) as template arguments
proc.iota_round_7
    dup
    pushw.mem

    push.1
    u32checked_xor

    swap

    push.2147516552
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (1u, 2147483778u) as template arguments
proc.iota_round_8
    dup
    pushw.mem

    push.1
    u32checked_xor

    swap

    push.2147483778
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (0u, 11u) as template arguments
proc.iota_round_9
    dup
    pushw.mem

    swap

    push.11
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (0u, 10u) as template arguments
proc.iota_round_10
    dup
    pushw.mem

    swap

    push.10
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (1u, 32898u) as template arguments
proc.iota_round_11
    dup
    pushw.mem

    push.1
    u32checked_xor

    swap

    push.32898
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (0u, 32771u) as template arguments
proc.iota_round_12
    dup
    pushw.mem

    swap

    push.32771
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (1u, 32907u) as template arguments
proc.iota_round_13
    dup
    pushw.mem

    push.1
    u32checked_xor

    swap

    push.32907
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (1u, 2147483659u) as template arguments
proc.iota_round_14
    dup
    pushw.mem

    push.1
    u32checked_xor

    swap

    push.2147483659
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (1u, 2147483786u) as template arguments
proc.iota_round_15
    dup
    pushw.mem

    push.1
    u32checked_xor

    swap

    push.2147483786
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (1u, 2147483777u) as template arguments
proc.iota_round_16
    dup
    pushw.mem

    push.1
    u32checked_xor

    swap

    push.2147483777
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (0u, 2147483777u) as template arguments
proc.iota_round_17
    dup
    pushw.mem

    swap

    push.2147483777
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (0u, 2147483656u) as template arguments
proc.iota_round_18
    dup
    pushw.mem

    swap

    push.2147483656
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (0u, 131u) as template arguments
proc.iota_round_19
    dup
    pushw.mem

    swap

    push.131
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (0u, 2147516419u) as template arguments
proc.iota_round_20
    dup
    pushw.mem

    swap

    push.2147516419
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (1u, 2147516552u) as template arguments
proc.iota_round_21
    dup
    pushw.mem

    push.1
    u32checked_xor

    swap

    push.2147516552
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (0u, 2147483784u) as template arguments
proc.iota_round_22
    dup
    pushw.mem

    swap

    push.2147483784
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (1u, 32768u) as template arguments
proc.iota_round_23
    dup
    pushw.mem

    push.1
    u32checked_xor

    swap

    push.32768
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] | b = 1600, n_r = 24, permutation's ι ( iota ) function, which is
# implemented in terms of 32 -bit word size; imagine https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L288-L306
# invoked with (0u, 2147516546u) as template arguments
proc.iota_round_24
    dup
    pushw.mem

    swap

    push.2147516546
    u32checked_xor

    swap

    movup.4
    popw.mem # write to state[0..2]
end

# keccak-p[b, n_r] permutation round, without `iota` function
# ( all other functions i.e. `theta`, `rho`, `pi`, `chi` are applied in order ) | b = 1600, n_r = 24
#
# As `iota` function involves xoring constant factors with first lane of state array ( read state[0, 0] ),
# specialised implementations are maintained; see above; required to be invoked seperately after completion of
# this procedure's execution.
#
# See https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L325-L340
proc.round.4
    storew.local.0
    swapw
    storew.local.1
    movupw.2
    storew.local.2
    movupw.3
    storew.local.3

    # reverse placement order of four VM words
    swapw
    movupw.2
    movupw.3

    exec.theta

    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.rho

    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.pi

    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.chi
end

# keccak-p[1600, 24] permutation, which applies 24 rounds on state array of size  5 x 5 x 64, where each
# 64 -bit lane is represented in bit interleaved form ( in terms of two 32 -bit words ).
#
# See https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/sha3.hpp#L379-L427
proc.keccak_p.4
    popw.local.0
    popw.local.1
    popw.local.2
    popw.local.3

    # permutation round 1
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_1

    # permutation round 2
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_2

    # permutation round 3
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_3

    # permutation round 4
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_4

    # permutation round 5
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_5

    # permutation round 6
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_6

    # permutation round 7
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_7

    # permutation round 8
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_8

    # permutation round 9
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_9

    # permutation round 10
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_10

    # permutation round 11
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_11

    # permutation round 12
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_12

    # permutation round 13
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_13

    # permutation round 14
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_14

    # permutation round 15
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_15

    # permutation round 16
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_16

    # permutation round 17
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_17

    # permutation round 18
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_18

    # permutation round 19
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_19

    # permutation round 20
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_20

    # permutation round 21
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_21

    # permutation round 22
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_22

    # permutation round 23
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_23

    # permutation round 24
    pushw.local.3
    pushw.local.2
    pushw.local.1
    pushw.local.0

    exec.round

    pushw.local.0

    repeat.3
        swap
        drop
    end

    exec.iota_round_24
end

# given two 32 -bit unsigned integers ( standard form ), representing upper and lower
# portion of a 64 -bit unsigned integer ( actually a keccak-[1600, 24] lane ),
# this function converts them into bit interleaved representation, where two 32 -bit
# unsigned integers ( even portion & then odd portion ) hold bits in even and odd
# indices of 64 -bit unsigned integer ( remember it's represented in terms of
# two 32 -bit elements )
#
# Read more about bit interleaved representation in section 2.1 of https://keccak.team/files/Keccak-implementation-3.2.pdf
#
# See https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/utils.hpp#L123-L149
export.to_bit_interleaved
    dup.1

    push.1
    u32checked_and

    dup.2
    u32checked_shr.1
    push.1
    u32checked_and

    swap

    dup.3

    u32checked_shr.2
    push.1
    u32checked_and

    u32checked_shl.1
    u32checked_or

    swap

    dup.3

    u32checked_shr.3
    push.1
    u32checked_and

    u32checked_shl.1
    u32checked_or

    swap

    dup.3

    u32checked_shr.4
    push.1
    u32checked_and

    u32checked_shl.2
    u32checked_or

    swap

    dup.3

    u32checked_shr.5
    push.1
    u32checked_and

    u32checked_shl.2
    u32checked_or

    swap

    dup.3

    u32checked_shr.6
    push.1
    u32checked_and

    u32checked_shl.3
    u32checked_or

    swap

    dup.3

    u32checked_shr.7
    push.1
    u32checked_and

    u32checked_shl.3
    u32checked_or

    swap

    dup.3

    u32checked_shr.8
    push.1
    u32checked_and

    u32checked_shl.4
    u32checked_or

    swap

    dup.3

    u32checked_shr.9
    push.1
    u32checked_and

    u32checked_shl.4
    u32checked_or

    swap

    dup.3

    u32checked_shr.10
    push.1
    u32checked_and

    u32checked_shl.5
    u32checked_or

    swap

    dup.3

    u32checked_shr.11
    push.1
    u32checked_and

    u32checked_shl.5
    u32checked_or

    swap

    dup.3

    u32checked_shr.12
    push.1
    u32checked_and

    u32checked_shl.6
    u32checked_or

    swap

    dup.3

    u32checked_shr.13
    push.1
    u32checked_and

    u32checked_shl.6
    u32checked_or

    swap

    dup.3

    u32checked_shr.14
    push.1
    u32checked_and

    u32checked_shl.7
    u32checked_or

    swap

    dup.3

    u32checked_shr.15
    push.1
    u32checked_and

    u32checked_shl.7
    u32checked_or

    swap

    dup.3

    u32checked_shr.16
    push.1
    u32checked_and

    u32checked_shl.8
    u32checked_or

    swap

    dup.3

    u32checked_shr.17
    push.1
    u32checked_and

    u32checked_shl.8
    u32checked_or

    swap

    dup.3

    u32checked_shr.18
    push.1
    u32checked_and

    u32checked_shl.9
    u32checked_or

    swap

    dup.3

    u32checked_shr.19
    push.1
    u32checked_and

    u32checked_shl.9
    u32checked_or

    swap

    dup.3

    u32checked_shr.20
    push.1
    u32checked_and

    u32checked_shl.10
    u32checked_or

    swap

    dup.3

    u32checked_shr.21
    push.1
    u32checked_and

    u32checked_shl.10
    u32checked_or

    swap

    dup.3

    u32checked_shr.22
    push.1
    u32checked_and

    u32checked_shl.11
    u32checked_or

    swap

    dup.3

    u32checked_shr.23
    push.1
    u32checked_and

    u32checked_shl.11
    u32checked_or

    swap

    dup.3

    u32checked_shr.24
    push.1
    u32checked_and

    u32checked_shl.12
    u32checked_or

    swap

    dup.3

    u32checked_shr.25
    push.1
    u32checked_and

    u32checked_shl.12
    u32checked_or

    swap

    dup.3

    u32checked_shr.26
    push.1
    u32checked_and

    u32checked_shl.13
    u32checked_or

    swap

    dup.3

    u32checked_shr.27
    push.1
    u32checked_and

    u32checked_shl.13
    u32checked_or

    swap

    dup.3

    u32checked_shr.28
    push.1
    u32checked_and

    u32checked_shl.14
    u32checked_or

    swap

    dup.3

    u32checked_shr.29
    push.1
    u32checked_and

    u32checked_shl.14
    u32checked_or

    swap

    dup.3

    u32checked_shr.30
    push.1
    u32checked_and

    u32checked_shl.15
    u32checked_or

    swap

    dup.3

    u32checked_shr.31
    push.1
    u32checked_and

    u32checked_shl.15
    u32checked_or

    swap

    dup.2

    push.1
    u32checked_and

    u32checked_shl.16
    u32checked_or

    swap

    dup.2

    u32checked_shr.1
    push.1
    u32checked_and

    u32checked_shl.16
    u32checked_or

    swap

    dup.2

    u32checked_shr.2
    push.1
    u32checked_and

    u32checked_shl.17
    u32checked_or

    swap

    dup.2

    u32checked_shr.3
    push.1
    u32checked_and

    u32checked_shl.17
    u32checked_or

    swap

    dup.2

    u32checked_shr.4
    push.1
    u32checked_and

    u32checked_shl.18
    u32checked_or

    swap

    dup.2

    u32checked_shr.5
    push.1
    u32checked_and

    u32checked_shl.18
    u32checked_or

    swap

    dup.2

    u32checked_shr.6
    push.1
    u32checked_and

    u32checked_shl.19
    u32checked_or

    swap

    dup.2

    u32checked_shr.7
    push.1
    u32checked_and

    u32checked_shl.19
    u32checked_or

    swap

    dup.2

    u32checked_shr.8
    push.1
    u32checked_and

    u32checked_shl.20
    u32checked_or

    swap

    dup.2

    u32checked_shr.9
    push.1
    u32checked_and

    u32checked_shl.20
    u32checked_or

    swap

    dup.2

    u32checked_shr.10
    push.1
    u32checked_and

    u32checked_shl.21
    u32checked_or

    swap

    dup.2

    u32checked_shr.11
    push.1
    u32checked_and

    u32checked_shl.21
    u32checked_or

    swap

    dup.2

    u32checked_shr.12
    push.1
    u32checked_and

    u32checked_shl.22
    u32checked_or

    swap

    dup.2

    u32checked_shr.13
    push.1
    u32checked_and

    u32checked_shl.22
    u32checked_or

    swap

    dup.2

    u32checked_shr.14
    push.1
    u32checked_and

    u32checked_shl.23
    u32checked_or

    swap

    dup.2

    u32checked_shr.15
    push.1
    u32checked_and

    u32checked_shl.23
    u32checked_or

    swap

    dup.2

    u32checked_shr.16
    push.1
    u32checked_and

    u32checked_shl.24
    u32checked_or

    swap

    dup.2

    u32checked_shr.17
    push.1
    u32checked_and

    u32checked_shl.24
    u32checked_or

    swap

    dup.2

    u32checked_shr.18
    push.1
    u32checked_and

    u32checked_shl.25
    u32checked_or

    swap

    dup.2

    u32checked_shr.19
    push.1
    u32checked_and

    u32checked_shl.25
    u32checked_or

    swap

    dup.2

    u32checked_shr.20
    push.1
    u32checked_and

    u32checked_shl.26
    u32checked_or

    swap

    dup.2

    u32checked_shr.21
    push.1
    u32checked_and

    u32checked_shl.26
    u32checked_or

    swap

    dup.2

    u32checked_shr.22
    push.1
    u32checked_and

    u32checked_shl.27
    u32checked_or

    swap

    dup.2

    u32checked_shr.23
    push.1
    u32checked_and

    u32checked_shl.27
    u32checked_or

    swap

    dup.2

    u32checked_shr.24
    push.1
    u32checked_and

    u32checked_shl.28
    u32checked_or

    swap

    dup.2

    u32checked_shr.25
    push.1
    u32checked_and

    u32checked_shl.28
    u32checked_or

    swap

    dup.2

    u32checked_shr.26
    push.1
    u32checked_and

    u32checked_shl.29
    u32checked_or

    swap

    dup.2

    u32checked_shr.27
    push.1
    u32checked_and

    u32checked_shl.29
    u32checked_or

    swap

    dup.2

    u32checked_shr.28
    push.1
    u32checked_and

    u32checked_shl.30
    u32checked_or

    swap

    dup.2

    u32checked_shr.29
    push.1
    u32checked_and

    u32checked_shl.30
    u32checked_or

    swap

    dup.2

    u32checked_shr.30
    push.1
    u32checked_and

    u32checked_shl.31
    u32checked_or

    swap

    dup.2

    u32checked_shr.31
    push.1
    u32checked_and

    u32checked_shl.31
    u32checked_or

    swap
end

# given two 32 -bit unsigned integers ( bit interleaved form ), representing even and odd
# positioned bits of a 64 -bit unsigned integer ( actually a keccak-[1600, 24] lane ),
# this function converts them into standard representation, where two 32 -bit
# unsigned integers hold higher ( 32 -bit ) and lower ( 32 -bit ) bits of standard
# representation of 64 -bit unsigned integer ( remember it's represented in terms of
# two 32 -bit elements )
#
# This function reverts the action done by `to_bit_interleaved` function implemented above.
#
# Read more about bit interleaved representation in section 2.1 of https://keccak.team/files/Keccak-implementation-3.2.pdf
#
# See https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/utils.hpp#L151-L175
export.from_bit_interleaved
    dup

    push.1
    u32checked_and

    dup.2

    push.1
    u32checked_and

    u32checked_shl.1
    u32checked_or

    dup.1

    u32checked_shr.1
    push.1
    u32checked_and

    u32checked_shl.2
    u32checked_or

    dup.2

    u32checked_shr.1
    push.1
    u32checked_and

    u32checked_shl.3
    u32checked_or

    dup.1

    u32checked_shr.2
    push.1
    u32checked_and

    u32checked_shl.4
    u32checked_or

    dup.2

    u32checked_shr.2
    push.1
    u32checked_and

    u32checked_shl.5
    u32checked_or

    dup.1

    u32checked_shr.3
    push.1
    u32checked_and

    u32checked_shl.6
    u32checked_or

    dup.2

    u32checked_shr.3
    push.1
    u32checked_and

    u32checked_shl.7
    u32checked_or

    dup.1

    u32checked_shr.4
    push.1
    u32checked_and

    u32checked_shl.8
    u32checked_or

    dup.2

    u32checked_shr.4
    push.1
    u32checked_and

    u32checked_shl.9
    u32checked_or

    dup.1

    u32checked_shr.5
    push.1
    u32checked_and

    u32checked_shl.10
    u32checked_or

    dup.2

    u32checked_shr.5
    push.1
    u32checked_and

    u32checked_shl.11
    u32checked_or

    dup.1

    u32checked_shr.6
    push.1
    u32checked_and

    u32checked_shl.12
    u32checked_or

    dup.2

    u32checked_shr.6
    push.1
    u32checked_and

    u32checked_shl.13
    u32checked_or

    dup.1

    u32checked_shr.7
    push.1
    u32checked_and

    u32checked_shl.14
    u32checked_or

    dup.2

    u32checked_shr.7
    push.1
    u32checked_and

    u32checked_shl.15
    u32checked_or

    dup.1

    u32checked_shr.8
    push.1
    u32checked_and

    u32checked_shl.16
    u32checked_or

    dup.2

    u32checked_shr.8
    push.1
    u32checked_and

    u32checked_shl.17
    u32checked_or

    dup.1

    u32checked_shr.9
    push.1
    u32checked_and

    u32checked_shl.18
    u32checked_or

    dup.2

    u32checked_shr.9
    push.1
    u32checked_and

    u32checked_shl.19
    u32checked_or

    dup.1

    u32checked_shr.10
    push.1
    u32checked_and

    u32checked_shl.20
    u32checked_or

    dup.2

    u32checked_shr.10
    push.1
    u32checked_and

    u32checked_shl.21
    u32checked_or

    dup.1

    u32checked_shr.11
    push.1
    u32checked_and

    u32checked_shl.22
    u32checked_or

    dup.2

    u32checked_shr.11
    push.1
    u32checked_and

    u32checked_shl.23
    u32checked_or

    dup.1

    u32checked_shr.12
    push.1
    u32checked_and

    u32checked_shl.24
    u32checked_or

    dup.2

    u32checked_shr.12
    push.1
    u32checked_and

    u32checked_shl.25
    u32checked_or

    dup.1

    u32checked_shr.13
    push.1
    u32checked_and

    u32checked_shl.26
    u32checked_or

    dup.2

    u32checked_shr.13
    push.1
    u32checked_and

    u32checked_shl.27
    u32checked_or

    dup.1

    u32checked_shr.14
    push.1
    u32checked_and

    u32checked_shl.28
    u32checked_or

    dup.2

    u32checked_shr.14
    push.1
    u32checked_and

    u32checked_shl.29
    u32checked_or

    dup.1

    u32checked_shr.15
    push.1
    u32checked_and

    u32checked_shl.30
    u32checked_or

    dup.2

    u32checked_shr.15
    push.1
    u32checked_and

    u32checked_shl.31
    u32checked_or

    dup.1

    u32checked_shr.16
    push.1
    u32checked_and

    dup.3

    u32checked_shr.16
    push.1
    u32checked_and

    u32checked_shl.1
    u32checked_or

    dup.2

    u32checked_shr.17
    push.1
    u32checked_and

    u32checked_shl.2
    u32checked_or

    dup.3

    u32checked_shr.17
    push.1
    u32checked_and

    u32checked_shl.3
    u32checked_or

    dup.2

    u32checked_shr.18
    push.1
    u32checked_and

    u32checked_shl.4
    u32checked_or

    dup.3

    u32checked_shr.18
    push.1
    u32checked_and

    u32checked_shl.5
    u32checked_or

    dup.2

    u32checked_shr.19
    push.1
    u32checked_and

    u32checked_shl.6
    u32checked_or

    dup.3

    u32checked_shr.19
    push.1
    u32checked_and

    u32checked_shl.7
    u32checked_or

    dup.2

    u32checked_shr.20
    push.1
    u32checked_and

    u32checked_shl.8
    u32checked_or

    dup.3

    u32checked_shr.20
    push.1
    u32checked_and

    u32checked_shl.9
    u32checked_or

    dup.2

    u32checked_shr.21
    push.1
    u32checked_and

    u32checked_shl.10
    u32checked_or

    dup.3

    u32checked_shr.21
    push.1
    u32checked_and

    u32checked_shl.11
    u32checked_or

    dup.2

    u32checked_shr.22
    push.1
    u32checked_and

    u32checked_shl.12
    u32checked_or

    dup.3

    u32checked_shr.22
    push.1
    u32checked_and

    u32checked_shl.13
    u32checked_or

    dup.2

    u32checked_shr.23
    push.1
    u32checked_and

    u32checked_shl.14
    u32checked_or

    dup.3

    u32checked_shr.23
    push.1
    u32checked_and

    u32checked_shl.15
    u32checked_or

    dup.2

    u32checked_shr.24
    push.1
    u32checked_and

    u32checked_shl.16
    u32checked_or

    dup.3

    u32checked_shr.24
    push.1
    u32checked_and

    u32checked_shl.17
    u32checked_or

    dup.2

    u32checked_shr.25
    push.1
    u32checked_and

    u32checked_shl.18
    u32checked_or

    dup.3

    u32checked_shr.25
    push.1
    u32checked_and

    u32checked_shl.19
    u32checked_or

    dup.2

    u32checked_shr.26
    push.1
    u32checked_and

    u32checked_shl.20
    u32checked_or

    dup.3

    u32checked_shr.26
    push.1
    u32checked_and

    u32checked_shl.21
    u32checked_or

    dup.2

    u32checked_shr.27
    push.1
    u32checked_and

    u32checked_shl.22
    u32checked_or

    dup.3

    u32checked_shr.27
    push.1
    u32checked_and

    u32checked_shl.23
    u32checked_or

    dup.2

    u32checked_shr.28
    push.1
    u32checked_and

    u32checked_shl.24
    u32checked_or

    dup.3

    u32checked_shr.28
    push.1
    u32checked_and

    u32checked_shl.25
    u32checked_or

    dup.2

    u32checked_shr.29
    push.1
    u32checked_and

    u32checked_shl.26
    u32checked_or

    dup.3

    u32checked_shr.29
    push.1
    u32checked_and

    u32checked_shl.27
    u32checked_or

    dup.2

    u32checked_shr.30
    push.1
    u32checked_and

    u32checked_shl.28
    u32checked_or

    dup.3

    u32checked_shr.30
    push.1
    u32checked_and

    u32checked_shl.29
    u32checked_or

    dup.2

    u32checked_shr.31
    push.1
    u32checked_and

    u32checked_shl.30
    u32checked_or

    dup.3

    u32checked_shr.31
    push.1
    u32checked_and

    u32checked_shl.31
    u32checked_or
end

# given 64 -bytes input ( in terms of sixteen u32 elements on stack top ) to 2-to-1
# keccak256 hash function, this function prepares 5 x 5 x 64 keccak-p[1600, 24] state
# bit array such that each of twenty five 64 -bit wide lane is represented in bit
# interleaved form, using two 32 -bit integers. After completion of execution of
# this function, state array should live in allocated memory ( fifty u32 elements ).
#
# See https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/keccak_256.hpp#L73-L153
proc.to_state_array.4
    popw.local.0
    popw.local.1
    popw.local.2
    popw.local.3

    exec.to_bit_interleaved

    exec.rev_4_elements
    drop
    drop
    swap

    exec.rev_4_elements
    swap

    exec.to_bit_interleaved

    exec.rev_4_elements
    drop
    drop
    swap

    movup.2
    movup.3

    pushw.local.0

    repeat.3
        swap
        drop
    end

    popw.mem # write to state[0..4]

    exec.to_bit_interleaved

    exec.rev_4_elements
    drop
    drop
    swap

    exec.rev_4_elements
    swap

    exec.to_bit_interleaved

    exec.rev_4_elements
    drop
    drop
    swap

    movup.2
    movup.3

    pushw.local.0

    drop
    repeat.2
        swap
        drop
    end

    popw.mem # write to state[4..8]

    exec.to_bit_interleaved

    exec.rev_4_elements
    drop
    drop
    swap

    exec.rev_4_elements
    swap

    exec.to_bit_interleaved

    exec.rev_4_elements
    drop
    drop
    swap

    movup.2
    movup.3

    pushw.local.0

    drop
    drop
    swap
    drop

    popw.mem # write to state[8..12]

    exec.to_bit_interleaved

    exec.rev_4_elements
    drop
    drop
    swap

    exec.rev_4_elements
    swap

    exec.to_bit_interleaved

    exec.rev_4_elements
    drop
    drop
    swap

    movup.2
    movup.3

    pushw.local.0

    drop
    drop
    drop

    popw.mem # write to state[12..16]

    push.0.0.0.1

    pushw.local.1

    repeat.3
        swap
        drop
    end

    popw.mem # write to state[16..20]

    push.0.0.0.0

    pushw.local.1

    drop
    repeat.2
        swap
        drop
    end

    popw.mem # write to state[20..24]

    push.0.0.0.0

    pushw.local.1

    drop
    drop
    swap
    drop

    popw.mem # write to state[24..28]

    push.0.0.0.0

    pushw.local.1

    drop
    drop
    drop

    popw.mem # write to state[28..32]

    push.0.0.2147483648.0

    pushw.local.2

    repeat.3
        swap
        drop
    end

    popw.mem # write to state[32..36]

    push.0.0.0.0

    pushw.local.2

    drop
    repeat.2
        swap
        drop
    end

    popw.mem # write to state[36..40]

    push.0.0.0.0

    pushw.local.2

    drop
    drop
    swap
    drop

    popw.mem # write to state[40..44]

    push.0.0.0.0

    pushw.local.2

    drop
    drop
    drop

    popw.mem # write to state[44..48]

    push.0.0.0.0

    pushw.local.3

    repeat.3
        swap
        drop
    end

    popw.mem # write to state[48..50]
end

# given 32 -bytes digest ( in terms of eight u32 elements on stack top ) in bit interleaved form,
# this function attempts to convert those into standard representation, where eight u32 elements
# live on stack top, each pair of them hold higher and lower bits of 64 -bit unsigned
# integer ( lane of keccak-p[1600, 24] state array )
#
# See https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/keccak_256.hpp#L180-L209
proc.to_digest
    movup.7
    movup.7

    exec.from_bit_interleaved

    exec.rev_4_elements
    drop
    drop
    swap

    movup.7
    movup.7

    exec.from_bit_interleaved

    exec.rev_4_elements
    drop
    drop
    swap

    movup.7
    movup.7

    exec.from_bit_interleaved

    exec.rev_4_elements
    drop
    drop
    swap

    movup.7
    movup.7

    exec.from_bit_interleaved

    exec.rev_4_elements
    drop
    drop
    swap
end

# given 64 -bytes input, in terms of sixteen 32 -bit unsigned integers, where each pair
# of them holding higher & lower 32 -bits of 64 -bit unsigned integer ( reinterpreted on
# host CPU from little endian byte array ) respectively, this function computes 32 -bytes
# keccak256 digest, held on stack top, represented in terms of eight 32 -bit unsigned integers,
# where each pair of them keeps higher and lower 32 -bits of 64 -bit unsigned integer respectively
#
# See https://github.com/itzmeanjan/merklize-sha/blob/1d35aae9da7fed20127489f362b4bc93242a516c/include/keccak_256.hpp#L232-L257
export.hash.13
    push.0.0.0
    push.env.locaddr.12

    push.env.locaddr.11
    push.env.locaddr.10
    push.env.locaddr.9
    push.env.locaddr.8

    push.env.locaddr.7
    push.env.locaddr.6
    push.env.locaddr.5
    push.env.locaddr.4

    push.env.locaddr.3
    push.env.locaddr.2
    push.env.locaddr.1
    push.env.locaddr.0

    exec.to_state_array

    push.0.0.0
    push.env.locaddr.12

    push.env.locaddr.11
    push.env.locaddr.10
    push.env.locaddr.9
    push.env.locaddr.8

    push.env.locaddr.7
    push.env.locaddr.6
    push.env.locaddr.5
    push.env.locaddr.4

    push.env.locaddr.3
    push.env.locaddr.2
    push.env.locaddr.1
    push.env.locaddr.0

    exec.keccak_p

    pushw.local.1
    pushw.local.0

    exec.to_digest
end
"),
// ----- std::crypto::hashes::sha256 --------------------------------------------------------------
("std::crypto::hashes::sha256", "# Given [x, ...] on stack top, this routine computes [y, ...]
# such that y = σ_0(x), as defined in SHA specification
#
# See https://github.com/itzmeanjan/merklize-sha/blob/8a2c006/include/sha2.hpp#L73-L79
proc.small_sigma_0
    dup
    u32unchecked_rotr.7

    swap

    dup
    u32unchecked_rotr.18

    swap

    u32unchecked_shr.3

    u32checked_xor
    u32checked_xor
end

# Given [x, ...] on stack top, this routine computes [y, ...]
# such that y = σ_1(x), as defined in SHA specification
#
# See https://github.com/itzmeanjan/merklize-sha/blob/8a2c006/include/sha2.hpp#L81-L87
proc.small_sigma_1
    dup
    u32unchecked_rotr.17

    swap

    dup
    u32unchecked_rotr.19

    swap

    u32unchecked_shr.10

    u32checked_xor
    u32checked_xor
end

# Given [x, ...] on stack top, this routine computes [y, ...]
# such that y = Σ_0(x), as defined in SHA specification
#
# See https://github.com/itzmeanjan/merklize-sha/blob/8a2c006/include/sha2.hpp#L57-L63
proc.cap_sigma_0
    dup
    u32unchecked_rotr.2

    swap

    dup
    u32unchecked_rotr.13

    swap

    u32unchecked_rotr.22

    u32checked_xor
    u32checked_xor
end

# Given [x, ...] on stack top, this routine computes [y, ...]
# such that y = Σ_1(x), as defined in SHA specification
#
# See https://github.com/itzmeanjan/merklize-sha/blob/8a2c006/include/sha2.hpp#L65-L71
proc.cap_sigma_1
    dup
    u32unchecked_rotr.6

    swap

    dup
    u32unchecked_rotr.11

    swap

    u32unchecked_rotr.25

    u32checked_xor
    u32checked_xor
end

# Given [x, y, z, ...] on stack top, this routine computes [o, ...]
# such that o = ch(x, y, z), as defined in SHA specification
#
# See https://github.com/itzmeanjan/merklize-sha/blob/8a2c006/include/sha2.hpp#L37-L45
proc.ch
    swap
    dup.1
    u32checked_and

    swap
    u32checked_not

    movup.2
    u32checked_and

    u32checked_xor
end

# Given [x, y, z, ...] on stack top, this routine computes [o, ...]
# such that o = maj(x, y, z), as defined in SHA specification
#
# See https://github.com/itzmeanjan/merklize-sha/blob/8a2c006/include/sha2.hpp#L47-L55
proc.maj
    dup.1
    dup.1
    u32checked_and

    swap
    dup.3
    u32checked_and

    movup.2
    movup.3
    u32checked_and

    u32checked_xor
    u32checked_xor
end

# Given [a, b, c, d, ...] on stack top, this routine reverses order of first 
# four elements on stack top such that final stack state looks like [d, c, b, a, ...]
proc.rev_element_order
    swap
    movup.2
    movup.3
end

# Given [a, b, c, d, ...] on stack top, this routine computes next message schedule word
# using following formula
#
# t0 = small_sigma_1(a) + b
# t1 = small_sigma_0(c) + d
# return t0 + t1
#
# If to be computed message schedule word has index i ∈ [16, 64), then 
# a, b, c, d will have following indices in message schedule
#
# a = msg[i - 2]
# b = msg[i - 7]
# c = msg[i - 15]
# d = msg[i - 16]
proc.compute_message_schedule_word
    exec.small_sigma_1
    movup.2
    exec.small_sigma_0

    u32overflowing_add3
    drop
    u32wrapping_add
end

# Given eight working variables of SHA256 ( i.e. hash state ), a 32 -bit round constant & 
# 32 -bit message word on stack top, this routine consumes constant & message word into 
# hash state.
#
# Expected stack state looks like
#
# [a, b, c, d, e, f, g, h, CONST_i, WORD_i] | i ∈ [0, 64)
#
# After finishing execution, stack looks like
#
# [a', b', c', d', e', f', g', h']
#
# See https://github.com/itzmeanjan/merklize-sha/blob/8a2c006/include/sha2_256.hpp#L165-L175
proc.consume_message_word
    dup.6
    dup.6
    dup.6
    exec.ch

    movup.9
    movup.10

    u32overflowing_add3
    drop

    dup.5
    exec.cap_sigma_1

    movup.9
    u32overflowing_add3
    drop

    dup.3
    dup.3
    dup.3
    exec.maj

    dup.2
    exec.cap_sigma_0

    u32wrapping_add

    movup.5
    dup.2
    u32wrapping_add
    movdn.5

    u32wrapping_add
end

# Given 32 -bytes hash state ( in terms of 8 SHA256 words ) and 64 -bytes input 
# message ( in terms of 16 SHA256 words ) on stack top, this routine computes
# whole message schedule of 64 message words and consumes them into hash state.
#
# Expected stack state:
#
# [state0, state1, state2, state3, state4, state5, state6, state7, msg0, msg1, msg2, msg3, msg4, msg5, msg6, msg7, msg8, msg9, msg10, msg11, msg12, msg13, msg14, msg15]
#
# Final stack state after completion of execution
#
# [state0', state1', state2', state3', state4', state5', state6', state7']
#
# Note, each SHA256 word is 32 -bit wide
#
# See https://github.com/itzmeanjan/merklize-sha/blob/8a2c006/include/sha2.hpp#L89-L113
# & https://github.com/itzmeanjan/merklize-sha/blob/8a2c006/include/sha2_256.hpp#L148-L187 ( loop body execution when i = 0 )
proc.prepare_message_schedule_and_consume.2
    popw.local.0
    popw.local.1

    dup.15
    dup.15

    dup.11
    swap
    dup.4
    dup.4
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[16]

    swap
    dup.12
    swap
    dup.5
    dup.5
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[17]

    dup.1
    dup.14
    swap
    dup.7
    dup.7
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[18]

    dup.15
    dup.2
    dup.9
    dup.9
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[19]

    swapw

    push.0x428a2f98
    pushw.local.1
    pushw.local.0
    exec.consume_message_word # consume msg[0]

    push.0x71374491
    movdn.8
    exec.consume_message_word # consume msg[1]

    push.0xb5c0fbcf
    movdn.8
    exec.consume_message_word # consume msg[2]

    push.0xe9b5dba5
    movdn.8
    exec.consume_message_word # consume msg[3]

    popw.local.0
    popw.local.1

    dup.15
    dup.15
    dup.15

    dup.4
    dup.9
    dup.9
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[20]

    swap
    dup.3
    dup.10
    dup.10
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[21]

    movup.2
    dup.2
    dup.11
    dup.11
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[22]

    dup.6
    dup.2
    dup.13
    dup.13
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[23]

    movupw.2

    push.0x3956c25b
    pushw.local.1
    pushw.local.0
    exec.consume_message_word # consume msg[4]

    push.0x59f111f1
    movdn.8
    exec.consume_message_word # consume msg[5]

    push.0x923f82a4
    movdn.8
    exec.consume_message_word # consume msg[6]

    push.0xab1c5ed5
    movdn.8
    exec.consume_message_word # consume msg[7]

    popw.local.0
    popw.local.1

    dup.6
    dup.2
    dup.11
    dup.11
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[24]

    dup.6
    dup.2
    dup.13
    dup.13
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[25]

    dup.6
    dup.2
    dup.15
    dup.15
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[26]

    dup.15
    dup.15
    swap
    dup.8
    dup.4
    exec.compute_message_schedule_word # computed msg[27]

    movupw.3

    push.0xd807aa98
    pushw.local.1
    pushw.local.0
    exec.consume_message_word # consume msg[8]

    push.0x12835b01
    movdn.8
    exec.consume_message_word # consume msg[9]

    push.0x243185be
    movdn.8
    exec.consume_message_word # consume msg[10]

    push.0x550c7dc3
    movdn.8
    exec.consume_message_word # consume msg[11]

    popw.local.0
    popw.local.1

    movupw.3
    movupw.3

    dup.14
    dup.10
    dup.7
    dup.7
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[28]

    dup.14
    dup.10
    dup.9
    dup.9
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[29]

    dup.14
    dup.2
    dup.11
    dup.11
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[30]

    dup.14
    dup.2
    dup.8
    dup.13
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[31]

    movupw.2

    push.0x72be5d74
    pushw.local.1
    pushw.local.0
    exec.consume_message_word # consume msg[12]

    push.0x80deb1fe
    movdn.8
    exec.consume_message_word # consume msg[13]

    push.0x9bdc06a7
    movdn.8
    exec.consume_message_word # consume msg[14]

    push.0xc19bf174
    movdn.8
    exec.consume_message_word # consume msg[15]

    popw.local.0
    popw.local.1

    movupw.3

    dup.14
    dup.6
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[32]

    dup.14
    dup.6
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[33]

    dup.14
    dup.2
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[34]

    dup.10
    dup.2
    dup.8
    dup.14
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[35]

    movupw.3
    exec.rev_element_order

    push.0xe49b69c1
    pushw.local.1
    pushw.local.0
    exec.consume_message_word # consume msg[16]

    push.0xefbe4786
    movdn.8
    exec.consume_message_word # consume msg[17]

    push.0x0fc19dc6
    movdn.8
    exec.consume_message_word # consume msg[18]

    push.0x240ca1cc
    movdn.8
    exec.consume_message_word # consume msg[19]

    popw.local.0
    popw.local.1

    movupw.3

    dup.14
    dup.6
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[36]

    dup.14
    dup.6
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[37]

    dup.14
    dup.2
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[38]

    dup.10
    dup.2
    dup.8
    dup.14
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[39]

    movupw.3
    exec.rev_element_order

    push.0x2de92c6f
    pushw.local.1
    pushw.local.0
    exec.consume_message_word # consume msg[20]

    push.0x4a7484aa
    movdn.8
    exec.consume_message_word # consume msg[21]

    push.0x5cb0a9dc
    movdn.8
    exec.consume_message_word # consume msg[22]

    push.0x76f988da
    movdn.8
    exec.consume_message_word # consume msg[23]

    popw.local.0
    popw.local.1

    movupw.3

    dup.14
    dup.6
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[40]

    dup.14
    dup.6
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[41]

    dup.14
    dup.2
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[42]

    dup.10
    dup.2
    dup.13
    dup.9
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[43]

    movupw.3
    exec.rev_element_order

    push.0x983e5152
    pushw.local.1
    pushw.local.0
    exec.consume_message_word # consume msg[24]

    push.0xa831c66d
    movdn.8
    exec.consume_message_word # consume msg[25]

    push.0xb00327c8
    movdn.8
    exec.consume_message_word # consume msg[26]

    push.0xbf597fc7
    movdn.8
    exec.consume_message_word # consume msg[27]

    popw.local.0
    popw.local.1

    movupw.3

    dup.14
    dup.6
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[44]

    dup.14
    dup.6
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[45]

    dup.14
    dup.2
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[46]

    dup.10
    dup.2
    dup.8
    dup.14
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[47]

    movupw.3
    exec.rev_element_order

    push.0xc6e00bf3
    pushw.local.1
    pushw.local.0
    exec.consume_message_word # consume msg[28]

    push.0xd5a79147
    movdn.8
    exec.consume_message_word # consume msg[29]

    push.0x06ca6351
    movdn.8
    exec.consume_message_word # consume msg[30]

    push.0x14292967
    movdn.8
    exec.consume_message_word # consume msg[31]

    popw.local.0
    popw.local.1

    movupw.3

    dup.14
    dup.6
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[48]

    dup.14
    dup.6
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[49]

    dup.14
    dup.2
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[50]

    dup.10
    dup.2
    dup.8
    dup.14
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[51]

    movupw.3
    exec.rev_element_order

    push.0x27b70a85
    pushw.local.1
    pushw.local.0
    exec.consume_message_word # consume msg[32]

    push.0x2e1b2138
    movdn.8
    exec.consume_message_word # consume msg[33]

    push.0x4d2c6dfc
    movdn.8
    exec.consume_message_word # consume msg[34]

    push.0x53380d13
    movdn.8
    exec.consume_message_word # consume msg[35]

    popw.local.0
    popw.local.1

    movupw.3

    dup.14
    dup.6
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[52]

    dup.14
    dup.6
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[53]

    dup.14
    dup.2
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[54]

    dup.10
    dup.2
    dup.8
    dup.14
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[55]

    movupw.3
    exec.rev_element_order

    push.0x650a7354
    pushw.local.1
    pushw.local.0
    exec.consume_message_word # consume msg[36]

    push.0x766a0abb
    movdn.8
    exec.consume_message_word # consume msg[37]

    push.0x81c2c92e
    movdn.8
    exec.consume_message_word # consume msg[38]

    push.0x92722c85
    movdn.8
    exec.consume_message_word # consume msg[39]

    popw.local.0
    popw.local.1

    movupw.3

    dup.14
    dup.6
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[56]

    dup.14
    dup.6
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[57]

    dup.14
    dup.2
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[58]

    dup.10
    dup.2
    dup.8
    dup.14
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[59]

    movupw.3
    exec.rev_element_order

    push.0xa2bfe8a1
    pushw.local.1
    pushw.local.0
    exec.consume_message_word # consume msg[40]

    push.0xa81a664b
    movdn.8
    exec.consume_message_word # consume msg[41]

    push.0xc24b8b70
    movdn.8
    exec.consume_message_word # consume msg[42]

    push.0xc76c51a3
    movdn.8
    exec.consume_message_word # consume msg[43]

    popw.local.0
    popw.local.1

    movupw.3

    dup.14
    dup.6
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[60]

    dup.14
    dup.6
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[61]

    dup.14
    dup.2
    dup.13
    dup.13
    movdn.3
    movdn.3
    exec.compute_message_schedule_word # computed msg[62]

    dup.10
    dup.2
    dup.8
    dup.14
    movdn.3
    movdn.2
    exec.compute_message_schedule_word # computed msg[63]

    movupw.3
    exec.rev_element_order

    push.0xd192e819
    pushw.local.1
    pushw.local.0
    exec.consume_message_word # consume msg[44]

    push.0xd6990624
    movdn.8
    exec.consume_message_word # consume msg[45]

    push.0xf40e3585
    movdn.8
    exec.consume_message_word # consume msg[46]

    push.0x106aa070
    movdn.8
    exec.consume_message_word # consume msg[47]

    popw.local.0
    popw.local.1

    movupw.2
    movupw.3
    movupw.3

    exec.rev_element_order

    push.0x19a4c116
    pushw.local.1
    pushw.local.0
    exec.consume_message_word # consume msg[48]

    push.0x1e376c08
    movdn.8
    exec.consume_message_word # consume msg[49]

    push.0x2748774c
    movdn.8
    exec.consume_message_word # consume msg[50]

    push.0x34b0bcb5
    movdn.8
    exec.consume_message_word # consume msg[51]

    movupw.2
    exec.rev_element_order
    movdnw.2

    push.0x391c0cb3
    movdn.8
    exec.consume_message_word # consume msg[52]

    push.0x4ed8aa4a
    movdn.8
    exec.consume_message_word # consume msg[53]

    push.0x5b9cca4f
    movdn.8
    exec.consume_message_word # consume msg[54]

    push.0x682e6ff3
    movdn.8
    exec.consume_message_word # consume msg[55]

    movupw.2
    exec.rev_element_order
    movdnw.2

    push.0x748f82ee
    movdn.8
    exec.consume_message_word # consume msg[56]

    push.0x78a5636f
    movdn.8
    exec.consume_message_word # consume msg[57]

    push.0x84c87814
    movdn.8
    exec.consume_message_word # consume msg[58]

    push.0x8cc70208
    movdn.8
    exec.consume_message_word # consume msg[59]

    movupw.2
    exec.rev_element_order
    movdnw.2

    push.0x90befffa
    movdn.8
    exec.consume_message_word # consume msg[60]

    push.0xa4506ceb
    movdn.8
    exec.consume_message_word # consume msg[61]

    push.0xbef9a3f7
    movdn.8
    exec.consume_message_word # consume msg[62]

    push.0xc67178f2
    movdn.8
    exec.consume_message_word # consume msg[63]

    push.0x6a09e667
    u32wrapping_add

    swap
    push.0xbb67ae85
    u32wrapping_add
    swap

    movup.2
    push.0x3c6ef372
    u32wrapping_add
    movdn.2

    movup.3
    push.0xa54ff53a
    u32wrapping_add
    movdn.3

    movup.4
    push.0x510e527f
    u32wrapping_add
    movdn.4

    movup.5
    push.0x9b05688c
    u32wrapping_add
    movdn.5

    movup.6
    push.0x1f83d9ab
    u32wrapping_add
    movdn.6

    movup.7
    push.0x5be0cd19
    u32wrapping_add
    movdn.7
end

# Given 32 -bytes hash state ( in terms of 8 SHA256 words ) and precomputed message 
# schedule of padding bytes ( in terms of 64 message words ), this routine consumes
# that into hash state, leaving final hash state, which is 32 -bytes SHA256 digest.
#
# Note, in SHA256 2-to-1 hashing, 64 -bytes are padded, which is processed as second message
# block ( each SHA256 message block is 64 -bytes wide ). That message block is used for generating 
# message schedule of 64 SHA256 words. That's exactly what can be precomputed & is consumed here 
# ( in this routine ) into provided hash state.
#
# Expected stack state:
#
# [state0, state1, state2, state3, state4, state5, state6, state7, ...]
#
# Final stack state after completion of execution
#
# [state0', state1', state2', state3', state4', state5', state6', state7']
#
# Note, each SHA256 word is 32 -bit wide
#
# See https://github.com/itzmeanjan/merklize-sha/blob/8a2c006/include/sha2_256.hpp#L148-L187 ( loop 
# body execution when i = 1 i.e. consuming padding bytes )
proc.consume_padding_message_schedule
    dupw.1
    dupw.1

    push.2147483648
    movdn.8
    push.0x428a2f98
    movdn.8
    exec.consume_message_word # consume msg[0]

    push.0
    movdn.8
    push.0x71374491
    movdn.8
    exec.consume_message_word # consume msg[1]

    push.0
    movdn.8
    push.0xb5c0fbcf
    movdn.8
    exec.consume_message_word # consume msg[2]

    push.0
    movdn.8
    push.0xe9b5dba5
    movdn.8
    exec.consume_message_word # consume msg[3]

    push.0
    movdn.8
    push.0x3956c25b
    movdn.8
    exec.consume_message_word # consume msg[4]

    push.0
    movdn.8
    push.0x59f111f1
    movdn.8
    exec.consume_message_word # consume msg[5]

    push.0
    movdn.8
    push.0x923f82a4
    movdn.8
    exec.consume_message_word # consume msg[6]

    push.0
    movdn.8
    push.0xab1c5ed5
    movdn.8
    exec.consume_message_word # consume msg[7]

    push.0
    movdn.8
    push.0xd807aa98
    movdn.8
    exec.consume_message_word # consume msg[8]

    push.0
    movdn.8
    push.0x12835b01
    movdn.8
    exec.consume_message_word # consume msg[9]

    push.0
    movdn.8
    push.0x243185be
    movdn.8
    exec.consume_message_word # consume msg[10]

    push.0
    movdn.8
    push.0x550c7dc3
    movdn.8
    exec.consume_message_word # consume msg[11]

    push.0
    movdn.8
    push.0x72be5d74
    movdn.8
    exec.consume_message_word # consume msg[12]

    push.0
    movdn.8
    push.0x80deb1fe
    movdn.8
    exec.consume_message_word # consume msg[13]

    push.0
    movdn.8
    push.0x9bdc06a7
    movdn.8
    exec.consume_message_word # consume msg[14]

    push.512
    movdn.8
    push.0xc19bf174
    movdn.8
    exec.consume_message_word # consume msg[15]

    push.2147483648
    movdn.8
    push.0xe49b69c1
    movdn.8
    exec.consume_message_word # consume msg[16]

    push.20971520
    movdn.8
    push.0xefbe4786
    movdn.8
    exec.consume_message_word # consume msg[17]

    push.2117632
    movdn.8
    push.0x0fc19dc6
    movdn.8
    exec.consume_message_word # consume msg[18]

    push.20616
    movdn.8
    push.0x240ca1cc
    movdn.8
    exec.consume_message_word # consume msg[19]

    push.570427392
    movdn.8
    push.0x2de92c6f
    movdn.8
    exec.consume_message_word # consume msg[20]

    push.575995924
    movdn.8
    push.0x4a7484aa
    movdn.8
    exec.consume_message_word # consume msg[21]

    push.84449090
    movdn.8
    push.0x5cb0a9dc
    movdn.8
    exec.consume_message_word # consume msg[22]

    push.2684354592
    movdn.8
    push.0x76f988da
    movdn.8
    exec.consume_message_word # consume msg[23]

    push.1518862336
    movdn.8
    push.0x983e5152
    movdn.8
    exec.consume_message_word # consume msg[24]

    push.6067200
    movdn.8
    push.0xa831c66d
    movdn.8
    exec.consume_message_word # consume msg[25]

    push.1496221
    movdn.8
    push.0xb00327c8
    movdn.8
    exec.consume_message_word # consume msg[26]

    push.4202700544
    movdn.8
    push.0xbf597fc7
    movdn.8
    exec.consume_message_word # consume msg[27]

    push.3543279056
    movdn.8
    push.0xc6e00bf3
    movdn.8
    exec.consume_message_word # consume msg[28]

    push.291985753
    movdn.8
    push.0xd5a79147
    movdn.8
    exec.consume_message_word # consume msg[29]

    push.4142317530
    movdn.8
    push.0x06ca6351
    movdn.8
    exec.consume_message_word # consume msg[30]

    push.3003913545
    movdn.8
    push.0x14292967
    movdn.8
    exec.consume_message_word # consume msg[31]

    push.145928272
    movdn.8
    push.0x27b70a85
    movdn.8
    exec.consume_message_word # consume msg[32]

    push.2642168871
    movdn.8
    push.0x2e1b2138
    movdn.8
    exec.consume_message_word # consume msg[33]

    push.216179603
    movdn.8
    push.0x4d2c6dfc
    movdn.8
    exec.consume_message_word # consume msg[34]

    push.2296832490
    movdn.8
    push.0x53380d13
    movdn.8
    exec.consume_message_word # consume msg[35]

    push.2771075893
    movdn.8
    push.0x650a7354
    movdn.8
    exec.consume_message_word # consume msg[36]

    push.1738633033
    movdn.8
    push.0x766a0abb
    movdn.8
    exec.consume_message_word # consume msg[37]

    push.3610378607
    movdn.8
    push.0x81c2c92e
    movdn.8
    exec.consume_message_word # consume msg[38]

    push.1324035729
    movdn.8
    push.0x92722c85
    movdn.8
    exec.consume_message_word # consume msg[39]

    push.1572820453
    movdn.8
    push.0xa2bfe8a1
    movdn.8
    exec.consume_message_word # consume msg[40]

    push.2397971253
    movdn.8
    push.0xa81a664b
    movdn.8
    exec.consume_message_word # consume msg[41]

    push.3803995842
    movdn.8
    push.0xc24b8b70
    movdn.8
    exec.consume_message_word # consume msg[42]

    push.2822718356
    movdn.8
    push.0xc76c51a3
    movdn.8
    exec.consume_message_word # consume msg[43]

    push.1168996599
    movdn.8
    push.0xd192e819
    movdn.8
    exec.consume_message_word # consume msg[44]

    push.921948365
    movdn.8
    push.0xd6990624
    movdn.8
    exec.consume_message_word # consume msg[45]

    push.3650881000
    movdn.8
    push.0xf40e3585
    movdn.8
    exec.consume_message_word # consume msg[46]

    push.2958106055
    movdn.8
    push.0x106aa070
    movdn.8
    exec.consume_message_word # consume msg[47]

    push.1773959876
    movdn.8
    push.0x19a4c116
    movdn.8
    exec.consume_message_word # consume msg[48]

    push.3172022107
    movdn.8
    push.0x1e376c08
    movdn.8
    exec.consume_message_word # consume msg[49]

    push.3820646885
    movdn.8
    push.0x2748774c
    movdn.8
    exec.consume_message_word # consume msg[50]

    push.991993842
    movdn.8
    push.0x34b0bcb5
    movdn.8
    exec.consume_message_word # consume msg[51]

    push.419360279
    movdn.8
    push.0x391c0cb3
    movdn.8
    exec.consume_message_word # consume msg[52]

    push.3797604839
    movdn.8
    push.0x4ed8aa4a
    movdn.8
    exec.consume_message_word # consume msg[53]

    push.322392134
    movdn.8
    push.0x5b9cca4f
    movdn.8
    exec.consume_message_word # consume msg[54]

    push.85264541
    movdn.8
    push.0x682e6ff3
    movdn.8
    exec.consume_message_word # consume msg[55]

    push.1326255876
    movdn.8
    push.0x748f82ee
    movdn.8
    exec.consume_message_word # consume msg[56]

    push.640108622
    movdn.8
    push.0x78a5636f
    movdn.8
    exec.consume_message_word # consume msg[57]

    push.822159570
    movdn.8
    push.0x84c87814
    movdn.8
    exec.consume_message_word # consume msg[58]

    push.3328750644
    movdn.8
    push.0x8cc70208
    movdn.8
    exec.consume_message_word # consume msg[59]

    push.1107837388
    movdn.8
    push.0x90befffa
    movdn.8
    exec.consume_message_word # consume msg[60]

    push.1657999800
    movdn.8
    push.0xa4506ceb
    movdn.8
    exec.consume_message_word # consume msg[61]

    push.3852183409
    movdn.8
    push.0xbef9a3f7
    movdn.8
    exec.consume_message_word # consume msg[62]

    push.2242356356
    movdn.8
    push.0xc67178f2
    movdn.8
    exec.consume_message_word # consume msg[63]

    movup.8
    u32wrapping_add

    swap
    movup.8
    u32wrapping_add
    swap

    movup.2
    movup.8
    u32wrapping_add
    movdn.2

    movup.3
    movup.8
    u32wrapping_add
    movdn.3

    movup.4
    movup.8
    u32wrapping_add
    movdn.4

    movup.5
    movup.8
    u32wrapping_add
    movdn.5

    movup.6
    movup.8
    u32wrapping_add
    movdn.6

    movup.7
    movup.8
    u32wrapping_add
    movdn.7
end

# Given 64 -bytes input, this routine computes 32 -bytes SAH256 digest
#
# Expected stack state:
#
# [m0, m1, m2, m3, m4, m5, m6, m7, m8, m9, m10, m11, m12, m13, m14, m15] | m[0,16) = 32 -bit word
#
# Note, each SHA256 word is 32 -bit wide, so that's how input is expected.
# If you've 64 -bytes, consider packing 4 consecutive bytes into single word, 
# maintaining big endian byte order.
#
# Final stack state:
#
# [dig0, dig1, dig2, dig3, dig4, dig5, dig6, dig7]
#
# SHA256 digest is represented in terms of eight 32 -bit words ( big endian byte order ).
export.hash
    push.0x5be0cd19.0x1f83d9ab.0x9b05688c.0x510e527f
    push.0xa54ff53a.0x3c6ef372.0xbb67ae85.0x6a09e667

    exec.prepare_message_schedule_and_consume
    exec.consume_padding_message_schedule
end
"),
// ----- std::math::u256 --------------------------------------------------------------------------
("std::math::u256", "export.add_unsafe
    swapw.3
    movup.3
    movup.7
    u32overflowing_add
    movup.4
    movup.7
    u32overflowing_add3
    movup.4
    movup.6
    u32overflowing_add3
    movup.4
    movup.5
    u32overflowing_add3
    movdn.12
    swapw.2
    movup.12
    movup.4
    movup.8
    u32overflowing_add3
    movup.4
    movup.7
    u32overflowing_add3
    movup.4
    movup.6
    u32overflowing_add3
    movup.4
    movup.5
    u32overflowing_add3
    drop
end

export.sub_unsafe
    swapw.3
    movup.3
    movup.7
    u32overflowing_sub
    movup.7
    u32overflowing_add
    movup.5
    movup.2
    u32overflowing_sub
    movup.2
    add
    movup.6
    u32overflowing_add
    movup.5
    movup.2
    u32overflowing_sub
    movup.2
    add
    movup.5
    u32overflowing_add
    movup.5
    movup.2
    u32overflowing_sub
    movup.2
    add
    movdn.12
    swapw.2
    movup.12
    movup.4
    u32overflowing_add
    movup.8
    movup.2
    u32overflowing_sub
    movup.2
    add
    movup.4
    u32overflowing_add
    movup.7
    movup.2
    u32overflowing_sub
    movup.2
    add
    movup.4
    u32overflowing_add
    movup.6
    movup.2
    u32overflowing_sub
    movup.2
    add
    movup.5
    movup.5
    movup.2
    u32overflowing_add
    drop
    u32overflowing_sub
    drop
end

export.and
    swapw.3
    movup.3
    movup.7
    u32checked_and
    movup.3
    movup.6
    u32checked_and
    movup.3
    movup.5
    u32checked_and
    movup.3
    movup.4
    u32checked_and
    swapw.2
    movup.3
    movup.7
    u32checked_and
    movup.3
    movup.6
    u32checked_and
    movup.3
    movup.5
    u32checked_and
    movup.3
    movup.4
    u32checked_and
end

export.or
    swapw.3
    movup.3
    movup.7
    u32checked_or
    movup.3
    movup.6
    u32checked_or
    movup.3
    movup.5
    u32checked_or
    movup.3
    movup.4
    u32checked_or
    swapw.2
    movup.3
    movup.7
    u32checked_or
    movup.3
    movup.6
    u32checked_or
    movup.3
    movup.5
    u32checked_or
    movup.3
    movup.4
    u32checked_or
end

export.xor
    swapw.3
    movup.3
    movup.7
    u32checked_xor
    movup.3
    movup.6
    u32checked_xor
    movup.3
    movup.5
    u32checked_xor
    movup.3
    movup.4
    u32checked_xor
    swapw.2
    movup.3
    movup.7
    u32checked_xor
    movup.3
    movup.6
    u32checked_xor
    movup.3
    movup.5
    u32checked_xor
    movup.3
    movup.4
    u32checked_xor
end

export.iszero_unsafe
    eq.0
    repeat.7
        swap
        eq.0
        and
    end
end

export.eq_unsafe
    swapw.3
    eqw
    movdn.8
    dropw
    dropw
    movdn.8
    eqw
    movdn.8
    dropw
    dropw
    and
end

# ===== MULTIPLICATION ============================================================================

proc.mulstep
    movdn.2
    u32overflowing_madd
    movdn.2
    u32overflowing_add
    movup.2
    add
end

proc.mulstep4
    movup.12
    dup.1
    movup.10
    push.0 # start k at 0
    exec.mulstep
    swap
    movdn.9
    dup.1
    movup.9
    movup.13
    swap.3
    exec.mulstep
    swap
    movdn.8
    dup.1
    movup.8
    movup.12
    swap.3
    exec.mulstep
    swap
    movdn.7
    dup.1
    movup.7
    movup.11
    swap.3
    exec.mulstep
    swap
    movdn.6
end

# Performs addition of two unsigned 256 bit integers discarding the overflow.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b7, b6, b5, b4, b3, b2, b1, b0, a7, a6, a5, a4, a3, a2, a1, a0, ...] -> [c7, c6, c5, c4, c3, c2, c1, c0, ...]
# where c = (a * b) % 2^256, and a0, b0, and c0 are least significant 32-bit limbs of a, b, and c respectively.
export.mul_unsafe.6
    # Memory storing setup
    popw.local.0
    # b[5-8] at 0
    storew.local.1
    # b[0-4] at 1
    push.0 dropw
    # b[0] at top of stack, followed by a[0-7]
    movdn.8
    storew.local.2
    # a[0-4] at 2
    swapw
    storew.local.3
    # a[5-8] at 3
    padw
    storew.local.4
    storew.local.5
    # p at 4 and 5

    # b[0]
    dropw
    swapw
    pushw.local.4
    movdnw.2
    movup.12

    exec.mulstep4

    movdn.9
    movdn.9
    swapw
    popw.local.4
    pushw.local.5
    swapw
    movup.9
    movup.9

    dup.1
    movup.6
    movup.10
    swap.3
    exec.mulstep
    swap
    movdn.5
    dup.1
    movup.5
    movup.9
    swap.3
    exec.mulstep
    swap
    movdn.4
    dup.1
    movup.4
    movup.8
    swap.3
    exec.mulstep
    swap
    movdn.3
    swap
    movup.2
    movup.6
    swap.3
    exec.mulstep

    drop
    popw.local.5

    # b[1]
    pushw.local.4
    pushw.local.5
    movup.7
    dropw
    pushw.local.3 pushw.local.2 # load the xs
    pushw.local.1
    movup.2
    movdn.3
    push.0 dropw # only need b[1]

    exec.mulstep4

    movdn.9
    movdn.9
    swapw
    movdn.3
    pushw.local.4
    push.0 dropw # only need p[0]
    movdn.3
    # save p[0-3] to memory, not needed any more
    popw.local.4

    pushw.local.5
    movup.3
    drop
    swapw
    movup.9
    movup.9

    dup.1
    movup.6
    movup.9
    swap.3
    exec.mulstep
    swap
    movdn.7
    dup.1
    movup.5
    movup.7
    swap.3
    exec.mulstep
    swap
    movdn.5
    swap
    movup.3
    movup.4
    swap.3
    exec.mulstep

    drop
    swap
    drop
    popw.local.5

    # b[2]
    pushw.local.4
    pushw.local.5
    movup.7
    movup.7
    dropw
    pushw.local.3 pushw.local.2 # load the xs
    pushw.local.1
    swap
    movdn.3
    push.0 dropw # only need b[1]

    exec.mulstep4

    movdn.9
    movdn.9
    swapw
    movdn.3
    movdn.3
    pushw.local.4
    drop drop
    movdn.3
    movdn.3
    popw.local.4

    pushw.local.5
    movup.3
    movup.3
    drop
    drop
    swapw
    movup.9
    movup.9

    dup.1
    movup.6
    movup.8
    swap.3
    exec.mulstep
    swap
    movdn.6
    dup.1
    movup.5
    movup.6
    swap.3
    exec.mulstep
    swap
    swap drop
    movdn.3
    drop drop drop
    popw.local.5

    # b[3]
    pushw.local.4
    pushw.local.5

    movup.7 movup.7 movup.7
    dropw
    pushw.local.3 pushw.local.2

    pushw.local.1
    movdn.3
    push.0 dropw

    exec.mulstep4

    movdn.9
    movdn.9

    swapw
    movup.3
    pushw.local.4
    drop
    movup.3

    popw.local.4
    pushw.local.5
    movdn.3
    push.0 dropw
    swapw
    movup.9
    movup.9

    swap
    movup.5
    movup.6
    swap.3
    exec.mulstep

    drop
    movdn.3
    push.0 dropw

    # b[4]
    pushw.local.3 pushw.local.2 # load the xs
    # OPTIM: don't need a[4-7], but can't use mulstep4 if we don't load

    pushw.local.0
    push.0 dropw # b[4]

    exec.mulstep4
    dropw drop drop # OPTIM: don't need a[4-7], but can't use mulstep4 if we don't load

    # b[5]
    pushw.local.3
    pushw.local.0
    movup.2 movdn.3
    push.0 dropw
    movup.7
    dup.1
    movup.6
    push.0
    exec.mulstep
    swap
    movdn.7
    movup.4
    dup.2
    movup.7
    swap.3
    exec.mulstep
    swap
    movdn.5
    swap
    movup.3
    movup.4
    swap.3
    exec.mulstep
    drop
    swap
    drop

    # b[6]
    pushw.local.3
    pushw.local.0
    swap
    movdn.3
    push.0 dropw
    movup.6
    dup.1
    movup.6
    push.0
    exec.mulstep
    swap
    movdn.6
    swap
    movup.4
    movup.5
    swap.3
    exec.mulstep
    drop
    movdn.2
    drop drop

    # b[7]
    pushw.local.3
    pushw.local.0

    movdn.3 push.0 dropw
    movup.4
    movup.5
    movdn.2
    push.0
    exec.mulstep
    drop
    movdn.3
    drop drop drop

    pushw.local.4
    swapw
end"),
// ----- std::math::u64 ---------------------------------------------------------------------------
("std::math::u64", "# ===== HELPER FUNCTIONS ==========================================================================

# Asserts that both values at the top of the stack are u64 values.
# The input values are assumed to be represented using 32 bit limbs, fails if they are not.
proc.u32assert4
    u32assert
    movup.3
    u32assert
    movup.3
    u32assert
    movup.3
    u32assert
    movup.3
end

# ===== ADDITION ==================================================================================

# Performs addition of two unsigned 64 bit integers preserving the overflow.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [overflowing_flag, c_hi, c_lo, ...], where c = (a + b) % 2^64
export.overflowing_add
    swap
    movup.3
    u32overflowing_add
    movup.3
    movup.3
    u32overflowing_add3
end

# Performs addition of two unsigned 64 bit integers discarding the overflow.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = (a + b) % 2^64
export.wrapping_add
    exec.overflowing_add
    drop
end

# Performs addition of two unsigned 64 bit integers, fails when overflowing.
# The input values are assumed to be represented using 32 bit limbs, fails if they are not.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = (a + b) % 2^64
export.checked_add
    exec.u32assert4
    exec.overflowing_add
    eq.0
    assert
end

# ===== SUBTRACTION ===============================================================================

# Performs subtraction of two unsigned 64 bit integers discarding the overflow.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = (a - b) % 2^64
export.wrapping_sub
    movup.3
    movup.2
    u32overflowing_sub
    movup.3
    movup.3
    u32overflowing_sub
    drop
    swap
    u32overflowing_sub
    drop
end

# Performs subtraction of two unsigned 64 bit integers, fails when underflowing.
# The input values are assumed to be represented using 32 bit limbs, fails if they are not.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = (a - b) % 2^64
export.checked_sub
    exec.u32assert4
    movup.3
    movup.2
    u32overflowing_sub
    movup.3
    movup.3
    u32overflowing_sub
    eq.0
    assert
    swap
    u32overflowing_sub
    eq.0
    assert
end

# Performs subtraction of two unsigned 64 bit integers preserving the overflow.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [underflowing_flag, c_hi, c_lo, ...], where c = (a - b) % 2^64
export.overflowing_sub
    movup.3
    movup.2
    u32overflowing_sub
    movup.3
    movup.3
    u32overflowing_sub
    swap
    movup.2
    u32overflowing_sub
    movup.2
    or
end

# ===== MULTIPLICATION ============================================================================

# Performs multiplication of two unsigned 64 bit integers discarding the overflow.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = (a * b) % 2^64
export.wrapping_mul
    dup.3
    dup.2
    u32overflowing_mul
    movup.4
    movup.4
    u32overflowing_madd
    drop
    movup.3
    movup.3
    u32overflowing_madd
    drop
end

# Performs multiplication of two unsigned 64 bit integers preserving the overflow.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_mid_hi, c_mid_lo, c_lo, ...], where c = (a * b) % 2^64
# This takes 18 cycles.
export.overflowing_mul
    dup.3
    dup.2
    u32overflowing_mul
    dup.4
    movup.4
    u32overflowing_madd
    swap
    movup.5
    dup.4
    u32overflowing_madd
    movup.5
    movup.5
    u32overflowing_madd
    movup.3
    movup.2
    u32overflowing_add
    movup.2
    add
end

# Performs multiplication of two unsigned 64 bit integers, fails when overflowing.
# The input values are assumed to be represented using 32 bit limbs, fails if they are not.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = (a * b) % 2^64
export.checked_mul
    exec.u32assert4
    exec.overflowing_mul
    u32checked_or
    eq.0
    assert
end

# ===== COMPARISONS ===============================================================================

# Performs less-than comparison of two unsigned 64 bit integers.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c, ...], where c = 1 when a < b, and 0 otherwise.
export.unchecked_lt
    movup.3
    movup.2
    u32overflowing_sub
    movdn.3
    drop
    u32overflowing_sub
    swap
    eq.0
    movup.2
    and
    or
end

# Performs less-than comparison of two unsigned 64 bit integers.
# The input values are assumed to be represented using 32 bit limbs, fails if they are not.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c, ...], where c = 1 when a < b, and 0 otherwise.
export.checked_lt
    exec.u32assert4
    exec.unchecked_lt
end

# Performs greater-than comparison of two unsigned 64 bit integers.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c, ...], where c = 1 when a > b, and 0 otherwise.
# This takes 11 cycles.
export.unchecked_gt
    movup.2
    u32overflowing_sub
    movup.2
    movup.3
    u32overflowing_sub
    swap
    drop
    movup.2
    eq.0
    and
    or
end

# Performs greater-than comparison of two unsigned 64 bit integers.
# The input values are assumed to be represented using 32 bit limbs, fails if they are not.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c, ...], where c = 1 when a > b, and 0 otherwise.
export.checked_gt
    exec.u32assert4
    exec.unchecked_gt
end

# Performs less-than-or-equal comparison of two unsigned 64 bit integers.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c, ...], where c = 1 when a <= b, and 0 otherwise.
export.unchecked_lte
    exec.unchecked_gt
    not
end

# Performs less-than-or-equal comparison of two unsigned 64 bit integers.
# The input values are assumed to be represented using 32 bit limbs, fails if they are not.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c, ...], where c = 1 when a <= b, and 0 otherwise.
export.checked_lte
    exec.u32assert4
    exec.unchecked_gt
    not
end

# Performs greater-than-or-equal comparison of two unsigned 64 bit integers.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c, ...], where c = 1 when a >= b, and 0 otherwise.
export.unchecked_gte
    exec.unchecked_lt
    not
end

# Performs greater-than-or-equal comparison of two unsigned 64 bit integers.
# The input values are assumed to be represented using 32 bit limbs, fails if they are not.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c, ...], where c = 1 when a >= b, and 0 otherwise.
export.checked_gte
    exec.u32assert4
    exec.unchecked_lt
    not
end

# Performs equality comparison of two unsigned 64 bit integers.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c, ...], where c = 1 when a == b, and 0 otherwise.
export.unchecked_eq
    movup.2
    u32checked_eq
    swap
    movup.2
    u32checked_eq
    and
end

# Performs equality comparison of two unsigned 64 bit integers.
# The input values are assumed to be represented using 32 bit limbs, fails if they are not.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c, ...], where c = 1 when a == b, and 0 otherwise.
export.checked_eq
    exec.u32assert4
    exec.unchecked_eq
end

# Performs inequality comparison of two unsigned 64 bit integers.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c, ...], where c = 1 when a != b, and 0 otherwise.
export.unchecked_neq
    movup.2
    u32checked_neq
    swap
    movup.2
    u32checked_neq
    or
end

# Performs inequality comparison of two unsigned 64 bit integers.
# The input values are assumed to be represented using 32 bit limbs, fails if they are not.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c, ...], where c = 1 when a == b, and 0 otherwise.
export.checked_neq
    exec.u32assert4
    exec.unchecked_eq
end

# Performs comparison to zero of an unsigned 64 bit integer.
# The input value is assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [a_hi, a_lo, ...] -> [c, ...], where c = 1 when a == 0, and 0 otherwise.
export.unchecked_eqz
    eq.0
    swap
    eq.0
    and
end

# Performs comparison to zero of an unsigned 64 bit integer.
# The input value is assumed to be represented using 32 bit limbs, fails if it is not.
# Stack transition looks as follows:
# [a_hi, a_lo, ...] -> [c, ...], where c = 1 when a == 0, and 0 otherwise.
export.checked_eqz
    u32assert
    swap
    u32assert
    swap
    eq.0
    swap
    eq.0
    and
end

# Compares two unsigned 64 bit integers and drop the larger one from the stack.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = a when a < b, and b otherwise.
export.unchecked_min
    dupw
    exec.unchecked_gt
    movup.4
    movup.3
    dup.2
    cdrop
    movdn.3
    cdrop
end

# Compares two unsigned 64 bit integers and drop the larger one from the stack.
# The input values are assumed to be represented using 32 bit limbs, fails if they are not.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = a when a < b, and b otherwise.
export.checked_min
    exec.u32assert4
    exec.unchecked_min
end

# Compares two unsigned 64 bit integers and drop the smaller one from the stack.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = a when a > b, and b otherwise.
export.unchecked_max
    dupw
    exec.unchecked_lt
    movup.4
    movup.3
    dup.2
    cdrop
    movdn.3
    cdrop
end

# Compares two unsigned 64 bit integers and drop the smaller one from the stack.
# The input values are assumed to be represented using 32 bit limbs, fails if they are not.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = a when a > b, and b otherwise.
export.checked_max
    exec.u32assert4
    exec.unchecked_max
end


# ===== DIVISION ==================================================================================

# Performs division of two unsigned 64 bit integers discarding the remainder.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = a // b
export.unchecked_div
    adv.u64div          # inject the quotient and the remainder into the advice tape

    push.adv.1          # read the quotient from the advice tape and make sure it consists of
    u32assert           # 32-bit limbs
    push.adv.1          # TODO: this can be optimized once we have u32assert2 instruction
    u32assert

    dup.3               # multiply quotient by the divisor and make sure the resulting value
    dup.2               # fits into 2 32-bit limbs
    u32overflowing_mul
    dup.4
    dup.4
    u32overflowing_madd
    eq.0
    assert
    dup.5
    dup.3
    u32overflowing_madd
    eq.0
    assert
    dup.4
    dup.3
    mul
    eq.0
    assert

    push.adv.1          # read the remainder from the advice tape and make sure it consists of
    u32assert           # 32-bit limbs
    push.adv.1
    u32assert

    movup.7             # make sure the divisor is greater than the remainder. this also consumes
    movup.7             # the divisor
    dup.3
    dup.3
    exec.unchecked_gt
    assert

    swap                # add remainder to the previous result; this also consumes the remainder
    movup.3
    u32overflowing_add
    movup.3
    movup.3
    u32overflowing_add3
    eq.0
    assert

    movup.4             # make sure the result we got is equal to the dividend
    assert.eq
    movup.3
    assert.eq           # quotient remains on the stack
end

# Performs division of two unsigned 64 bit integers discarding the remainder.
# The input values are assumed to be represented using 32 bit limbs, fails if they are not.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = a // b
export.checked_div
    exec.u32assert4
    exec.unchecked_div
end

# ===== MODULO OPERATION ==========================================================================

# Performs modulo operation of two unsigned 64 bit integers.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = a % b
export.unchecked_mod
    adv.u64div          # inject the quotient and the remainder into the advice tape

    push.adv.1          # read the quotient from the advice tape and make sure it consists of
    u32assert           # 32-bit limbs
    push.adv.1          # TODO: this can be optimized once we have u32assert2 instruction
    u32assert

    dup.3               # multiply quotient by the divisor and make sure the resulting value
    dup.2               # fits into 2 32-bit limbs
    u32overflowing_mul
    dup.4
    movup.4
    u32overflowing_madd
    eq.0
    assert
    dup.4
    dup.3
    u32overflowing_madd
    eq.0
    assert
    dup.3
    movup.3
    mul
    eq.0
    assert

    push.adv.1          # read the remainder from the advice tape and make sure it consists of
    u32assert           # 32-bit limbs
    push.adv.1
    u32assert

    movup.5             # make sure the divisor is greater than the remainder. this also consumes
    movup.5             # the divisor
    dup.3
    dup.3
    exec.unchecked_gt
    assert

    dup.1               # add remainder to the previous result
    movup.4
    u32overflowing_add
    movup.4
    dup.3
    u32overflowing_add3
    eq.0
    assert

    movup.4             # make sure the result we got is equal to the dividend
    assert.eq
    movup.3
    assert.eq           # remainder remains on the stack
end

# Performs modulo operation of two unsigned 64 bit integers.
# The input values are assumed to be represented using 32 bit limbs, fails if they are not.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = a % b
export.checked_mod
    exec.u32assert4
    exec.unchecked_mod
end

# ===== DIVMOD OPERATION ==========================================================================

# Performs divmod operation of two unsigned 64 bit integers.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [r_hi, r_lo, q_hi, q_lo ...], where r = a % b, q = a / b
export.unchecked_divmod
    adv.u64div          # inject the quotient and the remainder into the advice tape

    push.adv.1          # read the quotient from the advice tape and make sure it consists of
    u32assert           # 32-bit limbs
    push.adv.1          # TODO: this can be optimized once we have u32assert2 instruction
    u32assert

    dup.3               # multiply quotient by the divisor and make sure the resulting value
    dup.2               # fits into 2 32-bit limbs
    u32overflowing_mul
    dup.4
    dup.4
    u32overflowing_madd
    eq.0
    assert
    dup.5
    dup.3
    u32overflowing_madd
    eq.0
    assert
    dup.4
    dup.3
    mul
    eq.0
    assert

    push.adv.1          # read the remainder from the advice tape and make sure it consists of
    u32assert           # 32-bit limbs 
    push.adv.1
    u32assert

    movup.7             # make sure the divisor is greater than the remainder. this also consumes
    movup.7             # the divisor
    dup.3
    dup.3
    exec.unchecked_gt
    assert

    dup.1               # add remainder to the previous result
    movup.4
    u32overflowing_add
    movup.4
    dup.3
    u32overflowing_add3
    eq.0
    assert

    movup.6             # make sure the result we got is equal to the dividend
    assert.eq
    movup.5
    assert.eq           # remainder remains on the stack
end

# Performs divmod operation of two unsigned 64 bit integers.
# The input values are assumed to be represented using 32 bit limbs, fails if they are not.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [r_hi, r_lo, q_hi, q_lo ...], where r = a % b, q = a / b
export.checked_divmod
    exec.u32assert4
    exec.unchecked_divmod
end

# ===== BITWISE OPERATIONS ========================================================================

# Performs bitwise AND of two unsigned 64-bit integers.
# The input values are assumed to be represented using 32 bit limbs, but this is not checked.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = a AND b.
export.checked_and
    swap
    movup.3
    u32checked_and
    swap
    movup.2
    u32checked_and
end

# Performs bitwise OR of two unsigned 64 bit integers.
# The input values are assumed to be represented using 32 bit limbs, fails if they are not.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = a OR b.
export.checked_or
    swap
    movup.3
    u32checked_or
    swap
    movup.2
    u32checked_or
end

# Performs bitwise XOR of two unsigned 64 bit integers.
# The input values are assumed to be represented using 32 bit limbs, fails if they are not.
# Stack transition looks as follows:
# [b_hi, b_lo, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = a XOR b.
export.checked_xor
    swap
    movup.3
    u32checked_xor
    swap
    movup.2
    u32checked_xor
end

# Performs left shift of one unsigned 64-bit integer using the pow2 operation.
# The input value to be shifted is assumed to be represented using 32 bit limbs.
# The shift value is assumed to be in the range [0, 64).
# Stack transition looks as follows:
# [b, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = a << b mod 2^64.
# This takes 50 cycles.
export.unchecked_shl
    unchecked_pow2
    u32split
    exec.wrapping_mul
end


# Performs right shift of one unsigned 64-bit integer using the pow2 operation.
# The input value to be shifted is assumed to be represented using 32 bit limbs.
# The shift value is assumed to be in the range [0, 64).
# Stack transition looks as follows:
# [b, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = a >> b.
# This takes 66 cycles.
export.unchecked_shr
    unchecked_pow2
    u32split

    dup.1
    add
    movup.2
    swap
    u32unchecked_divmod
    movup.3
    movup.3
    dup
    eq.0
    u32overflowing_sub
    not
    movdn.4
    dup
    movdn.4
    u32unchecked_divmod
    drop
    push.4294967296
    dup.5
    mul
    movup.4
    div
    movup.2
    mul
    add
    movup.2
    cswap
end

# Performs left shift of one unsigned 64-bit integer preserving the overflow and
# using the pow2 operation.
# The input value to be shifted is assumed to be represented using 32 bit limbs.
# The shift value is assumed to be in the range [0, 64).
# Stack transition looks as follows:
# [b, a_hi, a_lo, ...] -> [d_hi, d_lo, c_hi, c_lo, ...], where (d,c) = a << b, 
# which d contains the bits shifted out.
# This takes 57 cycles.
export.overflowing_shl
    unchecked_pow2
    u32split
    exec.overflowing_mul
end

# Performs right shift of one unsigned 64-bit integer preserving the overflow and
# using the pow2 operation.
# The input value to be shifted is assumed to be represented using 32 bit limbs.
# The shift value is assumed to be in the range [0, 64).
# Stack transition looks as follows:
# [b, a_hi, a_lo, ...] -> [d_hi, d_lo, c_hi, c_lo, ...], where c = a >> b, d = a << (64 - b).
# This takes 138 cycles.
export.overflowing_shr
    push.64             # (64 - b)
    dup.1
    sub

    dup.3               # dup [b, a_hi, a_lo]
    dup.3
    dup.3
    exec.unchecked_shr  # c = a >> b

    movdn.5             # move result [c_hi, c_lo] to be in the format [d_hi, d_lo, c_hi, c_lo, ...]
    movdn.5

    padw                # padding positions 0, 1, 2, 3 and 4 to be able to use cdropw
    push.0

    movup.6             # bring and b
    eq.0
    cdropw              # if b is 0, swap the positions 0, 1, 2 and 3 with 0, (64 - b), a_hi, a_lo
                        # regardless of this condition, drop 0, 1, 2 and 3
    drop                # drop the last added 0 or dup b to keep the format [b, a_hi, a_lo, ....]

    exec.unchecked_shl  # d = a << (64 - b)
end

# Performs left rotation of one unsigned 64-bit integer using the pow2 operation.
# The input value to be shifted is assumed to be represented using 32 bit limbs.
# The shift value is assumed to be in the range [0, 64).
# Stack transition looks as follows:
# [b, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = a << b mod 2^64.
# This takes 57 cycles.
export.unchecked_rotl
    push.31
    dup.1
    u32overflowing_sub
    swap
    drop
    movdn.3

    # Shift the low limb.
    push.31
    u32checked_and
    unchecked_pow2
    dup
    movup.3
    u32overflowing_mul

    # Shift the high limb.
    movup.3
    movup.3
    u32overflowing_madd

    # Carry the overflow shift to the low bits.
    movup.2
    add
    swap

    # Conditionally select the limb order based on whether it's shifting by > 31 or not.
    movup.2
    cswap
end

# Performs right rotation of one unsigned 64-bit integer using the pow2 operation.
# The input value to be shifted is assumed to be represented using 32 bit limbs.
# The shift value is assumed to be in the range [0, 64).
# Stack transition looks as follows:
# [b, a_hi, a_lo, ...] -> [c_hi, c_lo, ...], where c = a << b mod 2^64.
# This takes 62 cycles.
export.unchecked_rotr
    push.31
    dup.1
    u32overflowing_sub
    swap
    drop
    movdn.3

    # Shift the low limb left by 32-b.
    push.31
    u32checked_and
    push.32
    swap
    u32overflowing_sub
    drop
    unchecked_pow2
    dup
    movup.3
    u32overflowing_mul

    # Shift the high limb left by 32-b.
    movup.3
    movup.3
    u32overflowing_madd

    # Carry the overflow shift to the low bits.
    movup.2
    add
    swap

    # Conditionally select the limb order based on whether it's shifting by > 31 or not.
    movup.2
    not
    cswap
end
"),
// ----- std::sys ---------------------------------------------------------------------------------
("std::sys", "# Removes elements deep in the stack until the depth of the stack is exactly 16. The elements
# are removed in such a way that the top 16 elements of the stack remain unchanged.
# Input: Stack with 16 or more elements.
# Output: Stack with only the original top 16 elements.
export.finalize_stack.4
    popw.local.0
    popw.local.1
    popw.local.2
    popw.local.3
    push.env.sdepth
    neq.16
    while.true
        dropw
        push.env.sdepth
        neq.16
    end
    loadw.local.3
    swapw.3
    loadw.local.2
    swapw.2
    loadw.local.1
    swapw.1
    loadw.local.0
end
"),
];
