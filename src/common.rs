use crate::{Executor, Result};
use futures_util::{Stream, StreamExt};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pub(crate) use crunchyroll_rs_internal::Request;

/// Contains a variable amount of items and the maximum / total of item which are available.
/// Mostly used when fetching pagination results.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[request(executor(data))]
#[serde(bound = "T: Request + DeserializeOwned")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct V2BulkResult<T, M = serde_json::Map<String, serde_json::Value>>
where
    T: Default + DeserializeOwned + Request,
    M: Default + DeserializeOwned + Send,
{
    pub data: Vec<T>,
    #[serde(default)]
    pub total: u32,

    #[serde(default)]
    pub(crate) meta: M,
}

#[allow(clippy::type_complexity)]
pub struct Pagination<T: Default + DeserializeOwned + Request> {
    data: Vec<T>,

    init: bool,
    next_fn: Box<
        dyn FnMut(
            u32,
            Arc<Executor>,
            Vec<(String, String)>,
        ) -> Pin<Box<dyn Future<Output = Result<(Vec<T>, u32)>> + Send + 'static>>,
    >,
    next_state: Option<Pin<Box<dyn Future<Output = Result<(Vec<T>, u32)>> + Send + 'static>>>,

    fn_executor: Arc<Executor>,
    fn_query: Vec<(String, String)>,

    count: u32,
    total: u32,
}

impl<T: Default + DeserializeOwned + Request> Stream for Pagination<T> {
    type Item = Result<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.count < self.total || !self.init {
            let this = self.get_mut();

            if this.next_state.is_none() {
                let f = this.next_fn.as_mut();
                this.next_state = Some(f(
                    this.count,
                    this.fn_executor.clone(),
                    this.fn_query.clone(),
                ))
            }

            let fut = this.next_state.as_mut().unwrap();
            match Pin::new(fut).poll(cx) {
                Poll::Ready(result) => match result {
                    Ok((t, total)) => {
                        this.data = t;
                        this.total = total;
                        this.next_state = None;
                    }
                    Err(e) => return Poll::Ready(Some(Err(e))),
                },
                Poll::Pending => return Poll::Pending,
            }

            this.init = true;
            this.count += 1;
            Poll::Ready(Some(Ok(this.data.remove(0))))
        } else {
            Poll::Ready(None)
        }
    }
}

impl<T: Default + DeserializeOwned + Request> Unpin for Pagination<T> {}

impl<T: Default + DeserializeOwned + Request> Pagination<T> {
    pub(crate) fn new<F>(
        pagination_fn: F,
        executor: Arc<Executor>,
        query_args: Vec<(String, String)>,
    ) -> Self
    where
        F: FnMut(
                u32,
                Arc<Executor>,
                Vec<(String, String)>,
            )
                -> Pin<Box<dyn Future<Output = Result<(Vec<T>, u32)>> + Send + 'static>>
            + Send
            + 'static,
    {
        Self {
            data: vec![],
            init: false,
            next_fn: Box::new(pagination_fn),
            next_state: None,
            fn_executor: executor,
            fn_query: query_args,
            count: 0,
            total: 0,
        }
    }

    /// Return the total amount of items which can be fetched.
    pub async fn total(&mut self) -> u32 {
        if !self.init {
            StreamExt::next(self).await;
        }
        self.total
    }
}

/// Contains a variable amount of items and the maximum / total of item which are available.
/// Mostly used when fetching pagination results.
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[request(executor(items))]
#[serde(bound = "T: Request + DeserializeOwned")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct BulkResult<T: Default + DeserializeOwned + Request> {
    pub items: Vec<T>,
    pub total: u32,
}

/// Just like [`BulkResult`] but without [`BulkResult::total`] because some request does not have
/// this field (but should?!).
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[request(executor(items))]
#[serde(bound = "T: Request + DeserializeOwned")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CrappyBulkResult<T: Default + DeserializeOwned + Request> {
    pub items: Vec<T>,
}

/// The standard representation of images how the api returns them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Image {
    pub source: String,
    #[serde(rename(deserialize = "type"))]
    pub image_type: String,
    pub height: u32,
    pub width: u32,
}

/// Helper trait for [`Crunchyroll::request`] generic returns.
/// Must be implemented for every struct which is used as generic parameter for [`Crunchyroll::request`].
#[doc(hidden)]
#[async_trait::async_trait]
pub trait Request: Send {
    /// Set a usable [`Executor`] instance to the struct if required
    async fn __set_executor(&mut self, _: Arc<Executor>) {}
}

/// Implement [`Request`] for cases where only the request must be done without needing an
/// explicit result.
impl Request for () {}

impl<K: Send, V: Send> Request for HashMap<K, V> {}

impl Request for serde_json::Value {}
