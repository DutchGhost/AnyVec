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

IndexStruct!(Type1, Type2, Type3, Type4, Type5, Type6, Type7, Type8, Type9, Type10);

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

select!(NAMES = [Type1, Type2], GENERICS = [A, B]);
select!(NAMES = [Type1, Type2, Type3], GENERICS = [A, B, C]);
select!(
    NAMES = [Type1, Type2, Type3, Type4],
    GENERICS = [A, B, C, D]
);
select!(
    NAMES = [Type1, Type2, Type3, Type4, Type5],
    GENERICS = [A, B, C, D, E]
);
select!(
    NAMES = [Type1, Type2, Type3, Type4, Type5, Type6],
    GENERICS = [A, B, C, D, E, F]
);
select!(
    NAMES = [Type1, Type2, Type3, Type4, Type5, Type6, Type7],
    GENERICS = [A, B, C, D, E, F, G]
);
select!(
    NAMES = [Type1, Type2, Type3, Type4, Type5, Type6, Type7, Type8],
    GENERICS = [A, B, C, D, E, F, G, H]
);
select!(
    NAMES = [Type1, Type2, Type3, Type4, Type5, Type6, Type7, Type8, Type9],
    GENERICS = [A, B, C, D, E, F, G, H, I]
);
select!(
    NAMES = [Type1, Type2, Type3, Type4, Type5, Type6, Type7, Type8, Type9, Type10],
    GENERICS = [A, B, C, D, E, F, G, H, I, J]
);
