use flutter_rust_bridge::rust2dart::IntoIntoDart;
use flutter_rust_bridge::IntoDart;
use flutter_rust_bridge::StreamSink;

pub struct ClosingStreamSink<T>(StreamSink<T>)
where
    T: IntoDart;

impl<T> ClosingStreamSink<T>
where
    T: IntoDart,
{
    pub fn add<D>(&self, value: T)
    where
        D: IntoDart,
        T: IntoIntoDart<D>,
    {
        _ = self.0.add(value);
    }
}

impl<T> From<StreamSink<T>> for ClosingStreamSink<T>
where
    T: IntoDart,
{
    fn from(value: StreamSink<T>) -> Self {
        ClosingStreamSink(value)
    }
}

impl<T> Drop for ClosingStreamSink<T>
where
    T: IntoDart,
{
    fn drop(&mut self) {
        self.0.close();
    }
}
