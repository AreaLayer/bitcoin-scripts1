use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::{AdditiveGroup, BigInteger, Field, PrimeField};
use num_bigint::BigUint;

use crate::bigint::U254;
use crate::bn254::fp254impl::Fp254Impl;
use crate::bn254::fq::Fq;
use crate::bn254::fr::Fr;
use crate::treepp::{script, Script};
use std::cmp::min;
use std::sync::OnceLock;

use super::utils::Hint;

static G1_DOUBLE_PROJECTIVE: OnceLock<Script> = OnceLock::new();
static G1_NONZERO_ADD_PROJECTIVE: OnceLock<Script> = OnceLock::new();

pub struct G1Projective;

impl G1Projective {
    pub fn push_generator() -> Script {
        script! {
            { Fq::push_one() }
            { Fq::push_hex("2") }
            { Fq::push_one() }
        }
    }

    pub fn push_zero() -> Script {
        script! {
            { Fq::push_zero() }
            { Fq::push_zero() }
            { Fq::push_zero() }
        }
    }

    pub fn push(element: ark_bn254::G1Projective) -> Script {
        script! {
            { Fq::push_u32_le(&BigUint::from(element.x).to_u32_digits()) }
            { Fq::push_u32_le(&BigUint::from(element.y).to_u32_digits()) }
            { Fq::push_u32_le(&BigUint::from(element.z).to_u32_digits()) }
        }
    }

    pub fn push_not_montgomery(element: ark_bn254::G1Projective) -> Script {
        script! {
            { Fq::push_u32_le_not_montgomery(&BigUint::from(element.x).to_u32_digits()) }
            { Fq::push_u32_le_not_montgomery(&BigUint::from(element.y).to_u32_digits()) }
            { Fq::push_u32_le_not_montgomery(&BigUint::from(element.z).to_u32_digits()) }
        }
    }

    pub fn is_zero_keep_element(a: u32) -> Script {
        script! {
            // Check if the third coordinate(z) is zero
            { Fq::is_zero_keep_element(a * 3) }
        }
    }

    pub fn nonzero_double() -> Script {
        G1_DOUBLE_PROJECTIVE
            .get_or_init(|| {
                script! {
                    { Fq::copy(2) }
                    { Fq::square() }
                    { Fq::copy(2) }
                    { Fq::square() }
                    { Fq::copy(0) }
                    { Fq::square() }
                    { Fq::add(5, 1) }
                    { Fq::square() }
                    { Fq::copy(1) }
                    { Fq::sub(1, 0) }
                    { Fq::copy(2) }
                    { Fq::sub(1, 0) }
                    { Fq::double(0) }
                    { Fq::copy(2) }
                    { Fq::double(0) }
                    { Fq::add(3, 0) }
                    { Fq::copy(0) }
                    { Fq::square() }
                    { Fq::copy(2) }
                    { Fq::double(0) }
                    { Fq::sub(1, 0) }
                    { Fq::copy(0) }
                    { Fq::sub(3, 0) }
                    { Fq::roll(2) }
                    { Fq::mul() }
                    { Fq::double(2) }
                    { Fq::double(0) }
                    { Fq::double(0) }
                    { Fq::sub(1, 0) }
                    { Fq::roll(2) }
                    { Fq::roll(3) }
                    { Fq::mul() }
                    { Fq::double(0) }
                }
            })
            .clone()
    }

    pub fn hinted_nonzero_double(a: ark_bn254::G1Projective) -> (Script, Vec<Hint>) {
        let mut hints = Vec::new();

        let (hinted_script1, hint1) = Fq::hinted_square(a.x);
        let (hinted_script2, hint2) = Fq::hinted_square(a.y);
        let (hinted_script3, hint3) = Fq::hinted_square(a.y.square());
        let (hinted_script4, hint4) = Fq::hinted_square(a.x + a.y.square());
        let three_x2 = a.x.square().double() + a.x.square();
        let (hinted_script5, hint5) = Fq::hinted_square(three_x2);
        let xy2 = a.x * a.y.square();
        let twelve_xy2 = xy2.double().double().double() + xy2.double().double();
        let nine_x4 = a.x.square().square().double().double().double() + a.x.square().square();
        let (hinted_script6, hint6) = Fq::hinted_mul(1, twelve_xy2 - nine_x4, 0, three_x2);
        let (hinted_script7, hint7) = Fq::hinted_mul(1, a.y, 0, a.z);

        let script_lines = vec! [
            // x, y, z
            Fq::copy(2),
            // x, y, z, x
            hinted_script1,
            // x, y, z, x^2
            Fq::copy(2),
            // x, y, z, x^2, y
            hinted_script2,
            // x, y, z, x^2, y^2
            Fq::copy(0),
            // x, y, z, x^2, y^2, y^2
            hinted_script3,
            // x, y, z, x^2, y^2, y^4
            Fq::add(5, 1),
            // y, z, x^2, y^4, x+y^2
            hinted_script4,
            // y, z, x^2, y^4, (x+y^2)^2
            Fq::copy(1),
            // y, z, x^2, y^4, (x+y^2)^2, y^4
            Fq::sub(1, 0),
            // y, z, x^2, y^4, x^2+2xy^2
            Fq::copy(2),
            // y, z, x^2, y^4, x^2+2xy^2, x^2
            Fq::sub(1, 0),
            // y, z, x^2, y^4, 2xy^2
            Fq::double(0),
            // y, z, x^2, y^4, 4xy^2
            Fq::copy(2),
            // y, z, x^2, y^4, 4xy^2, x^2
            Fq::double(0),
            // y, z, x^2, y^4, 4xy^2, 2x^2
            Fq::add(3, 0),
            // y, z, y^4, 4xy^2, 3x^2
            Fq::copy(0),
            // y, z, y^4, 4xy^2, 3x^2, 3x^2
            hinted_script5,
            // y, z, y^4, 4xy^2, 3x^2, 9x^4
            Fq::copy(2),
            // y, z, y^4, 4xy^2, 3x^2, 9x^4, 4xy^2
            Fq::double(0),
            // y, z, y^4, 4xy^2, 3x^2, 9x^4, 8xy^2
            Fq::sub(1, 0),
            // y, z, y^4, 4xy^2, 3x^2, 9x^4-8xy^2
            Fq::copy(0),
            // y, z, y^4, 4xy^2, 3x^2, 9x^4-8xy^2, 9x^4-8xy^2
            Fq::sub(3, 0),
            // y, z, y^4, 3x^2, 9x^4-8xy^2, 12xy^2-9x^4
            Fq::roll(2),
            // y, z, y^4, 9x^4-8xy^2, 12xy^2-9x^4, 3x^2
            hinted_script6,
            // y, z, y^4, 9x^4-8xy^2, 3x^2(12xy^2-9x^4)
            Fq::double(2),
            // y, z, 9x^4-8xy^2, 3x^2(12xy^2-9x^4), 2y^4
            Fq::double(0),
            Fq::double(0),
            // y, z, 9x^4-8xy^2, 3x^2(12xy^2-9x^4), 8y^4
            Fq::sub(1, 0),
            // y, z, 9x^4-8xy^2, 3x^2(12xy^2-9x^4)-8y^4
            Fq::roll(2),
            // y, 9x^4-8xy^2, 3x^2(12xy^2-9x^4)-8y^4, z
            Fq::roll(3),
            // 9x^4-8xy^2, 3x^2(12xy^2-9x^4)-8y^4, z, y
            hinted_script7,
            // 9x^4-8xy^2, 3x^2(12xy^2-9x^4)-8y^4, yz
            Fq::double(0),
            // 9x^4-8xy^2, 3x^2(12xy^2-9x^4)-8y^4, 2yz
        ];
        let mut script = script!{};
        for script_line in script_lines {
            script = script.push_script(script_line.compile());
        }

        hints.extend(hint1);
        hints.extend(hint2);
        hints.extend(hint3);
        hints.extend(hint4);
        hints.extend(hint5);
        hints.extend(hint6);
        hints.extend(hint7);

        (script, hints)
    }

    pub fn double() -> Script {
        script! {
            // Check if the first point is zero
            { G1Projective::is_zero_keep_element(0) }
            OP_NOTIF
                // Perform a regular addition
                { G1Projective::nonzero_double() }
            OP_ENDIF
        }
    }

