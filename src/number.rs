use std::cell::RefCell;
use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};
use std::ops;

pub const FRAC_BITS: usize = 64;

pub struct DivScratch {
    // u/v=q
    u: Vec<u32>,
    v: Vec<u32>,
    q: Vec<u32>,
}

impl DivScratch {
    pub fn new() -> Self {
        DivScratch {
            u: Vec::new(), v: Vec::new(), q: Vec::new()
        }
    }
}

thread_local! {
    static DIV_SCRATCH: RefCell<DivScratch> = RefCell::new(DivScratch::new());
}

pub struct Fixed(BigInt);

impl ops::Add<&Fixed> for &Fixed {
    type Output = Fixed;

    fn add(self, rhs: &Fixed) -> Fixed {
        Fixed(&self.0 + &rhs.0)
    }
}

impl ops::AddAssign<&Fixed> for Fixed {
    fn add_assign(&mut self, rhs: &Fixed) {
        self.0 += &rhs.0;
    }
}

impl ops::Sub<&Fixed> for &Fixed {
    type Output = Fixed;

    fn sub(self, rhs: &Fixed) -> Fixed {
        Fixed(&self.0 - &rhs.0)
    }
}

impl ops::SubAssign<&Fixed> for Fixed {
    fn sub_assign(&mut self, rhs: &Fixed) {
        self.0 -= &rhs.0;
    }
}

impl ops::Mul<&Fixed> for &Fixed {
    type Output = Fixed;

    fn mul(self, rhs: &Fixed) -> Fixed {
        Fixed((&self.0 * &rhs.0) >> FRAC_BITS)
    }
}

impl ops::Div<&Fixed> for &Fixed {
    type Output = Fixed;

    fn div(self, rhs: &Fixed) -> Fixed {
        Fixed((&self.0 << FRAC_BITS) / &rhs.0)
    }
}

impl ops::DivAssign<&Fixed> for Fixed {
    fn div_assign(&mut self, rhs: &Fixed) {
        DIV_SCRATCH.with(|s| {
            let mut s = s.borrow_mut();

            let sign = self.0.sign() * rhs.0.sign();

            s.u.clear();
            s.u.extend(std::iter::repeat(0).take(FRAC_BITS / 32)); // × S
            s.u.extend(self.0.iter_u32_digits());
            trim(&mut s.u);

            s.v.clear();
            s.v.extend(rhs.0.iter_u32_digits());
            trim(&mut s.v);
            assert!(!s.v.is_empty(), "division by zero");

            div_mag(&mut s);
            self.0.assign_from_slice(sign, &s.q);
        })
    }
}

impl ops::Shl<usize> for &Fixed {
    type Output = Fixed;
    fn shl(self, n: usize) -> Fixed {
        Fixed(&self.0 << n)
    }
}

impl ops::ShlAssign<usize> for Fixed {
    fn shl_assign(&mut self, n: usize) {
        self.0 <<= n;
    }
}

impl ops::Shr<usize> for &Fixed {
    type Output = Fixed;
    fn shr(self, n: usize) -> Fixed {
        Fixed(&self.0 >> n)
    }
}

impl ops::ShrAssign<usize> for Fixed {
    fn shr_assign(&mut self, n: usize) {
        self.0 >>= n;
    }
}

impl Clone for Fixed {
    fn clone(&self) -> Fixed {
        Fixed(self.0.clone())
    }

    fn clone_from(&mut self, source: &Fixed) {
        self.0.clone_from(&source.0);
    }
}

impl Fixed {
    pub fn from_f64(v: f64) -> Fixed {
        Fixed(BigInt::from_f64(v * 2f64.powi(FRAC_BITS as i32)).unwrap())
    }
    pub fn to_f64(&self) -> f64 {
        self.0.to_f64().unwrap() / 2f64.powi(FRAC_BITS as i32)
    }

    pub fn sqrt(&self) -> Fixed {
        Fixed((&self.0 << FRAC_BITS).sqrt())
    }

    pub fn square(&self) -> Fixed {
        self * self
    }
}

