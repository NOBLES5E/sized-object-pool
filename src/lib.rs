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

    pub fn try_pull(&self, size: usize) -> Result<DynamicPoolItem<T>, SizedPoolError> {
        let pool = self.get_subpool(size)?;
        match pool.try_take() {
            None => {
                tracing::debug!("not enough items in pool, allocating");
                Ok(pool.take())
            }
            Some(x) => Ok(x),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug)]
    struct TestItem {
        size: usize,
    }

    impl SizedAllocatable for TestItem {
        fn new(size: usize) -> Self {
            Self { size }
        }

        fn size(&self) -> usize {
            self.size
        }
    }

    impl DynamicReset for TestItem {
        fn reset(&mut self) {}
    }

    #[test]
    fn test_allocate() {
        let pool: SizedPool<TestItem> = SizedPool::new(0, 40, 1024);
        let mut items = Vec::new();
        for _ in 0..2048 {
            items.push(pool.try_pull(10).unwrap());
        }
    }
}
