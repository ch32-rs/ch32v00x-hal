use core::convert::Infallible;
use embedded_hal_1::digital::{ErrorType, InputPin, OutputPin, StatefulOutputPin };
use super::{Input, OpenDrain, Output, Pin};

impl<const P: char, const N: u8, MODE> ErrorType for Pin<P, N, Input<MODE>> {
    type Error = Infallible;
}

impl<const P: char, const N: u8, MODE> InputPin for Pin<P, N, Input<MODE>> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

impl<const P: char, const N: u8, MODE> ErrorType for Pin<P, N, Output<MODE>> {
    type Error = Infallible;
}

impl<const P: char, const N: u8, MODE> OutputPin for Pin<P, N, Output<MODE>> {
    #[inline]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_high())
    }

    #[inline]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.set_low())
    }
}

impl<const P: char, const N: u8, MODE> StatefulOutputPin for Pin<P, N, Output<MODE>> {
    #[inline]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_high())
    }

    /// Is the output pin set as low?
    #[inline]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_set_low())
    }
}

impl<const P: char, const N: u8> InputPin for Pin<P, N, Output<OpenDrain>> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_high())
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((*self).is_low())
    }
}

