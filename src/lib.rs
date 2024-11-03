//! This library enables the creation of recursive closures by providing a
//! single macro [`fix_fn`]. The functionality is similar to the
//! [Y combinator](https://en.wikipedia.org/wiki/Fixed-point_combinator#Fixed-point_combinators_in_lambda_calculus).
//! Recursive closures can have arbitrary amounts of parameters and can capture
//! variables.
//!
//! ```
//! use fix_fn::fix_fn;
//!
//! let fib = fix_fn!(|fib, i: u32| -> u32 {
//!     if i <= 1 {
//!            i
//!      } else {
//!          // fib will call the closure recursively
//!          fib(i - 1) + fib(i - 2)
//!      }
//!  });
//!
//! assert_eq!(fib(7), 13);
//! ```
//!
//! The generated code is not completely abstraction free as it uses one dyn trait
//! (without any boxing) to overcome rust's recursive type limitations.
//! In most cases, however, the optimizer should be able to eliminate any dynamic dispatch.
//!
//! Unfortunately, mutable recursive closures are not supported.

/// Takes a closure definition where the first parameter will be a [`Fn`] to the closure itself.
/// Returns a recursive closure with the same signature, except the first parameter will be
/// eliminated.
///
/// The passed closure needs to have at least one parameter. This
/// first parameter can be used to call the closure itself, achieving recursion.
/// It must not be annotated with a type.
///
/// Additional parameters will be parameters of the resulting closure.
/// All additional parameters must be annotated with types.
///
/// The closure definition needs to have a result-type annotation.
///
/// `move` can be used and has the [usual semantic](https://doc.rust-lang.org/1.18.0/book/first-edition/closures.html#move-closures).
///
/// # Example
///
/// ```
/// use fix_fn::fix_fn;
///  
/// let fib = fix_fn!(|fib, i: u32| -> u32 {
///     if i <= 1 {
///         i
///     } else {
///         // fib will call the closure recursively
///         fib(i - 1) + fib(i - 2)
///     }
/// });
///
/// // resulting lambda only has the `i: u32` parameter
/// assert_eq!(fib(7), 13);
/// ```
#[macro_export]
macro_rules! fix_fn {
    (
        $($mov:ident)? |$self_arg:ident $(, $arg_name:ident : $arg_type:ty)* $(,)? |
            -> $ret_type:ty
        $body:block
    ) => {{
        trait HideFn {
            fn call(&self, $($arg_name : $arg_type ,)*) -> $ret_type;
        }

        struct HideFnImpl<F: Fn(&dyn HideFn, $($arg_type ,)*) -> $ret_type>(F);

        impl<F: Fn(&dyn HideFn, $($arg_type ,)*) -> $ret_type> HideFn for HideFnImpl<F> {
            #[inline]
            fn call(&self, $($arg_name : $arg_type ,)*) -> $ret_type {
                self.0(self, $($arg_name ,)*)
            }
        }

        let inner = HideFnImpl(
            #[inline]
            $($mov)?
            |$self_arg, $($arg_name : $arg_type ,)*| -> $ret_type {
                let $self_arg = |$($arg_name : $arg_type ),*| $self_arg.call($($arg_name ,)*);
                {
                    $body
                }
            }
        );


        #[inline]
        move |$($arg_name : $arg_type),*| -> $ret_type {
            inner.call($($arg_name),*)
        }
    }};
    (
        $($mov:ident)? |$($arg_name:ident $(: $arg_type:ty)?),* $(,)?|
        $body:expr
    ) => {
        compile_error!("Closure passed to fix_fn needs return type!");
    };
    (
        $($mov:ident)? |$self_arg:ident : $self_type:ty $(, $arg_name:ident $(: $arg_type:ty)?)* $(,)? |
            -> $ret_type:ty
        $body:block
    ) => {
        compile_error!(concat!("First parameter ", stringify!($self_arg), " may not have type annotation!"));
    };
    (
        $($mov:ident)? |$self_arg:ident $(, $arg_name:ident $(: $arg_type:ty)?)* $(,)? |
            -> $ret_type:ty
        $body:block
    ) => {
        compile_error!("All parameters except first need to have an explicit type annotation!");
    };
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    #[test]
    fn test_zero_parameter() {
        fn create() -> impl Fn() -> i32 {
            let cell = RefCell::new(0);

            fix_fn!(move |rec| -> i32 {
                if *cell.borrow() == 10 {
                    10
                } else {
                    *cell.borrow_mut() += 1;
                    rec()
                }
            })
        }

        let f = create();

        assert_eq!(f(), 10);
    }

    #[test]
    fn test_one_parameter() {
        let fib = fix_fn!(|fib, i: u32| -> u32 {
            if i <= 1 {
                i
            } else {
                fib(i - 1) + fib(i - 2)
            }
        });

        assert_eq!(fib(7), 13);
    }

    #[test]
    fn test_two_parameter() {
        let pow = fix_fn!(|pow, x: u32, y: u32| -> u32 {
            if y == 1 {
                x
            } else if x % 2 == 0 {
                pow(x * x, x / 2)
            } else {
                x * pow(x, y - 1)
            }
        });

        assert_eq!(pow(3, 9), 19683);
    }
}
