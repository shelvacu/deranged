#![cfg_attr(not(feature = "std"), no_std)]
#![deny(
    anonymous_parameters,
    clippy::all,
    const_err,
    illegal_floating_point_literal_pattern,
    late_bound_lifetime_arguments,
    path_statements,
    patterns_in_fns_without_body,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates
)]
#![warn(
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::get_unwrap,
    clippy::nursery,
    clippy::pedantic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unwrap_used,
    clippy::use_debug,
    missing_copy_implementations,
    missing_debug_implementations,
    unused_qualifications,
    variant_size_differences
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::redundant_pub_crate
)]
#![doc(test(attr(deny(warnings))))]
#![cfg_attr(feature = "const-eval", feature(const_generics,const_evaluatable_checked))]

#[cfg(feature = "const-eval")]
use core::ops::{Index,IndexMut};
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::convert::{TryFrom, TryInto};
use core::fmt;
#[cfg(feature = "std")]
use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TryFromIntError;

impl fmt::Display for TryFromIntError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("out of range integral type conversion attempted")
    }
}
#[cfg(feature = "std")]
impl Error for TryFromIntError {}

#[cfg(feature = "unsafe-range-assert")]
macro_rules! unsafe_is_range {
    ($min:expr, $max_incl:expr, $val:expr) => {
        {
            let v = $val;
            // We have to do this "manually" because it won't allow ranges made from const generics
            // The comparison is only unused (usually <= 0 on an unsigned type) in half the cases, so isn't generic
            #[allow(clippy::manual_range_contains, unused_comparisons)]
            if $min <= v && v <= $max_incl {
                v
            } else {
                #[allow(unused_unsafe)]
                unsafe { ::core::hint::unreachable_unchecked() }
            }
        }
    };
}

#[cfg(not(feature = "unsafe-range-assert"))]
macro_rules! unsafe_is_range {
    ($min:expr, $max_incl:expr, $val:expr) => {
        {
            let v = $val;
            #[allow(clippy::manual_range_contains, unused_comparisons)]
            if v < $min || v > $max_incl { panic!("Out of range"); }
            v
        }
    }
}
// macro_rules! unsafe_is_range {
//     ($min:expr, $max_incl:expr, $val:expr) => {
//         {
//             let v = $val;
//             #[allow(clippy::manual_range_contains, unused_comparisons)]
//             if $min <= v && v <= $max_incl {
//                 v
//             } else {
//                 panic!("Expected val to be in range {}..={}, was {}", $min, $max_incl, v);
//             }
//         }
//     };
// }

macro_rules! const_try_opt {
    ($e:expr) => {
        match $e {
            Some(value) => value,
            None => return None,
        }
    };
}

macro_rules! if_signed {
    (true $($x:tt)*) => { $($x)*};
    (false $($x:tt)*) => {};
}

macro_rules! article {
    (true) => {
        "An"
    };
    (false) => {
        "A"
    };
}