    pub fn hinted_double(a: ark_bn254::G1Projective) -> (Script, Vec<Hint>) {
        let mut hints = Vec::new();
        let (hinted_nonzero_double, hint1) = G1Projective::hinted_nonzero_double(a);

        let script_lines = vec! [
            // Check if the first point is zero
            G1Projective::is_zero_keep_element(0),
            script!{OP_NOTIF},
                hinted_nonzero_double,
            script!{OP_ENDIF}
        ];
        let mut script = script!{};
        for script_line in script_lines {
            script = script.push_script(script_line.compile());
        }
        if !a.into_affine().is_zero() {
            hints.extend(hint1);
        }

        (script, hints)
    }

    pub fn nonzero_add() -> Script {
        G1_NONZERO_ADD_PROJECTIVE
            .get_or_init(|| {
                script! {
                    { Fq::copy(3) }
                    { Fq::square() }
                    { Fq::copy(1) }
                    { Fq::square() }
                    { Fq::roll(7) }
                    { Fq::copy(1) }
                    { Fq::mul() }
                    { Fq::roll(5) }
                    { Fq::copy(3) }
                    { Fq::mul() }
                    { Fq::copy(2) }
                    { Fq::roll(8) }
                    { Fq::mul() }
                    { Fq::copy(5) }
                    { Fq::mul() }
                    { Fq::copy(4) }
                    { Fq::roll(7) }
                    { Fq::mul() }
                    { Fq::copy(7) }
                    { Fq::mul() }
                    { Fq::add(7, 6)}
                    { Fq::copy(4) }
                    { Fq::sub(4, 0)}
                    { Fq::copy(0) }
                    { Fq::double(0) }
                    { Fq::square() }
                    { Fq::copy(1) }
                    { Fq::copy(1) }
                    { Fq::mul() }
                    { Fq::copy(5) }
                    { Fq::sub(5, 0) }
                    { Fq::double(0) }
                    { Fq::roll(6) }
                    { Fq::roll(3) }
                    { Fq::mul() }
                    { Fq::copy(1) }
                    { Fq::square() }
                    { Fq::copy(3) }
                    { Fq::sub(1, 0) }
                    { Fq::copy(1) }
                    { Fq::double(0) }
                    { Fq::sub(1, 0) }
                    { Fq::copy(0) }
                    { Fq::sub(2, 0) }
                    { Fq::roll(2) }
                    { Fq::mul() }
                    { Fq::roll(5) }
                    { Fq::roll(3) }
                    { Fq::mul() }
                    { Fq::double(0) }
                    { Fq::sub(1, 0) }
                    { Fq::roll(3) }
                    { Fq::square() }
                    { Fq::sub(0, 5) }
                    { Fq::sub(0, 4) }
                    { Fq::roll(3) }
                    { Fq::mul() }
                }
            })
            .clone()
    }

    pub fn hinted_nonzero_add(a: ark_bn254::G1Projective, b: ark_bn254::G1Projective) -> (Script, Vec<Hint>) {
        let mut hints = Vec::new();
        let var1 = a.z.square();
        let var2 = b.z.square();
        let var3 = a.x * var2;
        let var4 = b.x * var1;
        let var5= a.y * var2;
        let var6 = b.y * var1;
        let var7 = b.z * var5;
        let var8 = a.z * var6;
        let var9 = var4 - var3;
        let var10 = (var9 + var9).square();
        let var11 = var10 * var9;
        let var12 =  var8-var7 + var8-var7;
        let var13 = var10 * var3;
        let var14 = var12.square();
        let var15 = var13+var13+var13-var14+var11;
        // let var16 = var15 * var12;
        // let var17 = var7 * var11;

        let (hinted_script1, hint1) = Fq::hinted_square(a.z);
        let (hinted_script2, hint2) = Fq::hinted_square(b.z);
        let (hinted_script3, hint3) = Fq::hinted_mul(1, a.x, 0, var2);
        let (hinted_script4, hint4) = Fq::hinted_mul(1, b.x, 0, var1);
        let (hinted_script5, hint5) = Fq::hinted_mul(1, var2, 0, a.y);
        let (hinted_script6, hint6) = Fq::hinted_mul(1, var5, 0, b.z);
        let (hinted_script7, hint7) = Fq::hinted_mul(1, var1, 0, b.y);
        let (hinted_script8, hint8) = Fq::hinted_mul(1, var6, 0, a.z);
        let (hinted_script9, hint9) = Fq::hinted_square(var9 + var9);
        let (hinted_script10, hint10) = Fq::hinted_mul(1, var10, 0, var9);
        let (hinted_script11, hint11) = Fq::hinted_mul(1, var3, 0, var10);
        let (hinted_script12, hint12) = Fq::hinted_square(var12);
        let (hinted_script13, hint13) = Fq::hinted_mul(1, var15, 0, var12);
        let (hinted_script14, hint14) = Fq::hinted_mul(1, var7, 0, var11);
        let (hinted_script15, hint15) = Fq::hinted_square(a.z + b.z);
        let (hinted_script16, hint16) = Fq::hinted_mul(1, (a.z+b.z).square()-var1-var2, 0, var9);

        let script_lines = vec! [
            // ax ay az bx by bz
            Fq::copy(3),
            hinted_script1,
            // ax ay az bx by bz var1
            Fq::copy(1),
            hinted_script2,
            // ax ay az bx by bz var1 var2
            Fq::roll(7),
            Fq::copy(1),
            hinted_script3,
            // ay az bx by bz var1 var2 var3
            Fq::roll(5),
            Fq::copy(3),
            hinted_script4,
            // ay az by bz var1 var2 var3 var4
            Fq::copy(2),
            Fq::roll(8),
            hinted_script5,
            // az by bz var1 var2 var3 var4 var5
            Fq::copy(5),
            hinted_script6,
            // az by bz var1 var2 var3 var4 var7 
            Fq::copy(4),
            Fq::roll(7),
            hinted_script7,
            // az bz var1 var2 var3 var4 var7 var6
            Fq::copy(7),
            hinted_script8,
            // az bz var1 var2 var3 var4 var7 var8
            Fq::add(7, 6),
            // var1 var2 var3 var4 var7 var8 az+bz
            Fq::copy(4),
            // var1 var2 var3 var4 var7 var8 az+bz var3
            Fq::sub(4, 0),
            // var1 var2 var3 var7 var8 az+bz var9
            Fq::copy(0),
            // var1 var2 var3 var7 var8 az+bz var9 var9
            Fq::double(0),
            // var1 var2 var3 var7 var8 az+bz var9 2*var9
            hinted_script9,
            // var1 var2 var3 var7 var8 az+bz var9 var10
            Fq::copy(1),
            Fq::copy(1),
            hinted_script10,
            // var1 var2 var3 var7 var8 az+bz var9 var10 var11
            Fq::copy(5),
            Fq::sub(5, 0),
            // var1 var2 var3 var7 az+bz var9 var10 var11 var8-var7 
            Fq::double(0),
            // var1 var2 var3 var7 az+bz var9 var10 var11 var12 
            Fq::roll(6),
            // var1 var2 var7 az+bz var9 var10 var11 var12 var3
            Fq::roll(3),
            // var1 var2 var7 az+bz var9 var11 var12 var3 var10
            hinted_script11,
            // var1 var2 var7 az+bz var9 var11 var12 var13
            Fq::copy(1),
            hinted_script12,
            // var1 var2 var7 az+bz var9 var11 var12 var13 var14
            Fq::copy(3),
            // var1 var2 var7 az+bz var9 var11 var12 var13 var14 var11
            Fq::sub(1, 0),
            // var1 var2 var7 az+bz var9 var11 var12 var13 var14-var11
            Fq::copy(1),
            // var1 var2 var7 az+bz var9 var11 var12 var13 var14-var11 var13
            Fq::double(0),
            // var1 var2 var7 az+bz var9 var11 var12 var13 var14-var11 2*var13
            Fq::sub(1, 0),
            // var1 var2 var7 az+bz var9 var11 var12 var13 var14-var11-2*var13
            Fq::copy(0),
            // var1 var2 var7 az+bz var9 var11 var12 var13 var14-var11-2*var13 var14-var11-2*var13
            Fq::sub(2, 0),
            // var1 var2 var7 az+bz var9 var11 var12 var14-var11-2*var13 var15
            Fq::roll(2),
            // var1 var2 var7 az+bz var9 var11 var14-var11-2*var13 var15 var12
            hinted_script13,
            // var1 var2 var7 az+bz var9 var11 var14-var11-2*var13 var16
            Fq::roll(5),
            // var1 var2 var2 z+bz var9 var11 var14-var11-2*var13 var16 var7
            Fq::roll(3),
            // var1 var2 az+bz var9 var14-var11-2*var13 var16 var7 var11
            hinted_script14,
            // var1 var2 az+bz var9 var14-var11-2*var13 var16 var17
            Fq::double(0),
            // var1 var2 az+bz var9 var14-var11-2*var13 var16 2*var17
            Fq::sub(1, 0),
            // var1 var2 az+bz var9 var14-var11-2*var13 var16-2*var17
            Fq::roll(3),
            // var1 var2 var9 var14-var11-2*var13 var16-2*var17 az+bz
            hinted_script15,
            // var1 var2 var9 var14-var11-2*var13 var16-2*var17 (az+bz)^2
            Fq::sub(0, 5),
            // var2 var9 var14-var11-2*var13 var16-2*var17 (az+bz)^2-var1
            Fq::sub(0, 4),
            // var9 var14-var11-2*var13 var16-2*var17 (az+bz)^2-var1-var2
            Fq::roll(3),
            // var14-var11-2*var13 var16-2*var17 (az+bz)^2-var1-var2 var9
            hinted_script16,
            // var14-var11-2*var13 var16-2*var17 ((az+bz)^2-var1-var2)*var9
        ];
        let mut script = script!{};
        for script_line in script_lines {
            script = script.push_script(script_line.compile());
        }

        hints.extend(hint1);
        hints.extend(hint2);
        hints.extend(hint3);
        hints.extend(hint4);
        hints.extend(hint5);
        hints.extend(hint6);
        hints.extend(hint7);
        hints.extend(hint8);
        hints.extend(hint9);
        hints.extend(hint10);
        hints.extend(hint11);
        hints.extend(hint12);
        hints.extend(hint13);
        hints.extend(hint14);
        hints.extend(hint15);
        hints.extend(hint16);

        (script, hints)
    }

