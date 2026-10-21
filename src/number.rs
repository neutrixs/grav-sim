use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};
use std::ops;

pub const FRAC_BITS: usize = 64;

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