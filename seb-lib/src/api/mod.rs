use serde::de::DeserializeOwned;

pub(crate) mod cross_ref;
pub(crate) mod format_api;
pub(crate) mod google_books;
pub(crate) mod ietf;

pub trait Client
where
    Self: Default,
{
    fn get_text(&self, url: &str) -> Result<String, Error>;
    fn get_json<T>(&self, url: &str) -> Result<T, Error>
    where
        T: DeserializeOwned;
}

impl Client for reqwest::blocking::Client {
    fn get_text(&self, url: &str) -> Result<String, Error> {
        let resp = self
            .get(url)
            .send()
            .map_err(|e| Error::wrap(ErrorKind::IO, e))?;
        let text = resp
            .text()
            .map_err(|e| Error::wrap(ErrorKind::Deserialize, e))?;

        if text.is_empty() {
            Err(Error::new(ErrorKind::NoValue, "Response text is empty"))
        } else {
            Ok(text)
        }
    }

    fn get_json<T>(&self, url: &str) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        self.get(url)
            .send()
            .map_err(|e| Error::wrap(ErrorKind::IO, e))
            .and_then(|r| r.json().map_err(|e| Error::wrap(ErrorKind::Deserialize, e)))
    }
}

#[cfg(test)]
pub(crate) use test::{
    assert_url, impl_text_producer, MockClient, NetworkErrorProducer, Producer, URL_SINK,
};

use crate::{Error, ErrorKind};

#[cfg(test)]
mod test {

    use super::*;

    thread_local! {
        pub(crate) static URL_SINK: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
    }

    /// Asserts that the expected URL is the same as the one provided to the [`MockClient`].
    ///
    /// The [`MockClient`] will update the static thread local `URL_SINK` with the URL string that
    /// was passed to it, this allows for asserting that implementing functions or methods are
    /// parsing the correct URL.
    ///
    /// This macro provides a shortcut alternative to the following:
    ///
    /// ```ignore
    /// // .. test code including `MockClient`
    ///
    /// let url = crate::api::URL_SINK.with(|url| url.borrow().clone().unwrap_or_default());
    /// assert_eq!("expected url here", url);
    /// ```
    macro_rules! assert_url {
        ($expected: expr) => {
            assert_url!($expected, "");
        };
        ($expected: expr, $($arg: tt)+) => {
            let url = crate::api::URL_SINK.with(|url| url.borrow().clone().unwrap_or_default());
            assert_eq!($expected, url, $($arg)+);
        };
    }

    pub(crate) trait Producer<T>
    where
        Self: Default,
    {
        fn produce() -> Result<T, Error>;
    }

    #[derive(Default)]
    pub(crate) struct MockClient<P: Producer<String> = EmptyTextProducer> {
        _producer: std::marker::PhantomData<P>,
    }

    impl<P: Producer<String>> Client for MockClient<P> {
        fn get_text(&self, url: &str) -> Result<String, Error> {
            URL_SINK.with(|sink| *sink.borrow_mut() = Some(url.to_owned()));
            P::produce()
        }

        fn get_json<T>(&self, url: &str) -> Result<T, Error>
        where
            T: DeserializeOwned,
        {
            URL_SINK.with(|sink| *sink.borrow_mut() = Some(url.to_owned()));
            P::produce().and_then(|json| {
                serde_json::from_str(&json).map_err(|e| Error::wrap(ErrorKind::Deserialize, e))
            })
        }
    }

    macro_rules! impl_text_producer {
        ($($producer:ident => $exp:expr,)*) => {
            $(
                #[derive(Default)]
                pub(crate) struct $producer;

                impl crate::api::Producer<String> for $producer {
                    fn produce() -> Result<String, crate::Error> {
                        $exp
                    }
                }
            )*
        };
    }
    impl_text_producer! {
        EmptyTextProducer => Ok("".to_owned()),
        NetworkErrorProducer => Err(Error::new(ErrorKind::IO, "Network error")),
    }

    pub(crate) use assert_url;
    pub(crate) use impl_text_producer;
}
