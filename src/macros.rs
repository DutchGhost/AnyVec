use selectvec::{TypeSelect, TypeUnion, type_id, Select, Selector};

#[macro_export]
macro_rules! contains_type {
    ($T:ty, [$($O:ty),*]) => (
        false $(|| type_id::<$T>() == type_id::<$O>())*
    )
}

macro_rules! select {
    ([$($names:ident),*] => [$($generics:tt),*]) => (
        $(
            #[derive(Debug, Ord, PartialOrd, Hash, Eq, PartialEq, Default)]
            pub struct $names;

            impl Selector for $names {}
        )*

        select!(@INNER: [$($names),*] => $($generics),*);
    );

    (@INNER: [$name:ident $(,$names:ident)*] => $output:tt $(,$generics:tt)*) => (
        // impl<$output $(,$generics),*> Select<$name> for <$output, $($generics)*> {
        //     type Output = $output;
        // }

        select!(@IMPL $name => [$output $(,$generics)* ]);
        select!(@INNER: [$($names)*] => $($generics)*);
    );

    (@IMPL $name:ident => [$output:tt $(,$generic:tt)*]) => (
        impl <$output, $($generic),*> Select<$name> for ($output, $($generic),*)
        where
            $output: 'static
        {
            type Output = $output;
        }
    )
}

select!([A, B] => [AA, BB]);

macro_rules! Union {
    (pub union $name:ident {
        $($varname:ident: $generics:tt,)*
    }) => (
        pub union $name<$($generics),*> {
            $($varname: $generics,)*
        }

        impl <$($generics),*> TypeUnion for ($($generics),*)
        where
            $($generics: 'static),*
        {
            type Union = $name<$($generics),*>;

            #[inline]
            fn contains<T: 'static>() -> bool {
                contains_type!(T, [$($generics),*])
            }
        }

        unsafe impl <$($generics),*> TypeSelect<($($generics),*)> for $name<$($generics),*>
        where
            $($generics: 'static),*
        {}
    )
}

Union!(pub union Union2 {
    a: A,
    b: B,
});

Union!(pub union Union4 {
    a: A,
    b: B,
    c: C,
    d: D,
});

Union!(pub union Union5 {
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
});

Union!(pub union Union6 {
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
});

Union!(pub union Union7 {
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
    g: G,
});

Union!(pub union Union8 {
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
    g: G,
    h: H,
});

Union!(pub union Union9 {
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
    g: G,
    h: H,
    i: I,
});