
pub struct AppError {
    pub code : u8,
}


macro_rules! def_error {
    ($err_t:ty, $err_no:expr) => {
        impl From<$err_t> for AppError {
            fn from(_:$err_t) -> Self {
                AppError {
                    code: $err_no,
                }
            }
        }
    }
}

def_error!(embedded_nrf24l01::Error<stm32f1xx_hal::spi::Error>, 1);
def_error!(core::convert::Infallible, 2);
