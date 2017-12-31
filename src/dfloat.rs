use core::ops::{Neg, Add, Sub, Mul, Div};
use core::cmp::{PartialEq, PartialOrd, Ordering};
use core::clone::Clone;
use num_traits::{Zero, One, Bounded};
use float_traits::*;
#[cfg(not(feature = "use-fma"))]
use safeeft::{fasttwosum, safetwosum_straight as safetwosum,
              safetwoproduct_branch as safetwoproduct};
#[cfg(feature = "use-fma")]
use safeeft::{fasttwosum, safetwosum_fma as safetwosum, safetwoproduct_fma as safetwoproduct};
#[cfg(feature = "use-fma")]
use fma::Fma;

#[cfg(not(feature = "use-fma"))]
pub trait FloatComponent: IEEE754Float + Clone {}
#[cfg(not(feature = "use-fma"))]
impl<T: IEEE754Float + Clone> FloatComponent for T {}
#[cfg(feature = "use-fma")]
pub trait FloatComponent: IEEE754Float + Fma + Clone {}
#[cfg(feature = "use-fma")]
impl<T: IEEE754Float + Fma + Clone> FloatComponent for T {}


#[derive(Debug, Clone)]
pub struct DFloat<T: FloatComponent> {
    high: T,
    low: T,
}

impl<T: FloatComponent> Zero for DFloat<T> {
    fn zero() -> DFloat<T> {
        DFloat {
            high: T::zero(),
            low: T::zero(),
        }
    }
    fn is_zero(&self) -> bool {
        self.high.is_zero() && self.low.is_zero()
    }
}

impl<T: FloatComponent> One for DFloat<T> {
    fn one() -> DFloat<T> {
        DFloat {
            high: T::one(),
            low: T::zero(),
        }
    }
}

impl<T: FloatComponent> Bounded for DFloat<T> {
    fn max_value() -> DFloat<T> {
        DFloat {
            high: T::max_value(),
            low: T::max_value() * T::eps() / T::radix() / T::radix(),
        }
    }
    fn min_value() -> DFloat<T> {
        DFloat {
            high: T::min_value(),
            low: T::min_value() * T::eps() / T::radix() / T::radix(),
        }
    }
}

impl<T: FloatComponent> DFloat<T> {
    #[inline]
    pub fn zero() -> DFloat<T> {
        <DFloat<T> as Zero>::zero()
    }
    #[inline]
    pub fn one() -> DFloat<T> {
        <DFloat<T> as One>::one()
    }
    #[inline]
    pub fn infinity() -> DFloat<T> {
        DFloat {
            high: T::infinity(),
            low: T::zero(),
        }
    }
    #[inline]
    pub fn neg_infinity() -> DFloat<T> {
        DFloat {
            high: T::neg_infinity(),
            low: T::zero(),
        }
    }
    #[inline]
    pub fn max_value() -> DFloat<T> {
        <DFloat<T> as Bounded>::max_value()
    }
    #[inline]
    pub fn min_value() -> DFloat<T> {
        <DFloat<T> as Bounded>::min_value()
    }
    #[inline]
    pub fn min_positive() -> DFloat<T> {
        DFloat {
            high: T::unit_underflow(),
            low: T::zero(),
        }
    }
    #[inline]
    pub fn from_component(t: T) -> DFloat<T> {
        DFloat {
            high: t,
            low: T::zero(),
        }
    }
    #[inline]
    pub fn from_two_components(high: T, low: T) -> DFloat<T> {
        let t = safetwosum(high, low);
        if t.0.is_infinite() {
            DFloat {
                high: t.0,
                low: T::zero(),
            }
        } else {
            DFloat {
                high: t.0,
                low: t.1,
            }

        }
    }
    #[doc(hidden)]
    #[inline]
    pub unsafe fn from_double_components_unchecked(high: T, low: T) -> DFloat<T> {
        DFloat {
            high: high,
            low: low,
        }
    }
    #[inline]
    pub fn high(&self) -> &T {
        &self.high
    }
    #[inline]
    pub fn low(&self) -> &T {
        &self.low
    }
    #[inline]
    pub fn into_tuple(self) -> (T, T) {
        (self.high, self.low)
    }
}

impl<T: FloatComponent> PartialEq for DFloat<T> {
    fn eq(&self, rhs: &Self) -> bool {
        (self.high == rhs.high) && ((self.low == rhs.low) || (self.high.is_infinite()))
    }
}

impl<T: FloatComponent> PartialOrd for DFloat<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        if self.high != rhs.high {
            if self.high > rhs.high {
                Some(Ordering::Greater)
            } else if self.high < rhs.high {
                Some(Ordering::Less)
            } else {
                // NaN
                None
            }
        } else if self.high.is_infinite() {
            Some(Ordering::Equal)
        } else {
            if self.low > rhs.low {
                Some(Ordering::Greater)
            } else if self.low < rhs.low {
                Some(Ordering::Less)
            } else {
                Some(Ordering::Equal)
            }
        }
    }
}

impl<T: FloatComponent> Neg for DFloat<T> {
    type Output = DFloat<T>;

    #[inline]
    fn neg(self) -> DFloat<T> {
        DFloat {
            high: -self.high,
            low: -self.low,
        }
    }
}

impl<T: FloatComponent> Add<DFloat<T>> for DFloat<T> {
    type Output = DFloat<T>;
    fn add(self, rhs: DFloat<T>) -> DFloat<T> {
        let (sh, sl) = safetwosum(self.high, rhs.high);
        let (sh, sl) = fasttwosum(sh, sl + self.low + rhs.low);
        DFloat { high: sh, low: sl }
    }
}

