use windows::{
    Web::Http::HttpProgress,
    core::{Interface, Result},
};
use windows_core::{GUID, HSTRING, IInspectable, IUnknown};
use windows_future::{AsyncStatus, IAsyncInfo, IAsyncOperation, IAsyncOperationWithProgress};

use crate::{bindings, interfaces};

pub struct DynWinRTAsyncOperationWithProgress(IAsyncInfo, GUID);

impl Future for DynWinRTAsyncOperationWithProgress {
    type Output = windows::core::Result<HSTRING>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if (self.0.Status().unwrap() == AsyncStatus::Completed) {
            // let op: IAsyncOperationWithProgress<HSTRING, HttpProgress> = self.0.cast().unwrap();
            // let r = op.GetResults().unwrap();
            // println!("Got result: {}", r.to_string());
            let sig = interfaces::IAsyncOperationWithProgress();
            let mut ptr = std::ptr::null_mut();
            let hr = unsafe { self.0.query(&self.1, &mut ptr) };
            let ukn = unsafe { IUnknown::from_raw(ptr) };
            let results = sig.methods[10].call_dynamic(ukn.as_raw(), &[]);
            let result = results.map(|vs| vs[0].as_hstring().unwrap());
            println!(
                "Got result via vtable: {}",
                result.clone().unwrap().to_string()
            );
            return std::task::Poll::Ready(result);
        }
        cx.waker().wake_by_ref();
        std::task::Poll::Pending
    }
}

pub struct DynWinRTAsyncOperationIUnknown(pub IAsyncInfo, pub GUID);

impl Future for DynWinRTAsyncOperationIUnknown {
    type Output = windows::core::Result<IUnknown>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if (self.0.Status().unwrap() == AsyncStatus::Completed) {
            let insp: IInspectable = self.0.cast()?;
            println!("Inspectable: {:?}", insp.GetRuntimeClassName()?);

            let sig = interfaces::IAsyncOperation();
            let mut ptr = std::ptr::null_mut();
            unsafe { self.0.query(&self.1, &mut ptr) };
            let ukn = unsafe { IUnknown::from_raw(ptr) };
            let results = sig.methods[8].call_dynamic(ukn.as_raw(), &[]);
            let result = results.map(|vs| vs[0].as_object().unwrap().clone());
            // let op = insp.cast::<IAsyncOperation<bindings::PickFileResult>>()?;
            // let r = op.GetResults()?;
            // let u : IUnknown = r.cast()?;
            // return std::task::Poll::Ready(Ok(u));
            return std::task::Poll::Ready(result);
        }
        cx.waker().wake_by_ref();
        std::task::Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::{Foundation::Uri, Web::Http::HttpClient};
    use windows_core::{HSTRING, h};
    use windows_future::IAsyncInfo;

    use crate::dasync::DynWinRTAsyncOperationWithProgress;

    #[tokio::test]
    async fn simple_test() -> windows_core::Result<()> {
        let client = HttpClient::new()?;
        let url = Uri::CreateUri(h!("https://www.microsoft.com"))?;
        let response = client.GetStringAsync(&url)?;
        let asyncInfo: IAsyncInfo = response.cast()?;
        println!("status {:?}", asyncInfo.Status()?);
        let iid = IAsyncOperationWithProgress::<HSTRING, HttpProgress>::IID;
        let op = DynWinRTAsyncOperationWithProgress(asyncInfo, iid);
        let r = op.await?;
        assert!(!r.is_empty());
        println!("Response length: {}", r.to_string());
        Ok(())
    }
}
