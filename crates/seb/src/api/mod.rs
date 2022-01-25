use serde::de::DeserializeOwned;

pub(crate) mod cross_ref;
pub(crate) mod format_api;
pub(crate) mod google_books;
pub(crate) mod ietf;

/// The errors that may occur when using an API function
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// A network error containing the source error.
    Network(Box<dyn std::error::Error + Send + Sync>),
    /// A deserialization or parsing error containing the source error.
    Deserialize(Box<dyn std::error::Error + Send + Sync>),
    /// An error when the API was expected to send back a value in the body.
    NoValue,
}

impl Error {
    fn network<E>(source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Network(Box::new(source))
    }

    fn deserialize<E>(source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Deserialize(Box::new(source))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(_) => f.write_str("Network error"),
            Self::Deserialize(_) => f.write_str(
                "Cannot parse or deserialize the information returned \
                by the API",
            ),
            Self::NoValue => f.write_str("No value was found from the API for the request"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Network(src) | Self::Deserialize(src) => Some(src.as_ref()),
            Self::NoValue => None,
        }
    }
}

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
        let resp = self.get(url).send().map_err(Error::network)?;
        let text = resp.text().map_err(Error::deserialize)?;

        if text.is_empty() {
            Err(Error::NoValue)
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
            .map_err(Error::network)
            .and_then(|r| r.json().map_err(Error::deserialize))
    }
}

#[cfg(test)]
pub(crate) use test::{
    impl_text_producer, MockJsonClient, MockTextClient, NetworkErrorProducer, Producer,
};

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
                    fn produce() -> Result<String, crate::api::Error> {
                        $exp
                    }
                }
            )*
        };
    }
    impl_text_producer! {
        NetworkErrorProducer => Err(Error::Network(eyre::eyre!("Network error").into())),
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
            P::produce().and_then(|s| serde_json::from_str(&s).map_err(Error::deserialize))
        }
    }
}
