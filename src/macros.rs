use selectvec::{TypeSelect, TypeUnion, type_id, Select, Selector};

#[macro_export]
macro_rules! contains_type {
    ($T:ty, [$($O:ty),*]) => (
        false $(|| type_id::<$T>() == type_id::<$O>())*
    )
}

macro_rules! Zstruct {
    ($name:ident $(,$names:ident)*) => (
        #[derive(Debug, Ord, PartialOrd, Hash, Eq, PartialEq, Default)]
        pub struct $name;

        impl Selector for $name {}

        Zstruct!($($names),*);
    );

    () => ();
}

macro_rules! impl_select {
    (
        NAMES = [$name:ident $(,$names:ident)*],
        GENERICS = [$current:tt $(,$generics:tt)*],
        COPY = [$($copies:tt),*]
    ) => (
        impl <$($copies),*> Select<$name> for ($($copies),*)
        where
            $current: 'static
        {
            type Output = $current;
        }

        impl_select!(
            NAMES = [$($names),*],
            GENERICS = [$($generics),*],
            COPY = [$($copies),*]
        );
    );

    (
        NAMES = [],
        GENERICS = [],
        COPY = [$($copies:tt),*]
    ) => ();
}
macro_rules! select {
    //Take all names, and generate the structs directly.
    (
        NAMES = [$($names:ident),*],
        GENERICS = [$($generics:tt),*]
    ) => (

        //Generate the structs.
        //Zstruct!($($names),*);

        // Calls self with a copy of the generics,
        select!(
            NAMES = [$($names),*],
            GENERICS = [$($generics),*],
            ALL_GENERICS = [$($generics),*]
        );
    );

    //takes all names, knowing the current name,
    //takes all generics, knowing the current generic,
    //takes also a copy of all generics.
    (
        NAMES = [$($names:ident),*],
        GENERICS = [$current:tt $(,$gens:tt)*],
        ALL_GENERICS = [$($all_generics:tt),*]
    ) => (
        impl_select!(
            NAMES = [$($names),*],
            GENERICS = [$current $(,$gens)*],
            COPY = [$($all_generics),*]
        );
    );
}

Zstruct!(A, B, C, D, E, F, G, H, I, J);

select!(
    NAMES = [A, B],
    GENERICS = [AA, BB]
);

select!(
    NAMES = [A, B, C],
    GENERICS = [AA, BB, CC]
);

select!(
    NAMES = [A, B, C, D],
    GENERICS = [AA, BB, CC, DD]
);

select!(
    NAMES = [A, B, C, D, E],
    GENERICS = [AA, BB, CC, DD, EE]
);

select!(
    NAMES = [A, B, C, D, E, F],
    GENERICS = [AA, BB, CC, DD, EE, FF]
);

select!(
    NAMES = [A, B, C, D, E, F, G],
    GENERICS = [AA, BB, CC, DD, EE, FF, GG]
);

select!(
    NAMES = [A, B, C, D, E, F, G, H],
    GENERICS = [AA, BB, CC, DD, EE, FF, GG, HH]
);

select!(
    NAMES = [A, B, C, D, E, F, G, H, I],
    GENERICS = [AA, BB, CC, DD, EE, FF, GG, HH, II]
);

select!(
    NAMES = [A, B, C, D, E, F, G, H, I, J],
    GENERICS = [AA, BB, CC, DD, EE, FF, GG, HH, II, JJ]
);


macro_rules! Union {
    (pub union $name:ident {
        $($varname:ident: $generics:tt),*
    }) =>  (
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

macro_rules! GenUnion {
    //We always have 1 name and generic left.
    (
        NAMES = [],
        FIELDS = [$varname:ident:$generic:tt]
    ) => ();

    (
        NAMES = [$name:ident $(,$names:ident)*],
        FIELDS = [$varname:ident: $generic:ident $(,$varnames:ident: $generics:ident)*]
    ) => {
        Union!(pub union $name {
            $varname: $generic
            $(, $varnames: $generics)*
        });

        GenUnion!(
            NAMES = [$($names),*],
            FIELDS = [$($varnames: $generics),*]
        );
    };
}

GenUnion!(
    NAMES = [Union10, Union9, Union8, Union7, Union6, Union5, Union4, Union3, Union2],
    FIELDS = [a: A,    b: B,   c: C,   d: D,   e: E,   f: F,   g: G,   h: H,   i: I,   j: J]
);

// Union!(pub union Union2 {
//     a: A,
//     b: B,
// });

// Union!(pub union Union4 {
//     a: A,
//     b: B,
//     c: C,
//     d: D,
// });

// Union!(pub union Union5 {
//     a: A,
//     b: B,
//     c: C,
//     d: D,
//     e: E,
// });

// Union!(pub union Union6 {
//     a: A,
//     b: B,
//     c: C,
//     d: D,
//     e: E,
//     f: F,
// });

// Union!(pub union Union7 {
//     a: A,
//     b: B,
//     c: C,
//     d: D,
//     e: E,
//     f: F,
//     g: G,
// });

// Union!(pub union Union8 {
//     a: A,
//     b: B,
//     c: C,
//     d: D,
//     e: E,
//     f: F,
//     g: G,
//     h: H,
// });

// Union!(pub union Union9 {
//     a: A,
//     b: B,
//     c: C,
//     d: D,
//     e: E,
//     f: F,
//     g: G,
//     h: H,
//     i: I,
// });