macro_rules! impl_ranged {
    ($(
        $type:ident {
            internal: $internal:ident
            signed: $is_signed:ident
            into: [$($into:ident),* $(,)?]
            try_into: [$($try_into:ident),* $(,)?]
            try_from: [$($try_from:ident),* $(,)?]
        }
    )*) => {$(
        #[doc = concat!(
            article!($is_signed),
            " `",
            stringify!($internal),
            "` that is known to be in the range `MIN..=MAX`.",
        )]
        #[repr(transparent)]
        #[derive(Clone, Copy, Eq, Ord, Hash)]
        pub struct $type<const MIN: $internal, const MAX: $internal>(
            $internal,
        );

        impl<const MIN: $internal, const MAX: $internal> $type<MIN, MAX> {
            /// The smallest value that can be represented by this type.
            pub const MIN: Self = Self(MIN);

            /// The largest value that can be represented by this type.
            pub const MAX: Self = Self(MAX);

            /// Creates a ranged integer without checking the value.
            ///
            /// # Safety
            ///
            /// The value must be within the range `MIN..=MAX`.
            pub const unsafe fn new_unchecked(value: $internal) -> Self {
                Self(unsafe_is_range!(MIN, MAX, value))
            }

            /// Creates a ranged integer if the given value is in the range
            /// `MIN..=MAX`.
            pub const fn new(value: $internal) -> Option<Self> {
                if value < MIN || value > MAX {
                    None
                } else {
                    Some(Self(value))
                }
            }

            const fn new_saturating(value: $internal) -> Self {
                Self(if value < MIN {
                    MIN
                } else if value > MAX {
                    MAX
                } else {
                    value
                })
            }

            pub fn all_values() -> impl
                core::iter::Iterator<Item=Self> +
                core::iter::FusedIterator +
                core::iter::DoubleEndedIterator {
                (MIN..=MAX).into_iter().map(|n| unsafe { Self::new_unchecked(n) })
            }

            /// Returns the value as a primitive type.
            pub const fn get(self) -> $internal {
                unsafe_is_range!(MIN, MAX, self.0)
            }

            /// Checked integer addition. Computes `self + rhs`, returning
            /// `None` if the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub const fn checked_add(self, rhs: $internal) -> Option<Self> {
                Self::new(const_try_opt!(self.0.checked_add(rhs)))
            }

            /// Checked integer addition. Computes `self - rhs`, returning
            /// `None` if the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub const fn checked_sub(self, rhs: $internal) -> Option<Self> {
                Self::new(const_try_opt!(self.0.checked_sub(rhs)))
            }

            /// Checked integer addition. Computes `self * rhs`, returning
            /// `None` if the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub const fn checked_mul(self, rhs: $internal) -> Option<Self> {
                Self::new(const_try_opt!(self.0.checked_mul(rhs)))
            }

            /// Checked integer addition. Computes `self / rhs`, returning
            /// `None` if `rhs == 0` or if the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub const fn checked_div(self, rhs: $internal) -> Option<Self> {
                Self::new(const_try_opt!(self.0.checked_div(rhs)))
            }

            /// Checked Euclidean division. Computes `self.div_euclid(rhs)`,
            /// returning `None` if `rhs == 0` or if the resulting value is out
            /// of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub const fn checked_div_euclid(self, rhs: $internal) -> Option<Self> {
                Self::new(const_try_opt!(self.0.checked_div_euclid(rhs)))
            }

            /// Checked integer remainder. Computes `self % rhs`, returning
            /// `None` if `rhs == 0` or if the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub const fn checked_rem(self, rhs: $internal) -> Option<Self> {
                Self::new(const_try_opt!(self.0.checked_rem(rhs)))
            }

            /// Checked Euclidean remainder. Computes `self.rem_euclid(rhs)`,
            /// returning `None` if `rhs == 0` or if the resulting value is out
            /// of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub const fn checked_rem_euclid(self, rhs: $internal) -> Option<Self> {
                Self::new(const_try_opt!(self.0.checked_rem_euclid(rhs)))
            }

            /// Checked negation. Computes `-self`, returning `None` if the
            /// resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub const fn checked_neg(self) -> Option<Self> {
                Self::new(const_try_opt!(self.0.checked_neg()))
            }

            /// Checked shift left. Computes `self << rhs`, returning `None` if
            /// the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub const fn checked_shl(self, rhs: u32) -> Option<Self> {
                Self::new(const_try_opt!(self.0.checked_shl(rhs)))
            }

            /// Checked shift right. Computes `self >> rhs`, returning `None` if
            /// the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub const fn checked_shr(self, rhs: u32) -> Option<Self> {
                Self::new(const_try_opt!(self.0.checked_shr(rhs)))
            }

            if_signed!($is_signed
            /// Checked absolute value. Computes `self.abs()`, returning `None`
            /// if the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub const fn checked_abs(self) -> Option<Self> {
                Self::new(const_try_opt!(self.0.checked_abs()))
            });

            /// Checked exponentiation. Computes `self.pow(exp)`, returning
            /// `None` if the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub const fn checked_pow(self, exp: u32) -> Option<Self> {
                Self::new(const_try_opt!(self.0.checked_pow(exp)))
            }

            /// Saturating integer addition. Computes `self + rhs`, saturating
            /// at the numeric bounds.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub const fn saturating_add(self, rhs: $internal) -> Self {
                Self::new_saturating(self.0.saturating_add(rhs))
            }

            /// Saturating integer subtraction. Computes `self - rhs`,
            /// saturating at the numeric bounds.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub const fn saturating_sub(self, rhs: $internal) -> Self {
                Self::new_saturating(self.0.saturating_sub(rhs))
            }

            if_signed!($is_signed
            /// Saturating integer negation. Computes `self - rhs`, saturating
            /// at the numeric bounds.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub const fn saturating_neg(self) -> Self {
                Self::new_saturating(self.0.saturating_neg())
            });

            if_signed!($is_signed
            /// Saturating absolute value. Computes `self.abs()`, saturating at
            /// the numeric bounds.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub const fn saturating_abs(self) -> Self {
                Self::new_saturating(self.0.saturating_abs())
            });

            /// Saturating integer multiplication. Computes `self * rhs`,
            /// saturating at the numeric bounds.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub const fn saturating_mul(self, rhs: $internal) -> Self {
                Self::new_saturating(self.0.saturating_mul(rhs))
            }

            /// Saturating integer exponentiation. Computes `self.pow(exp)`,
            /// saturating at the numeric bounds.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub const fn saturating_pow(self, exp: u32) -> Self {
                Self::new_saturating(self.0.saturating_pow(exp))
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::Debug for $type<MIN, MAX> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::Display for $type<MIN, MAX> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> AsRef<$internal> for $type<MIN, MAX> {
            fn as_ref(&self) -> &$internal {
                let _ = unsafe_is_range!(MIN, MAX, self.0);
                &self.0
            }
        }

        impl<const MIN: $internal, const MAX: $internal> Borrow<$internal> for $type<MIN, MAX> {
            fn borrow(&self) -> &$internal {
                let _ = unsafe_is_range!(MIN, MAX, self.0);
                &self.0
            }
        }

        impl<
            const MIN_A: $internal,
            const MAX_A: $internal,
            const MIN_B: $internal,
            const MAX_B: $internal,
        > PartialEq<$type<MIN_B, MAX_B>> for $type<MIN_A, MAX_A> {
            fn eq(&self, other: &$type<MIN_B, MAX_B>) -> bool {
                unsafe_is_range!(MIN_A, MAX_A, self.0) == unsafe_is_range!(MIN_B, MAX_B, other.0)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> PartialEq<$internal> for $type<MIN, MAX> {
            fn eq(&self, other: &$internal) -> bool {
                unsafe_is_range!(MIN, MAX, self.0) == *other
            }
        }

        impl<const MIN: $internal, const MAX: $internal> PartialEq<$type<MIN, MAX>> for $internal {
            fn eq(&self, other: &$type<MIN, MAX>) -> bool {
                *self == unsafe_is_range!(MIN, MAX, other.0)
            }
        }

        impl<
            const MIN_A: $internal,
            const MAX_A: $internal,
            const MIN_B: $internal,
            const MAX_B: $internal,
        > PartialOrd<$type<MIN_B, MAX_B>> for $type<MIN_A, MAX_A> {
            fn partial_cmp(&self, other: &$type<MIN_B, MAX_B>) -> Option<Ordering> {
                unsafe_is_range!(MIN_A, MAX_A, self.0).partial_cmp(&unsafe_is_range!(MIN_B, MAX_B, other.0))
            }
        }

        impl<const MIN: $internal, const MAX: $internal> PartialOrd<$internal> for $type<MIN, MAX> {
            fn partial_cmp(&self, other: &$internal) -> Option<Ordering> {
                unsafe_is_range!(MIN, MAX, self.0).partial_cmp(other)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> PartialOrd<$type<MIN, MAX>> for $internal {
            fn partial_cmp(&self, other: &$type<MIN, MAX>) -> Option<Ordering> {
                self.partial_cmp(&unsafe_is_range!(MIN, MAX, other.0))
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::Binary for $type<MIN, MAX> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::LowerHex for $type<MIN, MAX> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::UpperHex for $type<MIN, MAX> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::LowerExp for $type<MIN, MAX> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::UpperExp for $type<MIN, MAX> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::Octal for $type<MIN, MAX> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        $(impl<const MIN: $internal, const MAX: $internal> From<$type<MIN,MAX>> for $into {
            fn from(value: $type<MIN, MAX>) -> Self {
                unsafe_is_range!(MIN, MAX, value.0).into()
            }
        })*

        $(impl<const MIN: $internal, const MAX: $internal> TryFrom<$type<MIN, MAX>> for $try_into {
            type Error = TryFromIntError;

            fn try_from(value: $type<MIN, MAX>) -> Result<Self, Self::Error> {
                if (MIN..=MAX).contains(&value.0) {
                    Ok(value.try_into()?)
                } else {
                    Err(TryFromIntError)
                }
            }
        })*

        $(impl<const MIN: $internal, const MAX: $internal> TryFrom<$try_from> for $type<MIN, MAX> {
            type Error = TryFromIntError;

            fn try_from(value: $try_from) -> Result<Self, Self::Error> {
                let value = match TryInto::<$internal>::try_into(value) {
                    Ok(value) => value,
                    Err(_) => return Err(TryFromIntError)
                };

                match Self::new(value) {
                    None => Err(TryFromIntError),
                    Some(v) => Ok(v),
                }
            }
        })*

        #[cfg(feature = "serde")]
        impl<const MIN: $internal, const MAX: $internal> serde::Serialize for $type<MIN, MAX> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.get().serialize(serializer)
            }
        }

        #[cfg(feature = "serde")]
        impl<
            'de,
            const MIN: $internal,
            const MAX: $internal,
        > serde::Deserialize<'de> for $type<MIN, MAX> {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let internal = <$internal>::deserialize(deserializer)?;
                Self::new(internal).ok_or_else(|| <D::Error as serde::de::Error>::invalid_value(
                    serde::de::Unexpected::Other("integer"),
                    &format!("an integer in the range {}..={}", MIN, MAX).as_ref()
                ))
            }
        }

        #[cfg(feature = "rand")]
        impl<
            const MIN: $internal,
            const MAX: $internal,
        > rand::distributions::Distribution<$type<MIN, MAX>> for rand::distributions::Standard {
            fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> $type<MIN, MAX> {
                $type(unsafe_is_range!(MIN, MAX, rng.gen_range(MIN..=MAX)))
            }
        }
    )*};
}

