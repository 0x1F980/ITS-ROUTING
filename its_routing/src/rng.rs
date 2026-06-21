//! Platform RNG for routing — hardware TRNG when enabled, else /dev/urandom.

use std::io::Read;
use its_transport::SecureRandom;

pub struct RoutingRng;

impl SecureRandom for RoutingRng {
    type Error = std::io::Error;

    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        #[cfg(feature = "hardware")]
        {
            return its_hardware::CliRng.fill_bytes(dest);
        }
        #[cfg(not(feature = "hardware"))]
        {
            std::fs::File::open("/dev/urandom")?.read_exact(dest)
        }
    }
}