    pub fn add() -> Script {
        script! {
            // Check if the first point is zero
            { G1Projective::is_zero_keep_element(0) }
            OP_IF
                // First point is zero
                { G1Projective::drop() }
            OP_ELSE
                // Check if the second point is zero
                { G1Projective::is_zero_keep_element(1) }
                OP_IF
                    // Second point is zero
                    { G1Projective::roll(1) }
                    { G1Projective::drop() }
                OP_ELSE
                    // Both summands are non-zero
                    { G1Projective::nonzero_add() }
                OP_ENDIF
            OP_ENDIF
        }
    }

    pub fn hinted_add(a: ark_bn254::G1Projective, b: ark_bn254::G1Projective) -> (Script, Vec<Hint>) {
        let mut hints = Vec::new();
        let (hinted_script1, hint1) = G1Projective::hinted_nonzero_add(a, b);

        let script_lines = vec! [
            // Check if the first point is zero
            G1Projective::is_zero_keep_element(0),
            script!{OP_IF},
                // First point is zero
                G1Projective::drop(),
            script!{OP_ELSE},
                // Check if the second point is zero
                G1Projective::is_zero_keep_element(1),
                script!{OP_IF},
                    // Second point is zero
                    G1Projective::roll(1),
                    G1Projective::drop(),
                script!{OP_ELSE},
                    hinted_script1,
                script!{OP_ENDIF},
            script!{OP_ENDIF},
        ];
        let mut script = script!{};
        for script_line in script_lines {
            script = script.push_script(script_line.compile());
        }
        if !a.into_affine().is_zero() && !b.into_affine().is_zero() {
            hints.extend(hint1);
        }

        (script, hints)
    }

    pub fn neg() -> Script {
        script! {
            { Fq::neg(1) }
            { Fq::roll(1) }
        }
    }

    pub fn copy(mut a: u32) -> Script {
        a *= 3;
        script! {
            { Fq::copy(a + 2) }
            { Fq::copy(a + 2) }
            { Fq::copy(a + 2) }
        }
    }

    pub fn roll(mut a: u32) -> Script {
        a *= 3;
        script! {
            { Fq::roll(a + 2) }
            { Fq::roll(a + 2) }
            { Fq::roll(a + 2) }
        }
    }

    pub fn equalverify() -> Script {
        script! {
            { Fq::copy(3) }
            { Fq::square() }
            { Fq::roll(4) }
            { Fq::copy(1) }
            { Fq::mul() }

            { Fq::copy(2) }
            { Fq::square() }
            { Fq::roll(3) }
            { Fq::copy(1) }
            { Fq::mul() }

            { Fq::roll(7) }
            { Fq::roll(2) }
            { Fq::mul() }
            { Fq::roll(5) }
            { Fq::roll(4) }
            { Fq::mul() }
            { Fq::equalverify(1, 0) }

            { Fq::roll(3) }
            { Fq::roll(1) }
            { Fq::mul() }
            { Fq::roll(2) }
            { Fq::roll(2) }
            { Fq::mul() }
            { Fq::equalverify(1, 0) }
        }
    }

    pub fn hinted_equalverify(a: ark_bn254::G1Projective, b: ark_bn254::G1Projective) -> (Script, Vec<Hint>) {
        let mut hints = Vec::new();

        let (hinted_script1, hint1) = Fq::hinted_square(a.z);
        let (hinted_script2, hint2) = Fq::hinted_mul(1, a.z, 0, a.z.square());
        let (hinted_script3, hint3) = Fq::hinted_square(b.z);
        let (hinted_script4, hint4) = Fq::hinted_mul(1, b.z, 0, b.z.square());
        let (hinted_script5, hint5) = Fq::hinted_mul(1, a.x, 0, b.z.square());
        let (hinted_script6, hint6) = Fq::hinted_mul(1, b.x, 0, a.z.square());
        let (hinted_script7, hint7) = Fq::hinted_mul(1, a.y, 0, b.z.square()*b.z);
        let (hinted_script8, hint8) = Fq::hinted_mul(1, b.y, 0, a.z.square()*a.z);
        
        let script_lines = vec! [
            Fq::copy(3),
            hinted_script1,
            Fq::roll(4),
            Fq::copy(1),
            hinted_script2,

            Fq::copy(2),
            hinted_script3,
            Fq::roll(3),
            Fq::copy(1),
            hinted_script4,

            Fq::roll(7),
            Fq::roll(2),
            hinted_script5,
            Fq::roll(5),
            Fq::roll(4),
            hinted_script6,
            Fq::equalverify(1, 0),

            Fq::roll(3),
            Fq::roll(1),
            hinted_script7,
            Fq::roll(2),
            Fq::roll(2),
            hinted_script8,
            Fq::equalverify(1, 0),
        ];
        let mut script = script!{};
        for script_line in script_lines {
            script = script.push_script(script_line.compile());
        }
        hints.extend(hint1);
        hints.extend(hint2);
        hints.extend(hint3);
        hints.extend(hint4);
        hints.extend(hint5);
        hints.extend(hint6);
        hints.extend(hint7);
        hints.extend(hint8);

        (script, hints)
    }

    pub fn drop() -> Script {
        script! {
            { Fq::drop() }
            { Fq::drop() }
            { Fq::drop() }
        }
    }

    pub fn toaltstack() -> Script {
        script! {
            { Fq::toaltstack() }
            { Fq::toaltstack() }
            { Fq::toaltstack() }
        }
    }

    pub fn fromaltstack() -> Script {
        script! {
            { Fq::fromaltstack() }
            { Fq::fromaltstack() }
            { Fq::fromaltstack() }
        }
    }

    // Input Stack: [x, y, z]
    // Output Stack: [x/z^2, y/z^3]
    pub fn into_affine() -> Script {
        script!(
            // Copy input x and y to altstack
            { Fq::copy(1) }
            { Fq::toaltstack() }
            { Fq::copy(2) }
            { Fq::toaltstack() }

            // 1. Check if the first point is zero
            { G1Projective::is_zero_keep_element(0) }
            OP_TOALTSTACK

            // 2. Otherwise, check if the point.z is one
            { Fq::is_one_keep_element(0) }
            OP_TOALTSTACK

            // Run normal calculation anyway
            // 2.2 Otherwise, Z is non-one, so it must have an inverse in a field.
            // conpute Z^-1
            { Fq::inv() }

            // compute Z^-2
            { Fq::copy(0) }
            { Fq::square() }
            // compute Z^-3 = Z^-2 * z^-1
            { Fq::copy(0) }
            { Fq::roll(2) }
            { Fq::mul() }

            // For now, stack: [x, y, z^-2, z^-3]

            // compute Y/Z^3 = Y * Z^-3
            { Fq::roll(2) }
            { Fq::mul() }

            // compute X/Z^2 = X * Z^-2
            { Fq::roll(1) }
            { Fq::roll(2) }
            { Fq::mul() }

            // Return (x,y)
            { Fq::roll(1) }

            // Select the result
            OP_FROMALTSTACK
            OP_FROMALTSTACK
            OP_IF
                // Z is zero so drop the calculated affine point and return the affine::identity
                OP_DROP
                { Fq::drop() }
                { Fq::drop() }
                { Fq::fromaltstack() }
                { Fq::fromaltstack() }
                { Fq::drop() }
                { Fq::drop() }
                { G1Affine::identity() }
            OP_ELSE
                OP_IF
                    // Z was one so drop the the calculated result and return the original input
                    // If Z is one, the point is already normalized, so that: projective.x = affine.x, projective.y = affine.y
                    { Fq::drop() }
                    { Fq::drop() }
                    { Fq::fromaltstack() }
                    { Fq::fromaltstack() }
                OP_ELSE
                    { Fq::fromaltstack() }
                    { Fq::fromaltstack() }
                    { Fq::drop() }
                    { Fq::drop() }
                OP_ENDIF
            OP_ENDIF
        )
    }