// u / v = q   (u gets clobbered, ends up holding the remainder, shifted)
fn div_mag(s: &mut DivScratch) {
    let (u, v, q) = (&mut s.u, &mut s.v, &mut s.q);
    let m = u.len();
    let n = v.len();
    q.clear();
    if m < n { return; }                     // u < v → quotient 0
    q.resize(m - n + 1, 0);
    const B: u64 = 1 << 32;

    // 1-limb divisor: simple short division
    if n == 1 {
        let d = v[0] as u64;
        let mut rem = 0u64;
        for j in (0..m).rev() {
            let t = (rem << 32) | u[j] as u64;
            q[j] = (t / d) as u32;
            rem = t % d;
        }
        trim(q);
        return;
    }

    // step 1: normalize so v's top limb has its highest bit set
    let sh = v[n - 1].leading_zeros();
    shl_in_place(v, sh);
    u.push(0);                               // room for bits shifted out the top
    shl_in_place(u, sh);

    for j in (0..=m - n).rev() {
        // step 2: guess qhat from the top 2 limbs
        let num = ((u[j + n] as u64) << 32) | u[j + n - 1] as u64;
        let mut qhat = num / v[n - 1] as u64;
        let mut rhat = num % v[n - 1] as u64;

        // step 3: refine, at most 2 corrections
        while qhat >= B
            || qhat * v[n - 2] as u64 > ((rhat << 32) | u[j + n - 2] as u64)
        {
            qhat -= 1;
            rhat += v[n - 1] as u64;
            if rhat >= B { break; }
        }

        // step 4: u_window -= qhat * v
        let mut k: i64 = 0;
        for i in 0..n {
            let p = qhat * v[i] as u64;
            let t = u[i + j] as i64 - k - (p & 0xFFFF_FFFF) as i64;
            u[i + j] = t as u32;
            k = (p >> 32) as i64 - (t >> 32);
        }
        let t = u[j + n] as i64 - k;
        u[j + n] = t as u32;
        q[j] = qhat as u32;

        // step 5: went negative → qhat was 1 too big, add v back
        if t < 0 {
            q[j] = q[j].wrapping_sub(1);
            let mut c = 0u64;
            for i in 0..n {
                let sum = u[i + j] as u64 + v[i] as u64 + c;
                u[i + j] = sum as u32;
                c = sum >> 32;
            }
            u[j + n] = u[j + n].wrapping_add(c as u32);
        }
    }
    trim(q);
}

fn shl_in_place(x: &mut [u32], sh: u32) {
    if sh == 0 { return; }
    for i in (1..x.len()).rev() {
        x[i] = (x[i] << sh) | (x[i - 1] >> (32 - sh));
    }
    x[0] <<= sh;
}

fn trim(x: &mut Vec<u32>) {
    while x.last() == Some(&0) { x.pop(); }
}

#[test]
fn div_matches_num_bigint() {
    use num_bigint::{BigInt, Sign};

    let raw = |limbs: &[u32]| Fixed(BigInt::from_slice(Sign::Plus, limbs));

    let mut cases: Vec<(Fixed, Fixed)> = [
        (5.0, 2.0),
        (-1.5, 0.25),
        (1.0, 1e-12),          // tiny divisor → 1-limb path
        (123456.789, -3.0),
    ]
        .iter()
        .map(|&(x, y)| (Fixed::from_f64(x), Fixed::from_f64(y)))
        .collect();

    // built to trigger the "add v back" step
    let v = raw(&[0xFFFF_FFFF, 0, 0x8000_0000]);
    cases.push((raw(&[0x8000_0000]), v.clone()));      // guess 1, real 0
    cases.push((raw(&[0x8000_0000, 1]), v.clone()));   // guess 3, real 2

    for (a, b) in &cases {
        // expected: exact integer math, done by num-bigint
        let expected = (&a.0 << FRAC_BITS) / &b.0;

        // DivAssign version
        let mut got = a.clone();
        got /= b;

        assert_eq!(got.0, expected, "{} / {}", a.0, b.0);
    }
}