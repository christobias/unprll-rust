// Copyright (c) 2019, The Unprll Project
// Copyright (c) 2014-2018, The Monero Project
// 
// All rights reserved.
// 
// Redistribution and use in source and binary forms, with or without modification, are
// permitted provided that the following conditions are met:
// 
// 1. Redistributions of source code must retain the above copyright notice, this list of
//    conditions and the following disclaimer.
// 
// 2. Redistributions in binary form must reproduce the above copyright notice, this list
//    of conditions and the following disclaimer in the documentation and/or other
//    materials provided with the distribution.
// 
// 3. Neither the name of the copyright holder nor the names of its contributors may be
//    used to endorse or promote products derived from this software without specific
//    prior written permission.
// 
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY
// EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL
// THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
// STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF
// THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
// 
// Parts of this file are originally copyright (c) 2012-2013 The Cryptonote developers

#include <assert.h>
#include <stdint.h>

#include "fe.c"

/* From ge.h */

typedef struct {
  fe X;
  fe Y;
  fe Z;
} ge_p2;

void ge_fromfe_frombytes_vartime(ge_p2 *r, const unsigned char *s) {
  fe u, v, w, x, y, z;
  unsigned char sign;

  /* From fe_frombytes.c */

  int64_t h0 = load_4(s);
  int64_t h1 = load_3(s + 4) << 6;
  int64_t h2 = load_3(s + 7) << 5;
  int64_t h3 = load_3(s + 10) << 3;
  int64_t h4 = load_3(s + 13) << 2;
  int64_t h5 = load_4(s + 16);
  int64_t h6 = load_3(s + 20) << 7;
  int64_t h7 = load_3(s + 23) << 5;
  int64_t h8 = load_3(s + 26) << 4;
  int64_t h9 = load_3(s + 29) << 2;
  int64_t carry0;
  int64_t carry1;
  int64_t carry2;
  int64_t carry3;
  int64_t carry4;
  int64_t carry5;
  int64_t carry6;
  int64_t carry7;
  int64_t carry8;
  int64_t carry9;

  carry9 = (h9 + (int64_t) (1<<24)) >> 25; h0 += carry9 * 19; h9 -= carry9 << 25;
  carry1 = (h1 + (int64_t) (1<<24)) >> 25; h2 += carry1; h1 -= carry1 << 25;
  carry3 = (h3 + (int64_t) (1<<24)) >> 25; h4 += carry3; h3 -= carry3 << 25;
  carry5 = (h5 + (int64_t) (1<<24)) >> 25; h6 += carry5; h5 -= carry5 << 25;
  carry7 = (h7 + (int64_t) (1<<24)) >> 25; h8 += carry7; h7 -= carry7 << 25;

  carry0 = (h0 + (int64_t) (1<<25)) >> 26; h1 += carry0; h0 -= carry0 << 26;
  carry2 = (h2 + (int64_t) (1<<25)) >> 26; h3 += carry2; h2 -= carry2 << 26;
  carry4 = (h4 + (int64_t) (1<<25)) >> 26; h5 += carry4; h4 -= carry4 << 26;
  carry6 = (h6 + (int64_t) (1<<25)) >> 26; h7 += carry6; h6 -= carry6 << 26;
  carry8 = (h8 + (int64_t) (1<<25)) >> 26; h9 += carry8; h8 -= carry8 << 26;

  u[0] = h0;
  u[1] = h1;
  u[2] = h2;
  u[3] = h3;
  u[4] = h4;
  u[5] = h5;
  u[6] = h6;
  u[7] = h7;
  u[8] = h8;
  u[9] = h9;

  /* End fe_frombytes.c */

  fe_sq2(v, u); /* 2 * u^2 */
  fe_1(w);
  fe_add(w, v, w); /* w = 2 * u^2 + 1 */
  fe_sq(x, w); /* w^2 */
  fe_mul(y, fe_ma2, v); /* -2 * A^2 * u^2 */
  fe_add(x, x, y); /* x = w^2 - 2 * A^2 * u^2 */
  fe_divpowm1(r->X, w, x); /* (w / x)^(m + 1) */
  fe_sq(y, r->X);
  fe_mul(x, y, x);
  fe_sub(y, w, x);
  fe_copy(z, fe_ma);
  if (fe_isnonzero(y)) {
    fe_add(y, w, x);
    if (fe_isnonzero(y)) {
      goto negative;
    } else {
      fe_mul(r->X, r->X, fe_fffb1);
    }
  } else {
    fe_mul(r->X, r->X, fe_fffb2);
  }
  fe_mul(r->X, r->X, u); /* u * sqrt(2 * A * (A + 2) * w / x) */
  fe_mul(z, z, v); /* -2 * A * u^2 */
  sign = 0;
  goto setsign;
negative:
  fe_mul(x, x, fe_sqrtm1);
  fe_sub(y, w, x);
  if (fe_isnonzero(y)) {
    assert((fe_add(y, w, x), !fe_isnonzero(y)));
    fe_mul(r->X, r->X, fe_fffb3);
  } else {
    fe_mul(r->X, r->X, fe_fffb4);
  }
  /* r->X = sqrt(A * (A + 2) * w / x) */
  /* z = -A */
  sign = 1;
setsign:
  if (fe_isnegative(r->X) != sign) {
    assert(fe_isnonzero(r->X));
    fe_neg(r->X, r->X);
  }
  fe_add(r->Z, z, w);
  fe_sub(r->Y, z, w);
  fe_mul(r->X, r->X, r->Z);
#if !defined(NDEBUG)
  {
    fe check_x, check_y, check_iz, check_v;
    fe_invert(check_iz, r->Z);
    fe_mul(check_x, r->X, check_iz);
    fe_mul(check_y, r->Y, check_iz);
    fe_sq(check_x, check_x);
    fe_sq(check_y, check_y);
    fe_mul(check_v, check_x, check_y);
    fe_mul(check_v, fe_d, check_v);
    fe_add(check_v, check_v, check_x);
    fe_sub(check_v, check_v, check_y);
    fe_1(check_x);
    fe_add(check_v, check_v, check_x);
    assert(!fe_isnonzero(check_v));
  }
#endif
}

/* From ge_tobytes.c */

void ge_tobytes(unsigned char *s, const ge_p2 *h) {
  fe recip;
  fe x;
  fe y;

  fe_invert(recip, h->Z);
  fe_mul(x, h->X, recip);
  fe_mul(y, h->Y, recip);
  fe_tobytes(s, y);
  s[31] ^= fe_isnegative(x) << 7;
}

int hash_to_point(const unsigned char *data, unsigned char *result) {
  ge_p2 res;
  ge_fromfe_frombytes_vartime(&res, data);
  ge_tobytes(result, &res);

  return 0;
}
