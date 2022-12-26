use core::convert::Infallible;

use embedded_hal::digital::v2::toggleable;
use embedded_hal::digital::v2::{InputPin, IoPin, OutputPin, PinState, StatefulOutputPin};

use super::{Floating, Input, OpenDrain, Output, Pin, PullDown, PullUp, PushPull};

impl<const P: char, const N: u8, MODE> OutputPin for Pin<P, N, Output<MODE>> {
    type Error = Infallible;

    #[inline(always)]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    #[inline(always)]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl<const P: char, const N: u8, MODE> StatefulOutputPin for Pin<P, N, Output<MODE>> {
    #[inline(always)]
    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_high())
    }

    #[inline(always)]
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_set_low())
    }
}

/// Opt-in to the software implementation.
impl<const P: char, const N: u8, MODE> toggleable::Default for Pin<P, N, Output<MODE>> {}

impl<const P: char, const N: u8> InputPin for Pin<P, N, Output<OpenDrain>> {
    type Error = Infallible;

    #[inline(always)]
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_high())
    }

    #[inline(always)]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_low())
    }
}

impl<const P: char, const N: u8, MODE> InputPin for Pin<P, N, Input<MODE>> {
    type Error = Infallible;

    #[inline(always)]
    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.is_high())
    }

    #[inline(always)]
    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.is_low())
    }
}

impl<const P: char, const N: u8> IoPin<Self, Self> for Pin<P, N, Output<OpenDrain>> {
    type Error = Infallible;
    fn into_input_pin(self) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn into_output_pin(mut self, state: PinState) -> Result<Self, Self::Error> {
        self.set_state(state);
        Ok(self)
    }
}

impl<const P: char, const N: u8> IoPin<Pin<P, N, Input<Floating>>, Self>
    for Pin<P, N, Output<OpenDrain>>
{
    type Error = Infallible;
    fn into_input_pin(self) -> Result<Pin<P, N, Input<Floating>>, Self::Error> {
        Ok(self.into_floating_input())
    }
    fn into_output_pin(mut self, state: PinState) -> Result<Self, Self::Error> {
        self.set_state(state);
        Ok(self)
    }
}

impl<const P: char, const N: u8> IoPin<Self, Pin<P, N, Output<OpenDrain>>>
    for Pin<P, N, Input<Floating>>
{
    type Error = Infallible;
    fn into_input_pin(self) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn into_output_pin(self, state: PinState) -> Result<Pin<P, N, Output<OpenDrain>>, Self::Error> {
        Ok(self.into_open_drain_output_in_state(state))
    }
}

impl<const P: char, const N: u8> IoPin<Pin<P, N, Input<Floating>>, Self>
    for Pin<P, N, Output<PushPull>>
{
    type Error = Infallible;
    fn into_input_pin(self) -> Result<Pin<P, N, Input<Floating>>, Self::Error> {
        Ok(self.into_floating_input())
    }
    fn into_output_pin(mut self, state: PinState) -> Result<Self, Self::Error> {
        self.set_state(state);
        Ok(self)
    }
}

impl<const P: char, const N: u8> IoPin<Self, Pin<P, N, Output<PushPull>>>
    for Pin<P, N, Input<Floating>>
{
    type Error = Infallible;
    fn into_input_pin(self) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn into_output_pin(self, state: PinState) -> Result<Pin<P, N, Output<PushPull>>, Self::Error> {
        Ok(self.into_push_pull_output_in_state(state))
    }
}

impl<const P: char, const N: u8> IoPin<Pin<P, N, Input<PullUp>>, Self>
    for Pin<P, N, Output<PushPull>>
{
    type Error = Infallible;
    fn into_input_pin(self) -> Result<Pin<P, N, Input<PullUp>>, Self::Error> {
        Ok(self.into_pull_up_input())
    }
    fn into_output_pin(mut self, state: PinState) -> Result<Self, Self::Error> {
        self.set_state(state);
        Ok(self)
    }
}

impl<const P: char, const N: u8> IoPin<Self, Pin<P, N, Output<PushPull>>>
    for Pin<P, N, Input<PullUp>>
{
    type Error = Infallible;
    fn into_input_pin(self) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn into_output_pin(self, state: PinState) -> Result<Pin<P, N, Output<PushPull>>, Self::Error> {
        Ok(self.into_push_pull_output_in_state(state))
    }
}

impl<const P: char, const N: u8> IoPin<Pin<P, N, Input<PullDown>>, Self>
    for Pin<P, N, Output<PushPull>>
{
    type Error = Infallible;
    fn into_input_pin(self) -> Result<Pin<P, N, Input<PullDown>>, Self::Error> {
        Ok(self.into_pull_down_input())
    }
    fn into_output_pin(mut self, state: PinState) -> Result<Self, Self::Error> {
        self.set_state(state);
        Ok(self)
    }
}

impl<const P: char, const N: u8> IoPin<Self, Pin<P, N, Output<PushPull>>>
    for Pin<P, N, Input<PullDown>>
{
    type Error = Infallible;
    fn into_input_pin(self) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn into_output_pin(self, state: PinState) -> Result<Pin<P, N, Output<PushPull>>, Self::Error> {
        Ok(self.into_push_pull_output_in_state(state))
    }
}