    // Input Stack: [x, y, z]
    // Output Stack: [x/z^2, y/z^3]
    pub fn hinted_into_affine(a: ark_bn254::G1Projective) -> (Script, Vec<Hint>) {
        let mut hints = Vec::new();
        
        let (
            (hinted_script1, hint1),
            (hinted_script2, hint2),
            (hinted_script3, hint3),
            (hinted_script4, hint4),
            (hinted_script5, hint5),
        ) = if a.z != ark_bn254::Fq::ONE && a.z != ark_bn254::Fq::ZERO {
            let (hinted_script1, hint1) = Fq::hinted_inv(a.z);
            let z_inv = a.z.inverse().unwrap();
            let (hinted_script2, hint2) = Fq::hinted_square(z_inv);
            let (hinted_script3, hint3) = Fq::hinted_mul(1, z_inv.square(), 0, z_inv);
            let (hinted_script4, hint4) = Fq::hinted_mul(1, a.y, 0, z_inv.square() * z_inv);
            let (hinted_script5, hint5) = Fq::hinted_mul(1, a.x, 0, z_inv.square());
            (
                (hinted_script1, hint1),
                (hinted_script2, hint2),
                (hinted_script3, hint3),
                (hinted_script4, hint4),
                (hinted_script5, hint5),
            )
        }
        else {
            let (hinted_script1, hint1) = (script! {}, vec![]);
            let (hinted_script2, hint2) = (script! {}, vec![]);
            let (hinted_script3, hint3) = (script! {}, vec![]);
            let (hinted_script4, hint4) = (script! {}, vec![]);
            let (hinted_script5, hint5) = (script! {}, vec![]);
            (
                (hinted_script1, hint1),
                (hinted_script2, hint2),
                (hinted_script3, hint3),
                (hinted_script4, hint4),
                (hinted_script5, hint5),
            )
        };

        let mut script = script! {};
        let script_lines = [
            // // Copy input x and y to altstack
            // Fq::copy(1),
            // Fq::toaltstack(),
            // Fq::copy(2),
            // Fq::toaltstack(),

            // 1. Check if the first point is zero
            G1Projective::is_zero_keep_element(0),

            script! {OP_IF},
                // Z is zero so drop the point and return the affine::identity
                Fq::drop(),
                Fq::drop(),
                Fq::drop(),
                G1Affine::identity(),
            script! {OP_ELSE},
                Fq::is_one_keep_element_not_montgomery(0),
                script! {OP_IF},
                    Fq::drop(),
                script! {OP_ELSE},
                    // 2.2 Otherwise, Z is non-one, so it must have an inverse in a field.
                    // conpute Z^-1
                    hinted_script1,

                    // compute Z^-2
                    Fq::copy(0),
                    hinted_script2,
                    // compute Z^-3 = Z^-2 * z^-1
                    Fq::copy(0),
                    Fq::roll(2),
                    hinted_script3,

                    // For now, stack: [x, y, z^-2, z^-3]

                    // compute Y/Z^3 = Y * Z^-3
                    Fq::roll(2),
                    hinted_script4,

                    // compute X/Z^2 = X * Z^-2
                    Fq::roll(1),
                    Fq::roll(2),
                    hinted_script5,

                    // Return (x,y)
                    Fq::roll(1),
                script! {OP_ENDIF},
            script! {OP_ENDIF},
        ];

        for script_line in script_lines {
            script = script.push_script(script_line.compile());
        }

        if a.z != ark_bn254::Fq::ONE && a.z != ark_bn254::Fq::ZERO {
            hints.extend(hint1);
            hints.extend(hint2);
            hints.extend(hint3);
            hints.extend(hint4);
            hints.extend(hint5);
        }
        
        (script, hints)
    }

    /// Convert a number to digits
    fn to_digits_helper<const DIGIT_COUNT: usize>(mut number: u32) -> [u8; DIGIT_COUNT] {
        let mut digits: [u8; DIGIT_COUNT] = [0; DIGIT_COUNT];
        for i in 0..DIGIT_COUNT {
            let digit = number % 2;
            number = (number - digit) / 2;
            digits[i] = digit as u8;
        }
        digits
    }

    /// input stack: point_0, scalar_0, ..., point_{TERMS-1}, scalar_{TERMS-1}
    /// output stack: sum of scalar_i * point_i for 0..TERMS
    /// comments: pi -> point_i, si -> scalar_i
    pub fn batched_scalar_mul<const TERMS: usize>() -> Script {
        // comments for 2
        // point_0 scalar_0 point_1 scalar_1
        let script = script! {
            // convert scalars to bit-style
            for i in 0..1 {
                { Fq::roll(4*(TERMS - i - 1) as u32) }

                { Fr::decode_montgomery() }
                { Fr::convert_to_le_bits_toaltstack() }
            }

            for term in 1..TERMS {
                { Fq::roll(4*(TERMS - term - 1) as u32) }

                { Fr::decode_montgomery() }
                { Fr::convert_to_le_bits_toaltstack() }

                for _ in 0..2*Fr::N_BITS {
                    OP_FROMALTSTACK
                }

                // zip scalars
                // [p0, p1, s1_0, s1_1, s1_2, ..., s0_0, s0_1, s0_2, ...]
                for i in 0..Fr::N_BITS {
                    { Fr::N_BITS - i } OP_ROLL
                    for _ in 0..term {OP_DUP OP_ADD} OP_ADD //  s0_0 + s1_0*2
                    OP_TOALTSTACK
                }
            }

            // get some bases (2^TERMS bases) [p0, p1]
            // ouptut: [p1+p0, p1, p0, 0]
            { G1Projective::push_zero() }
            { G1Projective::toaltstack() }

            for i in 1..(u32::pow(2, TERMS as u32)) {
                {G1Projective::push_zero()}
                for (j, mark) in Self::to_digits_helper::<TERMS>(i).iter().enumerate() {
                    if *mark == 1 {
                        { G1Projective::copy(TERMS as u32 - j as u32) }
                        { G1Projective::add() }
                    }
                }
                { G1Projective::toaltstack() }
            }

            for _ in 0..TERMS {
                { G1Projective::drop() }
            }

            for _ in 0..(u32::pow(2, TERMS as u32)) {
                { G1Projective::fromaltstack() }
            }

            { G1Projective::push_zero() } // target
            // [p1+p0, p1, p0, 0, target]
            // for i in 0..Fr::N_BITS {
            for i in 0..Fr::N_BITS {
                OP_FROMALTSTACK // idx = s1_0*2 + s0_0
                OP_1 OP_ADD // idx + 1

                // simulate {G1Projective::pick()}
                for _ in 0..26 { OP_DUP }
                for _ in 0..26 { OP_ADD }
                { 26 } OP_ADD // [p1+p0, p1, p0, 0, target, 27*(idx+1)+26]
                for _ in 0..26 { OP_DUP }
                for _ in 0..26 { OP_TOALTSTACK }
                { script!{ OP_PICK }.add_stack_hint(-((27 * 2^TERMS + 26) as i32), 1) }
                for _ in 0..26 {
                    OP_FROMALTSTACK
                    { script!{ OP_PICK }.add_stack_hint(-((27 * 2^TERMS + 26) as i32), 1)} }

                { G1Projective::add() }
                // jump the last one
                if i != Fr::N_BITS-1 {
                    { G1Projective::double() }
                }
            }

            // clear stack
            { G1Projective::toaltstack() }
            for _ in 0..u32::pow(2, TERMS as u32) {
                { G1Projective::drop() }
            }

            { G1Projective::fromaltstack() }
        };
        script
    }

