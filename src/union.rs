use std::any::TypeId;

use select::{TypeSelect, TypeUnion};

macro_rules! doc_comment {
    ($x:expr, $($tt:tt)*) => {
        #[doc = $x]
        $($tt)*
    };
}

/// Returns the TypeId of `T`
pub(crate) const fn type_id<T: 'static>() -> TypeId {
    TypeId::of::<T>()
}

#[macro_export]
macro_rules! contains_type {
    ($T:ty, [$($O:ty),*]) => (
        false $(|| type_id::<$T>() == type_id::<$O>())*
    )
}

macro_rules! Union {
    (   $d:tt,
        pub union $name:ident {
        $($fieldnames:ident: $generics:tt),*
    }) => (
        doc_comment!(
            concat!("This union can hold the following Generics: ", stringify!($($generics),*)),
            #[derive(Copy, Clone)]
            pub union $name<$($generics),*> {
                $($fieldnames: $generics,)*
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
        );
    )
}

macro_rules! GenUnion {
    (
        NAMES = [],
        FIELDS = [$fieldname:ident:$generic:tt]
    ) => ();

    (
        NAMES = [$name:ident $(,$names:ident)*],
        FIELDS = [$fieldname:ident: $generic:ident $(,$fieldnames:ident: $generics:ident)*]
    ) => {
        Union!(
            "This union can hold the following Generics: ",
            pub union $name {
            $fieldname: $generic
            $(, $fieldnames: $generics)*
        });

        GenUnion!(
            NAMES = [$($names),*],
            FIELDS = [$($fieldnames: $generics),*]
        );
    };
}

GenUnion!(
    NAMES = [Union10, Union9, Union8, Union7, Union6, Union5, Union4, Union3, Union2],
    FIELDS = [a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J]
);
