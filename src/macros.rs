#[macro_export]
#[doc(hidden)]
macro_rules! bd {
    ($value:expr) => {
        ::bigdecimal::BigDecimal::try_from($value).unwrap()
    };
}