    fn dfs(index: u32, depth: u32,  mask: u32, offset: u32) -> Script {
        if depth == 0 {
            return script!{
                OP_IF
                    { G1Projective::copy(offset - (mask + (1<<index))) }
                OP_ELSE
                    if mask == 0 {
                        OP_FROMALTSTACK
                        OP_NOT
                        OP_TOALTSTACK
                    } else {
                        { G1Projective::copy(offset - mask) }
                    }
                OP_ENDIF
            };
        }
        script!{
            OP_IF 
                { G1Projective::dfs(index+1, depth-1, mask + (1<<index), offset) }
            OP_ELSE
                { G1Projective::dfs(index+1, depth-1, mask, offset) }
            OP_ENDIF
        }
    }

    // [g1projective, scalar]
    pub fn scalar_mul() -> Script {
        let mut loop_scripts = Vec::new();
        let mut i = 0;
        // options: i_step = 2, 3, 4
        let i_step = 4;

        while i < Fr::N_BITS { 
            let depth = min(Fr::N_BITS - i, i_step);

            if i > 0 {
                let double_loop = script! {
                    for _ in 0..depth {
                        { G1Projective::double() }
                    }
                };
                loop_scripts.push(double_loop.clone());
            }

            loop_scripts.push(script!{
                for _ in 0..depth {
                    OP_FROMALTSTACK
                }
            });

            let add_loop = script! {
                OP_TRUE
                OP_TOALTSTACK
                { G1Projective::dfs(0, depth - 1, 0, 1<<i_step) }
                OP_FROMALTSTACK

                OP_IF
                    { G1Projective::add() }
                OP_ENDIF
            };
            loop_scripts.push(add_loop.clone());
            i += i_step;
        }

        script! {
            { Fr::decode_montgomery() }
            { Fr::convert_to_le_bits_toaltstack() }

            { G1Projective::copy(0) }
            { G1Projective::double() }
            for i in 3..(1<<i_step) { 
                { G1Projective::copy(0) }
                { G1Projective::copy(i - 1) }
                { G1Projective::add() }
            }

            { G1Projective::push_zero() }

            for script in loop_scripts {
                { script }
            }

            { G1Projective::toaltstack() }
            for _ in 1..(1<<i_step) {
                { G1Projective::drop() }
            }
            { G1Projective::fromaltstack() }
        }
    }

    fn dfs_with_constant_mul(index: u32, depth: u32,  mask: u32, p_mul: &Vec<ark_bn254::G1Projective>) -> Script {
        if depth == 0 {
            return script!{
                OP_IF
                    { G1Projective::push(p_mul[(mask + (1<<index)) as usize]) }
                OP_ELSE
                    if mask == 0 {
                        OP_FROMALTSTACK
                        OP_NOT
                        OP_TOALTSTACK
                    } else {
                        { G1Projective::push(p_mul[mask as usize]) }
                    }
                OP_ENDIF
            };
        }

        script!{
            OP_IF 
                { G1Projective::dfs_with_constant_mul(index+1, depth-1, mask + (1<<index), p_mul) }
            OP_ELSE
                { G1Projective::dfs_with_constant_mul(index+1, depth-1, mask, p_mul) }
            OP_ENDIF
        }
    }

    fn dfs_with_constant_mul_not_montgomery(index: u32, depth: u32,  mask: u32, p_mul: &Vec<ark_bn254::G1Projective>) -> Script {
        if depth == 0 {
            return script!{
                OP_IF
                    { G1Projective::push_not_montgomery(p_mul[(mask + (1<<index)) as usize]) }
                OP_ELSE
                    if mask == 0 {
                        OP_FROMALTSTACK
                        OP_NOT
                        OP_TOALTSTACK
                    } else {
                        { G1Projective::push_not_montgomery(p_mul[mask as usize]) }
                    }
                OP_ENDIF
            };
        }

        script!{
            OP_IF 
                { G1Projective::dfs_with_constant_mul_not_montgomery(index+1, depth-1, mask + (1<<index), p_mul) }
            OP_ELSE
                { G1Projective::dfs_with_constant_mul_not_montgomery(index+1, depth-1, mask, p_mul) }
            OP_ENDIF
        }
    }

    // [g1projective]
    pub fn scalar_mul_by_constant_g1(p: ark_bn254::G1Projective) -> Script {
        let mut loop_scripts = Vec::new();
        let mut i = 0;
        // options: i_step = 2-15
        let i_step = 12;

        let mut p_mul: Vec<ark_ec::short_weierstrass::Projective<ark_bn254::g1::Config>> = Vec::new();
        p_mul.push(ark_bn254::G1Projective::ZERO);
        p_mul.push(p);
        for _ in 0..(1<<i_step) { 
            p_mul.push(p_mul.last().unwrap() + p);
        }

        while i < Fr::N_BITS { 
            let depth = min(Fr::N_BITS - i, i_step);

            if i > 0 {
                let double_loop = script! {
                    for _ in 0..depth {
                        { G1Projective::double() }
                    }
                };
                loop_scripts.push(double_loop.clone());
            }

            loop_scripts.push(script!{
                for _ in 0..depth {
                    OP_FROMALTSTACK
                }
            });

            let add_loop = script! {
                OP_TRUE
                OP_TOALTSTACK
                { G1Projective::dfs_with_constant_mul(0, depth - 1, 0, &p_mul) }
                OP_FROMALTSTACK

                OP_IF
                    { G1Projective::add() }
                OP_ENDIF
            };
            loop_scripts.push(add_loop.clone());
            i += i_step;
        }

        script! {
            { Fr::decode_montgomery() }
            { Fr::convert_to_le_bits_toaltstack() }

            { G1Projective::push_zero() }

            for script in loop_scripts {
                { script }
            }
        }
    }

     // [g1projective]
     pub fn hinted_scalar_mul_by_constant_g1(scalar: ark_bn254::Fr,p: &mut ark_bn254::G1Projective) -> (Script, Vec<Hint>) {
        let (mut loop_scripts, mut hints) = (Vec::new(), Vec::new());
        let mut i = 0;
        // options: i_step = 2-15
        let i_step = 12;

        let mut p_mul: Vec<ark_ec::short_weierstrass::Projective<ark_bn254::g1::Config>> = Vec::new();
        p_mul.push(ark_bn254::G1Projective::ZERO);
        for _ in 0..(1<<i_step) { 
            p_mul.push(p_mul.last().unwrap() + p.clone());
        }

        let mut c: ark_bn254::G1Projective = ark_bn254::G1Projective::ZERO; 
        let scalar_bigint = scalar.into_bigint();
        while i < Fr::N_BITS { 
            let depth = min(Fr::N_BITS - i, i_step);

            if i > 0 {
                for _ in 0..depth {
                    let (double_script, double_hints) = G1Projective::hinted_double(c);
                    loop_scripts.push(double_script);
                    hints.extend(double_hints);
                    c = c.double();
                }
            }

            loop_scripts.push(script!{
                for _ in 0..depth {
                    OP_FROMALTSTACK
                }
            });

            let mut mask = 0;
            
            for j in 0..depth {
                mask *= 2;
                mask += scalar_bigint.get_bit((Fr::N_BITS - i - j - 1) as usize) as u32;
            }
            let (add_script, add_hints) = G1Projective::hinted_add(c, p_mul[mask as usize]);
            let add_loop = script! {
                OP_TRUE
                OP_TOALTSTACK
                { G1Projective::dfs_with_constant_mul_not_montgomery(0, depth - 1, 0, &p_mul) }
                OP_FROMALTSTACK

                OP_IF
                    { add_script }
                OP_ENDIF
            };
            loop_scripts.push(add_loop.clone());
            if mask != 0 {
                hints.extend(add_hints);
                    c = c + p_mul[mask as usize];
            }
            i += i_step;            
        }
        *p = c;
        let mut script = script! {
            { Fr::convert_to_le_bits_toaltstack() }

            { G1Projective::push_zero() }
        };

        for script_line in loop_scripts {
            script = script.push_script(script_line.compile());
        }

        (script, hints)
    }

}

pub struct G1Affine;

impl G1Affine {
    pub fn identity() -> Script {
        script! {
            { Fq::push_zero() }
            { Fq::push_zero() }
        }
    }

    pub fn is_on_curve() -> Script {
        script! {
            { Fq::copy(1) }
            { Fq::square() }
            { Fq::roll(2) }
            { Fq::mul() }
            { Fq::push_hex("3") }
            { Fq::add(1, 0) }
            { Fq::roll(1) }
            { Fq::square() }
            { Fq::equal(1, 0) }
        }
    }