impl_ranged! {
    U8 {
        internal: u8
        signed: false
        into: [u8, u16, u32, u64, u128, i16, i32, i64, i128]
        try_into: [usize, i8, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    U16 {
        internal: u16
        signed: false
        into: [u16, u32, u64, u128, i32, i64, i128]
        try_into: [u8, usize, i8, i16, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    U32 {
        internal: u32
        signed: false
        into: [u32, u64, u128, i64, i128]
        try_into: [u8, u16, usize, i8, i16, i32, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    U64 {
        internal: u64
        signed: false
        into: [u64, u128, i128]
        try_into: [u8, u16, u32, usize, i8, i16, i32, i64, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    U128 {
        internal: u128
        signed: false
        into: [u128]
        try_into: [u8, u16, u32, u64, usize, i8, i16, i32, i64, i128, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    Usize {
        internal: usize
        signed: false
        into: [usize]
        try_into: [u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    I8 {
        internal: i8
        signed: true
        into: [i8, i16, i32, i64, i128]
        try_into: [u8, u16, u32, u64, u128, usize, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    I16 {
        internal: i16
        signed: true
        into: [i16, i32, i64, i128]
        try_into: [u8, u16, u32, u64, u128, usize, i8, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    I32 {
        internal: i32
        signed: true
        into: [i32, i64, i128]
        try_into: [u8, u16, u32, u64, u128, usize, i8, i16, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    I64 {
        internal: i64
        signed: true
        into: [i64, i128]
        try_into: [u8, u16, u32, u64, u128, usize, i8, i16, i32, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    I128 {
        internal: i128
        signed: true
        into: [i128]
        try_into: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, isize]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
    Isize {
        internal: isize
        signed: true
        into: [isize]
        try_into: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128]
        try_from: [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize]
    }
}

#[cfg(feature = "const-eval")]
impl<T, const N: usize> Index<Usize<0,N>> for [T; N+1] {
    type Output = T;
    fn index(&self, idx: Usize<0,N>) -> &Self::Output {
        let i:usize = idx.into();
        #[cfg(feature = "unsafe-range-assert")]
        unsafe {
            self.get_unchecked(i)
        }
        #[cfg(not(feature = "unsafe-range-assert"))]
        self.get(i).unwrap()
    }
}

#[cfg(feature = "const-eval")]
impl<T, const N: usize> IndexMut<Usize<0,N>> for [T; N+1] {
    fn index_mut(&mut self, idx: Usize<0,N>) -> &mut Self::Output {
        let i:usize = idx.into();
        #[cfg(feature = "unsafe-range-assert")]
        unsafe {
            self.get_unchecked_mut(i)
        }
        #[cfg(not(feature = "unsafe-range-assert"))]
        self.get_mut(i).unwrap()
    }
}

#[cfg(not(feature = "const-eval"))]
macro_rules! impl_array_index {
    ($size:literal, $size_minus_one:literal) => {
        impl<T> Index<Usize<0,$size_minus_one>> for [T; $size] {
            type Output = T;
            fn index(&self, idx: Usize<0,$size_minus_one>) -> &Self::Output {
                let i:usize = unsafe_is_range!(0, $size_minus_one, idx.into());
                #[cfg(feature = "unsafe-range-assert")]
                unsafe {
                    self.get_unchecked(i)
                }
                #[cfg(not(feature = "unsafe-range-assert"))]
                self.get(i).unwrap()
            }
        }
        
        impl<T> IndexMut<Usize<0,$size_minus_one>> for [T; $size] {
            fn index_mut(&mut self, idx: Usize<0,$size_minus_one>) -> &mut Self::Output {
                let i:usize = unsafe_is_range!(0, $size_minus_one, idx.into());
                #[cfg(feature = "unsafe-range-assert")]
                unsafe {
                    self.get_unchecked_mut(i)
                }
                #[cfg(not(feature = "unsafe-range-assert"))]
                self.get_mut(i).unwrap()
            }
        }
    };
}

#[cfg(not(feature = "const-eval"))]
mod array_impls {
    use core::ops::{Index,IndexMut};
    use super::Usize;
    impl_array_index!(1,0);
    impl_array_index!(2,1);
    impl_array_index!(3,2);
    impl_array_index!(4,3);
    impl_array_index!(5,4);
    impl_array_index!(6,5);
    impl_array_index!(7,6);
    impl_array_index!(8,7);
    impl_array_index!(9,8);
    impl_array_index!(10,9);
    impl_array_index!(11,10);
    impl_array_index!(12,11);
    impl_array_index!(13,12);
    impl_array_index!(14,13);
    impl_array_index!(15,14);
    impl_array_index!(16,15);
    impl_array_index!(17,16);
    impl_array_index!(18,17);
    impl_array_index!(19,18);
    impl_array_index!(20,19);
    impl_array_index!(21,20);
    impl_array_index!(22,21);
    impl_array_index!(23,22);
    impl_array_index!(24,23);
    impl_array_index!(25,24);
    impl_array_index!(26,25);
    impl_array_index!(27,26);
    impl_array_index!(28,27);
    impl_array_index!(29,28);
    impl_array_index!(30,29);
    impl_array_index!(31,30);
    impl_array_index!(32,31);
    impl_array_index!(33,32);
    impl_array_index!(34,33);
    impl_array_index!(35,34);
    impl_array_index!(36,35);
    impl_array_index!(37,36);
    impl_array_index!(38,37);
    impl_array_index!(39,38);
    impl_array_index!(40,39);
    impl_array_index!(41,40);
    impl_array_index!(42,41);
    impl_array_index!(43,42);
    impl_array_index!(44,43);
    impl_array_index!(45,44);
    impl_array_index!(46,45);
    impl_array_index!(47,46);
    impl_array_index!(48,47);
    impl_array_index!(49,48);
    impl_array_index!(50,49);
    impl_array_index!(51,50);
    impl_array_index!(52,51);
    impl_array_index!(53,52);
    impl_array_index!(54,53);
    impl_array_index!(55,54);
    impl_array_index!(56,55);
    impl_array_index!(57,56);
    impl_array_index!(58,57);
    impl_array_index!(59,58);
    impl_array_index!(60,59);
    impl_array_index!(61,60);
    impl_array_index!(62,61);
    impl_array_index!(63,62);
    impl_array_index!(64,63);
    impl_array_index!(65,64);
    impl_array_index!(66,65);
    impl_array_index!(67,66);
    impl_array_index!(68,67);
    impl_array_index!(69,68);
    impl_array_index!(70,69);
    impl_array_index!(71,70);
    impl_array_index!(72,71);
    impl_array_index!(73,72);
    impl_array_index!(74,73);
    impl_array_index!(75,74);
    impl_array_index!(76,75);
    impl_array_index!(77,76);
    impl_array_index!(78,77);
    impl_array_index!(79,78);
    impl_array_index!(80,79);
    impl_array_index!(81,80);
    impl_array_index!(82,81);
    impl_array_index!(83,82);
    impl_array_index!(84,83);
    impl_array_index!(85,84);
    impl_array_index!(86,85);
    impl_array_index!(87,86);
    impl_array_index!(88,87);
    impl_array_index!(89,88);
    impl_array_index!(90,89);
    impl_array_index!(91,90);
    impl_array_index!(92,91);
    impl_array_index!(93,92);
    impl_array_index!(94,93);
    impl_array_index!(95,94);
    impl_array_index!(96,95);
    impl_array_index!(97,96);
    impl_array_index!(98,97);
    impl_array_index!(99,98);
    impl_array_index!(100,99);
    impl_array_index!(101,100);
    impl_array_index!(102,101);
    impl_array_index!(103,102);
    impl_array_index!(104,103);
    impl_array_index!(105,104);
    impl_array_index!(106,105);
    impl_array_index!(107,106);
    impl_array_index!(108,107);
    impl_array_index!(109,108);
    impl_array_index!(110,109);
    impl_array_index!(111,110);
    impl_array_index!(112,111);
    impl_array_index!(113,112);
    impl_array_index!(114,113);
    impl_array_index!(115,114);
    impl_array_index!(116,115);
    impl_array_index!(117,116);
    impl_array_index!(118,117);
    impl_array_index!(119,118);
    impl_array_index!(120,119);
    impl_array_index!(121,120);
    impl_array_index!(122,121);
    impl_array_index!(123,122);
    impl_array_index!(124,123);
    impl_array_index!(125,124);
    impl_array_index!(126,125);
    impl_array_index!(127,126);
    impl_array_index!(128,127);
    impl_array_index!(129,128);
    impl_array_index!(130,129);
    impl_array_index!(131,130);
    impl_array_index!(132,131);
    impl_array_index!(133,132);
    impl_array_index!(134,133);
    impl_array_index!(135,134);
    impl_array_index!(136,135);
    impl_array_index!(137,136);
    impl_array_index!(138,137);
    impl_array_index!(139,138);
    impl_array_index!(140,139);
    impl_array_index!(141,140);
    impl_array_index!(142,141);
    impl_array_index!(143,142);
    impl_array_index!(144,143);
    impl_array_index!(145,144);
    impl_array_index!(146,145);
    impl_array_index!(147,146);
    impl_array_index!(148,147);
    impl_array_index!(149,148);
    impl_array_index!(150,149);
    impl_array_index!(151,150);
    impl_array_index!(152,151);
    impl_array_index!(153,152);
    impl_array_index!(154,153);
    impl_array_index!(155,154);
    impl_array_index!(156,155);
    impl_array_index!(157,156);
    impl_array_index!(158,157);
    impl_array_index!(159,158);
    impl_array_index!(160,159);
    impl_array_index!(161,160);
    impl_array_index!(162,161);
    impl_array_index!(163,162);
    impl_array_index!(164,163);
    impl_array_index!(165,164);
    impl_array_index!(166,165);
    impl_array_index!(167,166);
    impl_array_index!(168,167);
    impl_array_index!(169,168);
    impl_array_index!(170,169);
    impl_array_index!(171,170);
    impl_array_index!(172,171);
    impl_array_index!(173,172);
    impl_array_index!(174,173);
    impl_array_index!(175,174);
    impl_array_index!(176,175);
    impl_array_index!(177,176);
    impl_array_index!(178,177);
    impl_array_index!(179,178);
    impl_array_index!(180,179);
    impl_array_index!(181,180);
    impl_array_index!(182,181);
    impl_array_index!(183,182);
    impl_array_index!(184,183);
    impl_array_index!(185,184);
    impl_array_index!(186,185);
    impl_array_index!(187,186);
    impl_array_index!(188,187);
    impl_array_index!(189,188);
    impl_array_index!(190,189);
    impl_array_index!(191,190);
    impl_array_index!(192,191);
    impl_array_index!(193,192);
    impl_array_index!(194,193);
    impl_array_index!(195,194);
    impl_array_index!(196,195);
    impl_array_index!(197,196);
    impl_array_index!(198,197);
    impl_array_index!(199,198);
    impl_array_index!(200,199);
    impl_array_index!(201,200);
    impl_array_index!(202,201);
    impl_array_index!(203,202);
    impl_array_index!(204,203);
    impl_array_index!(205,204);
    impl_array_index!(206,205);
    impl_array_index!(207,206);
    impl_array_index!(208,207);
    impl_array_index!(209,208);
    impl_array_index!(210,209);
    impl_array_index!(211,210);
    impl_array_index!(212,211);
    impl_array_index!(213,212);
    impl_array_index!(214,213);
    impl_array_index!(215,214);
    impl_array_index!(216,215);
    impl_array_index!(217,216);
    impl_array_index!(218,217);
    impl_array_index!(219,218);
    impl_array_index!(220,219);
    impl_array_index!(221,220);
    impl_array_index!(222,221);
    impl_array_index!(223,222);
    impl_array_index!(224,223);
    impl_array_index!(225,224);
    impl_array_index!(226,225);
    impl_array_index!(227,226);
    impl_array_index!(228,227);
    impl_array_index!(229,228);
    impl_array_index!(230,229);
    impl_array_index!(231,230);
    impl_array_index!(232,231);
    impl_array_index!(233,232);
    impl_array_index!(234,233);
    impl_array_index!(235,234);
    impl_array_index!(236,235);
    impl_array_index!(237,236);
    impl_array_index!(238,237);
    impl_array_index!(239,238);
    impl_array_index!(240,239);
    impl_array_index!(241,240);
    impl_array_index!(242,241);
    impl_array_index!(243,242);
    impl_array_index!(244,243);
    impl_array_index!(245,244);
    impl_array_index!(246,245);
    impl_array_index!(247,246);
    impl_array_index!(248,247);
    impl_array_index!(249,248);
    impl_array_index!(250,249);
    impl_array_index!(251,250);
    impl_array_index!(252,251);
    impl_array_index!(253,252);
    impl_array_index!(254,253);
    impl_array_index!(255,254);
    impl_array_index!(256,255);
}
