macro_rules! quantity_newtype {
    ($(#[$meta:meta])* $name:ident, $inner:ty, $getter:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $name($inner);

        impl $name {
            /// Wraps a raw value.
            pub const fn new(value: $inner) -> Self {
                Self(value)
            }

            /// Returns the raw value.
            pub const fn $getter(self) -> $inner {
                self.0
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }
    };
}

quantity_newtype!(
    /// Dose rate in Roentgen per hour, as reported on the wire.
    DoseRateRh,
    f32,
    as_f32
);
quantity_newtype!(
    /// Count rate in counts per second.
    CountRateCps,
    f32,
    as_f32
);
quantity_newtype!(
    /// Accumulated dose in Roentgen, as reported on the wire.
    DoseRoentgen,
    f32,
    as_f32
);
quantity_newtype!(
    /// Dose-rate alarm threshold register value (µR/h, or 0.01 µSv/h).
    RawMicroRoentgenPerHour,
    u32,
    as_u32
);
quantity_newtype!(
    /// Count-rate alarm threshold register value (counts per 10 s).
    RawCountsPer10s,
    u32,
    as_u32
);