impl<'a, T: FloatComponent> Add<&'a DFloat<T>> for DFloat<T> {
    type Output = DFloat<T>;
    fn add(self, rhs: &'a DFloat<T>) -> DFloat<T> {
        let (sh, sl) = safetwosum(self.high, rhs.high.clone());
        let (sh, sl) = fasttwosum(sh, sl + self.low + rhs.low.clone());
        DFloat { high: sh, low: sl }
    }
}

impl<T: FloatComponent> Sub<DFloat<T>> for DFloat<T> {
    type Output = DFloat<T>;
    fn sub(self, rhs: DFloat<T>) -> DFloat<T> {
        let (sh, sl) = safetwosum(self.high, -rhs.high);
        let (sh, sl) = fasttwosum(sh, sl + self.low - rhs.low);
        DFloat { high: sh, low: sl }
    }
}

impl<'a, T: FloatComponent> Sub<&'a DFloat<T>> for DFloat<T> {
    type Output = DFloat<T>;
    fn sub(self, rhs: &'a DFloat<T>) -> DFloat<T> {
        let (sh, sl) = safetwosum(self.high, -rhs.high.clone());
        let (sh, sl) = fasttwosum(sh, sl + self.low - rhs.low.clone());
        DFloat { high: sh, low: sl }
    }
}

impl<T: FloatComponent> Mul<DFloat<T>> for DFloat<T> {
    type Output = DFloat<T>;
    fn mul(self, rhs: DFloat<T>) -> DFloat<T> {
        let (mh, ml) = safetwoproduct(self.high.clone(), rhs.high.clone());
        DFloat {
            high: mh,
            low: ml + self.low * rhs.high + self.high * rhs.low,
        }
    }
}

impl<'a, T: FloatComponent> Mul<&'a DFloat<T>> for DFloat<T> {
    type Output = DFloat<T>;
    fn mul(self, rhs: &'a DFloat<T>) -> DFloat<T> {
        let (mh, ml) = safetwoproduct(self.high.clone(), rhs.high.clone());
        DFloat {
            high: mh,
            low: ml + self.low * rhs.high.clone() + self.high * rhs.low.clone(),
        }
    }
}

impl<T: FloatComponent> Div<DFloat<T>> for DFloat<T> {
    type Output = DFloat<T>;
    fn div(self, rhs: DFloat<T>) -> DFloat<T> {
        let qh = self.high.clone() / rhs.high.clone();
        if rhs.high.is_infinite() || qh.is_infinite() {
            DFloat {
                high: qh,
                low: T::zero(),
            }
        } else {
            let (z3, z4) = safetwoproduct(-qh.clone(), rhs.high.clone());
            let ql = if z3.is_infinite() {
                let (z3, z4) = safetwoproduct(-qh.clone(), rhs.high.clone() / T::radix());
                ((((z3 + (self.high / T::radix())) - qh.clone() * (rhs.low / T::radix())) +
                  self.low / T::radix()) + z4) / (rhs.high / T::radix())
            } else {
                ((((z3 + self.high) - qh.clone() * rhs.low) + self.low) + z4) / rhs.high
            };
            let (qh, ql) = fasttwosum(qh, ql);
            DFloat { high: qh, low: ql }
        }
    }
}

impl<'a, T: FloatComponent> Div<&'a DFloat<T>> for DFloat<T> {
    type Output = DFloat<T>;
    fn div(self, rhs: &'a DFloat<T>) -> DFloat<T> {
        let qh = self.high.clone() / rhs.high.clone();
        if rhs.high.is_infinite() || qh.is_infinite() {
            DFloat {
                high: qh,
                low: T::zero(),
            }
        } else {
            let (z3, z4) = safetwoproduct(-qh.clone(), rhs.high.clone());
            let ql = if z3.is_infinite() {
                let (z3, z4) = safetwoproduct(-qh.clone(), rhs.high.clone() / T::radix());
                ((((z3 + (self.high / T::radix())) - qh.clone() * (rhs.low.clone() / T::radix())) +
                  self.low / T::radix()) + z4) / (rhs.high.clone() / T::radix())
            } else {
                ((((z3 + self.high) - qh.clone() * rhs.low.clone()) + self.low) + z4) /
                rhs.high.clone()
            };
            let (qh, ql) = fasttwosum(qh, ql);
            DFloat { high: qh, low: ql }
        }
    }
}

impl<T: FloatComponent> DFloat<T> {
    pub fn sqrt(self) -> DFloat<T> {
        if self.high == T::zero() && self.low == T::zero() {
            DFloat::zero()
        } else if self.high.is_infinite() {
            DFloat {
                high: self.high,
                low: T::zero(),
            }
        } else {
            let rh = self.high.clone().sqrt();
            let (r3, r4) = safetwoproduct(-rh.clone(), rh.clone());
            let rl = ((r3 + self.high) + self.low + r4) / (T::radix() * rh.clone());
            let (rh, rl) = fasttwosum(rh, rl);
            DFloat { high: rh, low: rl }
        }
    }
}

mod tests {
    #[test]
    fn add() {
        use super::DFloat;
        let (d1, d2) = (DFloat::<f64>::zero(), DFloat::<f64>::zero());
        assert_eq!(d1.clone() + &d2, d1 + d2);
    }
}
