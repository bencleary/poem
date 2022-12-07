use crate::{Endpoint, Middleware, Request, Result};

/// Middleware that enables data to be available on all request handlers where specified.
/// Useful for sharing database connection pools, data structures, etc, between handlers.
///
///
/// ### Example Usage
/// ```
/// use std::{
///     collections::HashMap,
///     sync::{Arc, Mutex},
/// };
/// use poem::{
/// get, handler, listener::TcpListener, middleware::{AddData, Tracing}, web::{Path, Data}, EndpointExt, Route, Server,
/// };
///
/// struct AppState {
///     clients: Mutex<HashMap<String, String>>,
/// }
///
/// #[handler]
/// fn hello(Path(name): Path<String>, state: Data<&Arc<AppState>>) -> String {
///     let mut store = state.clients.lock().unwrap();
///     store.insert(name.to_string(), "some state object".to_string());
///     "store updated".to_string()
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), std::io::Error> {
///     
///     let state = Arc::new(AppState {
///         clients: Mutex::new(HashMap::new()),
///     });
///
///     let app = Route::new()
///         .at("/hello/:name", get(hello))
///         .with(AddData::new(state))
///         .with(Tracing);
///
///     Server::new(TcpListener::bind("127.0.0.1:3000"))
///     .name("hello-world")
///     .run(app)
///     .await
/// }
///
/// ```
/// ### Further Examples
///
/// - https://github.com/poem-web/poem/tree/master/examples/poem/mongodb
///
///
pub struct AddData<T> {
    value: T,
}

impl<T: Clone + Send + Sync + 'static> AddData<T> {
    /// Create new `AddData` middleware with any value.
    pub fn new(value: T) -> Self {
        AddData { value }
    }
}

impl<E, T> Middleware<E> for AddData<T>
where
    E: Endpoint,
    T: Clone + Send + Sync + 'static,
{
    type Output = AddDataEndpoint<E, T>;

    fn transform(&self, ep: E) -> Self::Output {
        AddDataEndpoint {
            inner: ep,
            value: self.value.clone(),
        }
    }
}

/// Endpoint for AddData middleware.
pub struct AddDataEndpoint<E, T> {
    inner: E,
    value: T,
}

#[async_trait::async_trait]
impl<E, T> Endpoint for AddDataEndpoint<E, T>
where
    E: Endpoint,
    T: Clone + Send + Sync + 'static,
{
    type Output = E::Output;

    async fn call(&self, mut req: Request) -> Result<Self::Output> {
        req.extensions_mut().insert(self.value.clone());
        self.inner.call(req).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{handler, test::TestClient, EndpointExt};

    #[tokio::test]
    async fn test_add_data() {
        #[handler(internal)]
        async fn index(req: &Request) {
            assert_eq!(req.extensions().get::<i32>(), Some(&100));
        }

        let cli = TestClient::new(index.with(AddData::new(100i32)));
        cli.get("/").send().await.assert_status_is_ok();
    }
}
