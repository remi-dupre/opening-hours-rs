pub mod display;
pub mod normalize;
pub mod paving;

macro_rules! ex {
    ( $( $tt: expr ),* $( , )? ) => {
        (file!(), line!() $( , $tt )*)
    };
}

pub(crate) use ex;
