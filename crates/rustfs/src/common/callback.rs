use std::marker::PhantomData;

/// This type offers a way of passing a callback into a trait function and forcing all
/// implementations of that trait function to actually call the callback.
///
/// Example
/// -------
/// ```
/// # use cryfs_rustfs::Callback;
///
/// trait MyTrait {
///   fn func<R, C: Callback<i32, R>>(&self, callback: C) -> R;
/// }
///
/// struct MyStruct {}
/// impl MyTrait for MyStruct {
///   fn func<R, C: Callback<i32, R>>(&self, callback: C) -> R {
///     // The compiler forces us to call callback because we don't know what
///     // type `R` is and the only way for us to create an instance is to call
///     // the callback.
///     callback.call(42)
///   }
/// }
/// ```
pub trait Callback<T, R> {
    fn call(self, v: T) -> R;
}

/// Instantiate a callback to call a function from a trait taking a [Callback] parameter.
///
/// Example
/// -------
/// ```
/// # use cryfs_rustfs::Callback;
///
/// trait MyTrait {
///   fn func<R, C: Callback<i32, R>>(&self, callback: C) -> R;
/// }
///
/// fn some_func(obj: impl MyTrait) {
///   obj.func(CallbackImpl::new(|v| {
///     println!("The value is {}", v);
///   }))
/// }
/// ```
pub struct CallbackImpl<T, F>
where
    F: FnOnce(T) -> (),
{
    f: F,
    _phantom: PhantomData<T>,
}

impl<T, F> CallbackImpl<T, F>
where
    F: FnOnce(T) -> (),
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: PhantomData,
        }
    }
}

impl<T, F> Callback<T, ()> for CallbackImpl<T, F>
where
    F: FnOnce(T) -> (),
{
    fn call(self, v: T) {
        (self.f)(v)
    }
}