    pub fn convert_to_compressed() -> Script {
        script! {
            // move y to the altstack
            { Fq::toaltstack() }
            // convert x into bytes
            { Fq::convert_to_be_bytes() }
            // bring y to the main stack
            { Fq::fromaltstack() }
            { Fq::decode_montgomery() }
            // push (q + 1) / 2
            { U254::push_hex(Fq::P_PLUS_ONE_DIV2) }
            // check if y >= (q + 1) / 2
            { U254::greaterthanorequal(1, 0) }
            // modify the most significant byte
            OP_IF
                { 0x80 } OP_ADD
            OP_ENDIF
        }
    }
    // Init stack: [x1,y1,x2,y2)
    pub fn equalverify() -> Script {
        script! {
            { Fq::roll(2) }
            { Fq::equalverify(1, 0) }
            { Fq::equalverify(1, 0) }
        }
    }
    // Input Stack: [x,y]
    // Output Stack: [x,y,z] (z=1)
    pub fn into_projective() -> Script { script!({ Fq::push_one() }) }
}

#[cfg(test)]
mod test {

    use crate::bn254::curves::{G1Affine, G1Projective};
    use crate::bn254::fq::Fq;
    use crate::bn254::utils::{fr_push, fr_push_not_montgomery, g1_affine_push, g1_affine_push_not_montgomery};
    use crate::treepp::*;
    use crate::{execute_script, execute_script_as_chunks, execute_script_without_stack_limit, run};

    use crate::bn254::fp254impl::Fp254Impl;
    use ark_bn254::Fr;
    use ark_ec::{AffineRepr, CurveGroup};
    use ark_ff::{BigInteger, Field, PrimeField};
    use ark_std::{end_timer, start_timer, UniformRand};
    use core::ops::{Add, Mul};
    use num_bigint::BigUint;
    use num_traits::{One, Zero};
    // use std::ops::Mul;

    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;
    use std::ops::Neg;

    #[test]
    fn test_affine_identity() {
        let equalverify = G1Affine::equalverify();
        println!("G1Affine.equalverify: {} bytes", equalverify.len());

        for _ in 0..1 {
            let expect = ark_bn254::G1Affine::identity();

            let script = script! {
                { G1Affine::identity() }
                { g1_affine_push(expect) }
                { equalverify.clone() }
                OP_TRUE
            };
            println!("curves::test_affine_identity = {} bytes", script.len());
            run(script);
        }
    }

    #[test]
    fn test_copy() {
        println!("G1.copy: {} bytes", G1Projective::copy(1).len());
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..1 {
            let a = ark_bn254::G1Projective::rand(&mut prng);
            let b = ark_bn254::G1Projective::rand(&mut prng);

            let script = script! {
                { G1Projective::push(a) }
                { G1Projective::push(b) }

                // Copy a
                { G1Projective::copy(1) }

                // Push another `a` and then compare
                { G1Projective::push(a) }
                { G1Projective::equalverify() }

                // Drop the original a and b
                { G1Projective::drop() }
                { G1Projective::drop() }
                OP_TRUE
            };
            println!("curves::test_copy = {} bytes", script.len());
            run(script);
        }
    }

    #[test]
    fn test_roll() {
        println!("G1.roll: {} bytes", G1Projective::roll(1).len());
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..1 {
            let a = ark_bn254::G1Projective::rand(&mut prng);
            let b = ark_bn254::G1Projective::rand(&mut prng);

            let script = script! {
                { G1Projective::push(a) }
                { G1Projective::push(b) }

                // Roll a
                { G1Projective::roll(1) }

                // Push another `a` and then compare
                { G1Projective::push(a) }
                { G1Projective::equalverify() }

                // Drop the original a and b
                { G1Projective::drop() }
                OP_TRUE
            };
            println!("curves::test_roll = {} bytes", script.len());
            run(script);
        }
    }

    #[test]
    fn test_double_projective() {
        println!("G1.double: {} bytes", G1Projective::double().len());
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..1 {
            let a = ark_bn254::G1Projective::rand(&mut prng);
            let c = a.add(&a);

            let script = script! {
                { G1Projective::push(a) }
                { G1Projective::double() }
                { G1Projective::push(c) }
                { G1Projective::equalverify() }
                OP_TRUE
            };
            println!("curves::test_double_projective = {} bytes", script.len());
            run(script);
        }
    }

