use dynamic_pool::DynamicPool;
pub use dynamic_pool::{DynamicPoolItem, DynamicReset};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SizedPoolError {
    #[error("the given size exceeds the maximum allowed size of the pool")]
    SizeExceedMaxSize,
}

pub trait SizedAllocatable {
    fn new(size: usize) -> Self;
    fn size(&self) -> usize;
}

pub struct SizedPool<T: SizedAllocatable + DynamicReset> {
    /// each entry represents an allocation queue of 2**n bytes block
    sub_pools: Vec<DynamicPool<T>>,
}

impl<T: SizedAllocatable + DynamicReset> SizedPool<T> {
    /// cap is the capacity of each subpool, pool_size_power_of_two is the number of subpools,
    /// init_fn takes the pool_size (the power of two) as input and outputs the reusable resource
    pub fn new(cap: usize, pool_size_power_of_two: u32, max_pool_size: usize) -> Self {
        let mut pools = Vec::new();
        for pool_power in 0..pool_size_power_of_two {
            let pool =
                DynamicPool::new(cap, max_pool_size, move || T::new(2_usize.pow(pool_power)));
            pools.push(pool);
        }
        Self { sub_pools: pools }
    }

    fn get_subpool_location(&self, size: usize) -> usize {
        (size.next_power_of_two().trailing_zeros()) as usize
    }

    fn get_subpool(&self, size: usize) -> Result<&DynamicPool<T>, SizedPoolError> {
        self.sub_pools
            .get(self.get_subpool_location(size))
            .ok_or(SizedPoolError::SizeExceedMaxSize)
    }

    pub fn try_pull(&self, size: usize) -> Result<Option<DynamicPoolItem<T>>, SizedPoolError> {
        Ok(self.get_subpool(size)?.try_take())
    }
}
