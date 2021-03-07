use object_pool::Pool;
pub use object_pool::Reusable;
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

pub struct SizedPool<T: SizedAllocatable> {
    /// each entry represents an allocation queue of 2**n bytes block
    sub_pools: Vec<Pool<T>>,
}

impl<T: SizedAllocatable> SizedPool<T> {
    /// cap is the capacity of each subpool, pool_size_power_of_two is the number of subpools,
    /// init_fn takes the pool_size (the power of two) as input and outputs the reusable resource
    pub fn new(cap: usize, pool_size_power_of_two: u32) -> Self {
        let mut pools = Vec::new();
        for pool_power in 0..pool_size_power_of_two {
            let pool = Pool::new(cap, || T::new(2_usize.pow(pool_power)));
            pools.push(pool);
        }
        Self { sub_pools: pools }
    }

    fn get_subpool_location(&self, size: usize) -> usize {
        (size.next_power_of_two().trailing_zeros()) as usize
    }

    fn get_subpool(&self, size: usize) -> Result<&Pool<T>, SizedPoolError> {
        self.sub_pools
            .get(self.get_subpool_location(size))
            .ok_or(SizedPoolError::SizeExceedMaxSize)
    }

    pub fn try_pull(&self, size: usize) -> Result<Option<Reusable<T>>, SizedPoolError> {
        Ok(self.get_subpool(size)?.try_pull())
    }

    pub fn pull(&self, size: usize) -> Result<Reusable<T>, SizedPoolError> {
        let reusable = self.try_pull(size)?;
        match reusable {
            None => Ok(Reusable::new(
                self.get_subpool(size)?,
                T::new(2_usize.pow(self.get_subpool_location(size) as u32)),
            )),
            Some(x) => Ok(x),
        }
    }

    pub fn attach(&self, t: T) -> Result<(), SizedPoolError> {
        self.get_subpool(t.size())?.attach(t);
        Ok(())
    }
}