    #[test]
    fn test_hinted_double_projective() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..1 {
            let a = ark_bn254::G1Projective::rand(&mut prng);
            let c = a.add(&a);

            let (hinted_double, mut hints) = G1Projective::hinted_double(a);
            let (hinted_equal_verify1, hints1) = G1Projective::hinted_equalverify(a+a, c);
            hints.extend(hints1);
            println!("G1.hinted_double: {} bytes", hinted_double.len());

            let script = script! {
                for hint in hints { 
                    { hint.push() }
                }
                { G1Projective::push_not_montgomery(a) }
                { hinted_double }
                { G1Projective::push_not_montgomery(c) }
                { hinted_equal_verify1 }
                OP_TRUE
            };
            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_nonzero_add_projective() {
        println!(
            "G1.nonzero_add: {} bytes",
            G1Projective::nonzero_add().len()
        );
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..1 {
            let a = ark_bn254::G1Projective::rand(&mut prng);
            let b = ark_bn254::G1Projective::rand(&mut prng);
            let c = a.add(&b);

            let script = script! {
                { G1Projective::push(a) }
                { G1Projective::push(b) }
                { G1Projective::nonzero_add() }
                { G1Projective::push(c) }
                { G1Projective::equalverify() }
                OP_TRUE
            };
            println!(
                "curves::test_nonzero_add_projective = {} bytes",
                script.len()
            );
            run(script);
        }
    }

    #[test]
    fn test_hinted_nonzero_add_projective() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..1 {
            let a = ark_bn254::G1Projective::rand(&mut prng);
            let b = ark_bn254::G1Projective::rand(&mut prng);
            let c = a.add(&b);

            let (hinted_nonzero_add, mut hints) = G1Projective::hinted_nonzero_add(a, b);
            let (hinted_equal_verify1, hints1) = G1Projective::hinted_equalverify(a+b, c);
            println!("G1.hinted_nonzero_add: {} bytes", hinted_nonzero_add.len());
            hints.extend(hints1);

            let script = script! {
                for hint in hints { 
                    { hint.push() }
                }
                { G1Projective::push_not_montgomery(a) }
                { G1Projective::push_not_montgomery(b) }
                { hinted_nonzero_add }
                { G1Projective::push_not_montgomery(c) }
                { hinted_equal_verify1 }
                OP_TRUE
            };
            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_add_curves() {
        println!("G1.nonzero_add: {} bytes", G1Projective::add().len());
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..1 {
            let a = ark_bn254::G1Projective::rand(&mut prng);
            let b = ark_bn254::G1Projective::rand(&mut prng);
            let c = a.add(&b);

            let script = script! {
                // Test random a + b = c
                { G1Projective::push(a) }
                { G1Projective::push(b) }
                { G1Projective::add() }
                { G1Projective::push(c) }
                { G1Projective::equalverify() }

                // Test random a + 0 = a
                { G1Projective::push(a) }
                { G1Projective::push_zero() }
                { G1Projective::add() }
                { G1Projective::push(a) }
                { G1Projective::equalverify() }

                // Test random 0 + a = a
                { G1Projective::push_zero() }
                { G1Projective::push(a) }
                { G1Projective::add() }
                { G1Projective::push(a) }
                { G1Projective::equalverify() }

                OP_TRUE
            };
            println!("curves::test_add = {} bytes", script.len());
            run_as_chunks(script, 30_000, 1000);
        }
    }

    #[test]
    fn test_hinted_add_curves() {
        println!("G1.hinted_add: {} bytes", G1Projective::add().len());
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..1 {
            let a = ark_bn254::G1Projective::rand(&mut prng);
            let b = ark_bn254::G1Projective::rand(&mut prng);
            let c = a.add(&b);
            let mut hints = Vec::new();
            let (hinted_add1, hints1) = G1Projective::hinted_add(a, b);
            let (hinted_equal_verify1, hints2) = G1Projective::hinted_equalverify(a+b, c);
            let (hinted_add2, hints3) = G1Projective::hinted_add(a, ark_bn254::G1Projective::zero());
            let (hinted_equal_verify2, hints4) = G1Projective::hinted_equalverify(a+ark_bn254::G1Projective::zero(), a);
            let (hinted_add3, hints5) = G1Projective::hinted_add(ark_bn254::G1Projective::zero(), a);
            let (hinted_equal_verify3, hints6) = G1Projective::hinted_equalverify(ark_bn254::G1Projective::zero()+a, a);
            hints.extend(hints1);
            hints.extend(hints2);
            hints.extend(hints3);
            hints.extend(hints4);
            hints.extend(hints5);
            hints.extend(hints6);
            let script = script! {
                for hint in hints { 
                    { hint.push() }
                }
                // Test random a + b = c
                { G1Projective::push_not_montgomery(a) }
                { G1Projective::push_not_montgomery(b) }
                { hinted_add1 }
                { G1Projective::push_not_montgomery(c) }
                { hinted_equal_verify1 }

                // Test random a + 0 = a
                { G1Projective::push_not_montgomery(a) }
                { G1Projective::push_zero() }
                { hinted_add2 }
                { G1Projective::push_not_montgomery(a) }
                { hinted_equal_verify2 }

                // Test random 0 + a = a
                { G1Projective::push_zero() }
                { G1Projective::push_not_montgomery(a) }
                { hinted_add3 }
                { G1Projective::push_not_montgomery(a) }
                { hinted_equal_verify3 }

                OP_TRUE
            };
            println!("curves::test_hinted_add = {} bytes", script.len());
            run(script);
        }
    }
    
    #[test]
    fn test_scalar_mul() {
        let scalar_mul = G1Projective::scalar_mul();
        println!("G1.scalar_mul: {} bytes", scalar_mul.len());

        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..1 {
            let scalar = Fr::rand(&mut prng);

            let p = ark_bn254::G1Projective::rand(&mut prng);
            let q = p.mul(scalar);

            let script = script! {
                { G1Projective::push(p) }
                { fr_push(scalar) }
                { scalar_mul.clone() }
                { G1Projective::push(q) }
                { G1Projective::equalverify() }
                OP_TRUE
            };
            println!("curves::test_scalar_mul = {} bytes", script.len());
            run(script);
        }
    }

    #[test]
    fn test_scalar_mul_by_constant_g1() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..1 {
            let scalar = Fr::rand(&mut prng);

            let p = ark_bn254::G1Projective::rand(&mut prng);
            let q = p.mul(scalar);

            let scalar_mul = G1Projective::scalar_mul_by_constant_g1(p);
            println!("G1.scalar_mul_by_constant_g1: {} bytes", scalar_mul.len());

            let script = script! {
                { fr_push(scalar) }
                { scalar_mul.clone() }
                { G1Projective::push(q) }
                { G1Projective::equalverify() }
                OP_TRUE
            };
            println!("curves::test_scalar_mul = {} bytes", script.len());
            run(script);
        }
    }

    #[test]
    fn test_hinted_scalar_mul_by_constant_g1() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..1 {
            let scalar = Fr::rand(&mut prng);

            let mut p = ark_bn254::G1Projective::rand(&mut prng);
            let q = p.mul(scalar);

            let (hinted_scalar_mul, mut hints) = G1Projective::hinted_scalar_mul_by_constant_g1(scalar, &mut p);
            assert_eq!(p, q);
            println!("G1.scalar_mul_by_constant_g1: {} bytes", hinted_scalar_mul.len());
            let (hinted_equal_verify, hint1) = G1Projective::hinted_equalverify(p, q);
            hints.extend(hint1);

            let script = script! {
                for hint in hints { 
                    { hint.push() }
                }
                { fr_push_not_montgomery(scalar) }
                { hinted_scalar_mul.clone() }
                { G1Projective::push_not_montgomery(q) }
                { hinted_equal_verify }
                OP_TRUE
            };
            println!("curves::test_scalar_mul = {} bytes", script.len());
            let exec_result = execute_script_without_stack_limit(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    // #[ignore]
    fn test_projective_into_affine() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..1 {
            let scalar = Fr::rand(&mut prng);

            let p_zero = ark_bn254::G1Projective::zero();
            let q_zero = p_zero.into_affine();

            let q_z_one = ark_bn254::G1Affine::rand(&mut prng);
            let p_z_one = ark_bn254::G1Projective::from(q_z_one);

            let p = ark_bn254::G1Projective::rand(&mut prng).mul(scalar);
            assert!(!p.z.is_one() && !p.z.is_zero());
            let q = p.into_affine();
            let z = p.z;
            let z_inv = z.inverse().unwrap();
            let z_inv_pow2 = z_inv.square();
            let z_inv_pow3 = z_inv_pow2.mul(z_inv);

            let start = start_timer!(|| "collect_script");

            let script = script! {
                // When point is zero.
                { G1Projective::push(p_zero) }
                { G1Projective::into_affine() }
                { g1_affine_push(q_zero) }
                { G1Affine::equalverify() }

                // when  p.z = one
                { G1Projective::push(p_z_one) }
                { G1Projective::into_affine() }
                { g1_affine_push(q_z_one) }
                { G1Affine::equalverify() }

                // Otherwise, (X,Y,Z)->(X/z^2, Y/z^3)
                { G1Projective::push(p) }
                { G1Projective::into_affine() }
                { g1_affine_push(q) }
                { G1Affine::equalverify() }
                OP_TRUE
            };
            end_timer!(start);

            println!(
                "curves::test_projective_into_affine = {} bytes",
                script.len()
            );
            let if_interval = script.max_op_if_interval();
            println!(
                "Max interval: {:?} debug info: {}, {}",
                if_interval,
                script.debug_info(if_interval.0),
                script.debug_info(if_interval.1)
            );

            let start = start_timer!(|| "execute_script");
            run(script);
            end_timer!(start);
        }
    }

    #[test]
    // #[ignore]
    fn test_hinted_projective_into_affine() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..1 {
            let scalar = Fr::rand(&mut prng);

            let p_zero = ark_bn254::G1Projective::zero();
            let q_zero = p_zero.into_affine();

            let q_z_one = ark_bn254::G1Affine::rand(&mut prng);
            let p_z_one = ark_bn254::G1Projective::from(q_z_one);

            let p = ark_bn254::G1Projective::rand(&mut prng).mul(scalar);
            assert!(!p.z.is_one() && !p.z.is_zero());
            let q = p.into_affine();
            let z = p.z;
            let z_inv = z.inverse().unwrap();
            let z_inv_pow2 = z_inv.square();
            let z_inv_pow3 = z_inv_pow2.mul(z_inv);

            let (hinted_into_affine_zero, hints_zero) = G1Projective::hinted_into_affine(p_zero);
            let (hinted_into_affine_z_one, hints_z_one) = G1Projective::hinted_into_affine(p_z_one);
            let (hinted_into_affine, hints) = G1Projective::hinted_into_affine(p);

            let start = start_timer!(|| "collect_script");

            let script = script! {
                for hint in hints_zero {
                    { hint.push() }
                }
                // When point is zero.
                { G1Projective::push_not_montgomery(p_zero) }
                { hinted_into_affine_zero }
                { g1_affine_push_not_montgomery(q_zero) }
                { G1Affine::equalverify() }

                for hint in hints_z_one {
                    { hint.push() }
                }
                // when  p.z = one
                { G1Projective::push_not_montgomery(p_z_one) }
                { hinted_into_affine_z_one }
                { g1_affine_push_not_montgomery(q_z_one) }
                { G1Affine::equalverify() }

                for hint in hints {
                    { hint.push() }
                }
                // Otherwise, (X,Y,Z)->(X/z^2, Y/z^3)
                { G1Projective::push_not_montgomery(p) }
                { hinted_into_affine }
                { g1_affine_push_not_montgomery(q) }
                { G1Affine::equalverify() }
                
                OP_TRUE
            };
            end_timer!(start);

            let start = start_timer!(|| "execute_script");
            let exec_result = execute_script(script);
            println!("Exec result: {}", exec_result);
            end_timer!(start);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_batched_scalar_mul2() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        // println!(
        // "script size is {}",
        // G1Projective::batched_scalar_mul::<2>().len()
        // );

        for _ in 0..1 {
            let scalar0 = Fr::rand(&mut prng);
            println!("scalar0 => {}", scalar0);
            let point0 = ark_bn254::G1Projective::rand(&mut prng);
            let scalar1 = Fr::rand(&mut prng);
            println!("scalar1 => {}", scalar1);
            let point1 = ark_bn254::G1Projective::rand(&mut prng);

            // batched_scalar_mul
            let q0 = point0.mul(scalar0);
            let q1 = point1.mul(scalar1);
            let q0q1 = q0.add(q1);

            let script = script! {
                { G1Projective::push(point0) }
                { fr_push(scalar0) }
                { G1Projective::push(point1) }
                { fr_push(scalar1) }
                { G1Projective::batched_scalar_mul::<2>() }
                { G1Projective::push(q0q1) }
                { G1Projective::equalverify() }
                OP_TRUE
            };
            let if_interval = script.max_op_if_interval();
            println!(
                "Max interval: {:?} debug info: {}, {}",
                if_interval,
                script.debug_info(if_interval.0),
                script.debug_info(if_interval.1)
            );
            run(script);
        }
    }

    #[test]
    fn test_affine_into_projective() {
        let equalverify = G1Projective::equalverify();
        println!("G1.equalverify: {} bytes", equalverify.len());

        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..1 {
            let scalar = Fr::rand(&mut prng);

            let p = ark_bn254::G1Projective::rand(&mut prng).mul(scalar);
            let q = p.into_affine();

            let start = start_timer!(|| "collect_script");

            let script = script! {
                { g1_affine_push(q) }
                { G1Affine::into_projective() }
                { G1Projective::push(p) }
                { equalverify.clone() }
                OP_TRUE
            };
            end_timer!(start);

            println!(
                "curves::test_affine_into_projective = {} bytes",
                script.len()
            );
            let start = start_timer!(|| "execute_script");
            run(script);
            end_timer!(start);
        }
    }

    #[test]
    fn test_batched_scalar_mul3() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        // println!(
        // "script size is {}",
        // G1Projective::batched_scalar_mul::<2>().len()
        // );

        for _ in 0..1 {
            let scalar0 = Fr::rand(&mut prng);
            println!("scalar0 => {}", scalar0);
            let point0 = ark_bn254::G1Projective::rand(&mut prng);

            let scalar1 = Fr::rand(&mut prng);
            println!("scalar1 => {}", scalar1);
            let point1 = ark_bn254::G1Projective::rand(&mut prng);

            let scalar2 = Fr::rand(&mut prng);
            println!("scalar2 => {}", scalar2);
            let point2 = ark_bn254::G1Projective::rand(&mut prng);

            // let scalar3 = Fr::rand(&mut prng);
            // println!("scalar3 => {}", scalar3);
            // let point3 = ark_bn254::G1Projective::rand(&mut prng);

            // let scalar4 = Fr::rand(&mut prng);
            // println!("scalar4 => {}", scalar4);
            // let point4 = ark_bn254::G1Projective::rand(&mut prng);

            // batched_scalar_mul
            let q0 = point0.mul(scalar0);
            let q1 = point1.mul(scalar1);
            let q2 = point2.mul(scalar2);
            // let q3 = point3.mul(scalar3);
            // let q4 = point4.mul(scalar4);
            let sum = q0.add(q1).add(q2);

            let script = script! {
                { G1Projective::push(point0) }
                { fr_push(scalar0) }
                { G1Projective::push(point1) }
                { fr_push(scalar1) }
                { G1Projective::push(point2) }
                { fr_push(scalar2) }
                // { G1Projective::push(point3) }
                // { fr_push(scalar3) }
                // { G1Projective::push(point4) }
                // { fr_push(scalar4) }

                { G1Projective::batched_scalar_mul::<3>() }
                { G1Projective::push(sum) }
                { G1Projective::equalverify() }
                OP_TRUE
            };
            run(script);
        }
    }

    #[test]
    fn test_projective_equalverify() {
        let equalverify = G1Projective::equalverify();
        println!("G1.equalverify: {} bytes", equalverify.len());

        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..1 {
            let scalar = Fr::rand(&mut prng);

            let p = ark_bn254::G1Projective::rand(&mut prng).mul(scalar);
            let q = p.into_affine();

            let script = script! {
                { G1Projective::push(p) }
                { Fq::push_u32_le(&BigUint::from(q.x).to_u32_digits()) }
                { Fq::push_u32_le(&BigUint::from(q.y).to_u32_digits()) }
                { Fq::push_one() }
                { equalverify.clone() }
                OP_TRUE
            };
            println!("curves::test_equalverify = {} bytes", script.len());
            run(script);
        }
    }

    #[test]
    fn test_hinted_projective_equalverify() {
        let mut prng = ChaCha20Rng::seed_from_u64(0);
        
        for _ in 0..1 {
            let scalar = Fr::rand(&mut prng);
            
            let p = ark_bn254::G1Projective::rand(&mut prng).mul(scalar);
            let q = p.into_affine();
            let (equalverify, hints) = G1Projective::hinted_equalverify(p, q.into_group());
            println!("G1.equalverify: {} bytes", equalverify.len());

            let script = script! {
                for hint in hints {
                    { hint.push() }
                }
                { G1Projective::push_not_montgomery(p) }
                { Fq::push_u32_le_not_montgomery(&BigUint::from(q.x).to_u32_digits()) }
                { Fq::push_u32_le_not_montgomery(&BigUint::from(q.y).to_u32_digits()) }
                { Fq::push_one_not_montgomery() }
                { equalverify.clone() }
                OP_TRUE
            };
            println!("curves::test_equalverify = {} bytes", script.len());
            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_affine_equalverify() {
        let equalverify = G1Affine::equalverify();
        println!("G1Affine.equalverify: {} bytes", equalverify.len());

        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..1 {
            let scalar = Fr::rand(&mut prng);

            let p = ark_bn254::G1Projective::rand(&mut prng).mul(scalar);
            let q = p.into_affine();

            let script = script! {
                { g1_affine_push(p.into_affine()) }
                { g1_affine_push(q) }
                { equalverify.clone() }
                OP_TRUE
            };
            println!("curves::test_equalverify = {} bytes", script.len());
            run(script);
        }
    }

    #[test]
    fn test_affine_is_on_curve() {
        let affine_is_on_curve = G1Affine::is_on_curve();
        println!("G1.affine_is_on_curve: {} bytes", affine_is_on_curve.len());

        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..3 {
            let p = ark_bn254::G1Affine::rand(&mut prng);

            let script = script! {
                { g1_affine_push(p) }
                { affine_is_on_curve.clone() }
            };
            run(script);

            let script = script! {
                { g1_affine_push(p) }
                { Fq::double(0) }
                { affine_is_on_curve.clone() }
                OP_NOT
            };
            println!("curves::test_affine_is_on_curve = {} bytes", script.len());
            run(script);
        }
    }

    #[test]
    fn test_convert_to_compressed() {
        let convert_to_compressed_script = G1Affine::convert_to_compressed();
        println!(
            "G1.convert_to_compressed_script: {} bytes",
            convert_to_compressed_script.len()
        );

        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..3 {
            let mut p = ark_bn254::G1Affine::rand(&mut prng);
            if p.y()
                .unwrap()
                .gt(&ark_bn254::Fq::from_bigint(ark_bn254::Fq::MODULUS_MINUS_ONE_DIV_TWO).unwrap())
            {
                p = p.neg();
            }

            let bytes = p.x().unwrap().into_bigint().to_bytes_be();

            let script = script! {
                { g1_affine_push(p) }
                { convert_to_compressed_script.clone() }
                for i in 0..32 {
                    { bytes[i] } OP_EQUALVERIFY
                }
                OP_TRUE
            };
            run(script);
        }

        for _ in 0..3 {
            let mut p = ark_bn254::G1Affine::rand(&mut prng);
            if p.y()
                .unwrap()
                .into_bigint()
                .le(&ark_bn254::Fq::MODULUS_MINUS_ONE_DIV_TWO)
            {
                p = p.neg();
            }
            assert!(p
                .y()
                .unwrap()
                .into_bigint()
                .gt(&ark_bn254::Fq::MODULUS_MINUS_ONE_DIV_TWO));

            let bytes = p.x().unwrap().into_bigint().to_bytes_be();

            let script = script! {
                { g1_affine_push(p) }
                { convert_to_compressed_script.clone() }
                { bytes[0] | 0x80 }
                OP_EQUALVERIFY
                for i in 1..32 {
                    { bytes[i] } OP_EQUALVERIFY
                }
                OP_TRUE
            };
            run(script);
        }

        for _ in 0..3 {
            let p = ark_bn254::G1Affine::rand(&mut prng);
            let bytes = p.x().unwrap().into_bigint().to_bytes_be();

            let script = script! {
                { Fq::push_u32_le(&BigUint::from(p.x).to_u32_digits()) }
                { Fq::push_hex(Fq::P_PLUS_ONE_DIV2) }
                { convert_to_compressed_script.clone() }
                { bytes[0] | 0x80 }
                OP_EQUALVERIFY
                for i in 1..32 {
                    { bytes[i] } OP_EQUALVERIFY
                }
                OP_TRUE
            };
            run(script);
        }
    }
}
