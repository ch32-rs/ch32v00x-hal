//! Universal Synchronous Asynchronous Receiver Transmitter (USART)

use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ops::DerefMut;
use core::pin::Pin;
use core::ptr;

use crate::hal::prelude::*;
use crate::hal::serial;
use crate::pac;
use crate::rcc::{BusClock, Enable, Reset};
use crate::state;
use nb::block;

use crate::pac::{RCC, UART4, UART5, UART7, USART1, USART2, USART3};

use crate::gpio::{self, Alternate};

use crate::rcc::Clocks;
use crate::{BitsPerSecond, U32Ext};
