#![cfg_attr(feature = "use-fma", feature(cfg_target_feature))]
extern crate core;
extern crate num_traits;
extern crate float_traits;
extern crate safeeft;
#[cfg(feature = "use-fma")]
extern crate fma;

mod dfloat;
pub use dfloat::DFloat;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
