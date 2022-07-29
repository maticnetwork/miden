
## std::crypto::hashes::sha256
| Procedure | Description |
| ----------- | ------------- |
| hash |  Given 64 -bytes input, this routine computes 32 -bytes SAH256 digest<br /><br /> Expected stack state:<br /><br /> [m0, m1, m2, m3, m4, m5, m6, m7, m8, m9, m10, m11, m12, m13, m14, m15] \| m[0,16) = 32 -bit word<br /><br /> Note, each SHA256 word is 32 -bit wide, so that's how input is expected.<br /> If you've 64 -bytes, consider packing 4 consecutive bytes into single word, <br /> maintaining big endian byte order.<br /><br /> Final stack state:<br /><br /> [dig0, dig1, dig2, dig3, dig4, dig5, dig6, dig7]<br /><br /> SHA256 digest is represented in terms of eight 32 -bit words ( big endian byte order ). |
