use select::{Select, Selector};

macro_rules! IndexStruct {
    ($name: ident $(, $names:ident)*) => (
        /// This type is used to 'index' into a tuple of generics.
        /// See [`Select`] what Generic it selects.
        #[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
        pub struct $name;

        impl Selector for $name {}

        IndexStruct!($($names),*);
    );

    () => ();
}

IndexStruct!(A, B, C, D, E, F, G, H, I, J);

macro_rules! impl_select {
    (
        NAMES = [$name: ident $(,$names:ident)*],
        GENERICS = [$current:tt $(,$generics:tt)*],
        COPIES = [$($copies:tt),*]
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
            COPIES = [$($copies),*]
        );
    );

    (
        NAMES = [],
        GENERICS = [],
        COPIES = [$($copies:tt),*]
    ) => ();
}

macro_rules! select {
    (
        NAMES = [$($names:ident),*],
        GENERICS = [$($generics:tt),*]
    ) => (
        select!(
            NAMES = [$($names),*],
            GENERICS = [$($generics),*],
            ALL_GENERICS = [$($generics),*]
        );
    );

    (
        NAMES = [$($names:ident),*],
        GENERICS = [$current:tt $(,$gens:tt)*],
        ALL_GENERICS = [$($all_generics:tt),*]
    ) => (
        impl_select!(
            NAMES = [$($names),*],
            GENERICS = [$current $(,$gens)*],
            COPIES = [$($all_generics),*]
        );
    );
}

/*
 *
 * s = 'ABCDEFGHIJ'
 * for (idx, letter) in enumerate(s[1:], start = 2):
 *      names = ' '.join(["NAMES", "=", '[', ', '.join(s[:idx]), ']' ])
 *      generics = ' '.join(["GENERICS", "=", '[', ', '.join([letter * 2 for letter in s[:idx]]), ']' ])
 *
 *      totall = ''.join(["select!(", names, ', ', generics, ');'])
 *      print(totall)
 */

select!(NAMES = [A, B], GENERICS = [AA, BB]);
select!(NAMES = [A, B, C], GENERICS = [AA, BB, CC]);
select!(NAMES = [A, B, C, D], GENERICS = [AA, BB, CC, DD]);
select!(NAMES = [A, B, C, D, E], GENERICS = [AA, BB, CC, DD, EE]);
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
