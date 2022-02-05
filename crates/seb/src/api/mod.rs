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
    impl_text_producer, MockJsonClient, MockTextClient, NetworkErrorProducer, Producer,
};

use crate::{Error, ErrorKind};

#[cfg(test)]
mod test {
    use super::*;

    pub(crate) trait Producer<T>
    where
        Self: Default,
    {
        fn produce() -> Result<T, Error>;
    }

    #[derive(Default)]
    pub(crate) struct MockTextClient<P: Producer<String>>(std::marker::PhantomData<P>);

    impl<P: Producer<String>> Client for MockTextClient<P> {
        fn get_text(&self, _: &str) -> Result<String, Error> {
            P::produce()
        }

        fn get_json<T>(&self, _: &str) -> Result<T, Error>
        where
            T: serde::de::DeserializeOwned,
        {
            unimplemented!("Not required")
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
        NetworkErrorProducer => Err(Error::new(ErrorKind::IO, "Network error")),
    }

    pub(crate) use impl_text_producer;

    #[derive(Default)]
    pub(crate) struct MockJsonClient<P: Producer<String>>(std::marker::PhantomData<P>);

    impl<P: Producer<String>> Client for MockJsonClient<P> {
        fn get_text(&self, _: &str) -> Result<String, Error> {
            unimplemented!("Not required")
        }

        fn get_json<T>(&self, _: &str) -> Result<T, Error>
        where
            T: serde::de::DeserializeOwned,
        {
            P::produce().and_then(|s| {
                serde_json::from_str(&s).map_err(|e| Error::wrap(ErrorKind::Deserialize, e))
            })
        }
    }
}
