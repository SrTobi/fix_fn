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
                let $self_arg = |$($arg_name : $arg_type)*| $self_arg.call($($arg_name ,)*);
                {
                    $body
                }
            }
        );


        #[inline]
        move |$($arg_name : $arg_type)*| -> $ret_type {
            inner.call($($arg_name)*)
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
        let x = 2;

        let f = fix_fn!(|f, i: i32| -> i32 {
            if i == 0 {
                0
            } else {
                x + f(i - 1)
            }
        });

        let t = 4;
        assert_eq!(f(t), t * x);
    }

    #[test]
    fn test_two_parameter() {
        let fib = fix_fn!(|fib, i: u32| -> u32 {
            if i <= 1 {
                i
            } else {
                fib(i - 1) + fib(i - 2)
            }
        });

        assert_eq!(fib(7), 13);
    }
}
