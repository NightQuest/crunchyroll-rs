use crate::common::Request;
use crate::error::{CrunchyrollError, CrunchyrollErrorContext, Result};
use crate::{Crunchyroll, Executor, Locale};
use serde::de::{DeserializeOwned, Error};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Write;
use std::sync::Arc;

trait FixStream: DeserializeOwned {
    type Variant: DeserializeOwned;
}

fn deserialize_streams<'de, D: Deserializer<'de>, T: FixStream>(
    deserializer: D,
) -> Result<HashMap<Locale, T>, D::Error> {
    let as_map: HashMap<String, HashMap<Locale, Value>> = HashMap::deserialize(deserializer)?;

    let mut raw: HashMap<Locale, HashMap<String, Value>> = HashMap::new();
    for (key, value) in as_map {
        for (mut locale, data) in value {
            if locale == Locale::Custom(":".to_string()) {
                locale = Locale::Custom("".to_string());
            }

            // check only for errors and not use the `Ok(...)` result in `raw` because `T::Variant`
            // then must implement `serde::Serialize`
            if let Err(e) = T::Variant::deserialize(&data) {
                return Err(Error::custom(e.to_string()));
            }

            if let Some(entry) = raw.get_mut(&locale) {
                entry.insert(key.clone(), data.clone());
            } else {
                raw.insert(locale, HashMap::from([(key.clone(), data)]));
            }
        }
    }

    let as_value = serde_json::to_value(raw).map_err(|e| Error::custom(e.to_string()))?;
    serde_json::from_value(as_value).map_err(|e| Error::custom(e.to_string()))
}

/// A video stream.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct VideoStream {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub media_id: String,
    /// Audio locale of the stream.
    pub audio_locale: Locale,
    /// All subtitles.
    pub subtitles: HashMap<Locale, StreamSubtitle>,

    /// All stream variants.
    /// One stream has multiple variants how it can be delivered. At the time of writing,
    /// all variants are either [HLS](https://en.wikipedia.org/wiki/HTTP_Live_Streaming)
    /// or [MPEG-DASH](https://en.wikipedia.org/wiki/Dynamic_Adaptive_Streaming_over_HTTP) streams.
    ///
    /// The data is stored in a map where the key represents the data's hardsub locale (-> subtitles
    /// are "burned" into the video) and the value all stream variants.
    /// If you want no hardsub at all, use the `Locale::Custom("".into())` map entry.
    #[serde(rename = "streams")]
    #[serde(deserialize_with = "deserialize_streams")]
    #[cfg_attr(not(feature = "__test_strict"), default(HashMap::new()))]
    pub variants: HashMap<Locale, VideoVariants>,

    #[cfg(feature = "__test_strict")]
    captions: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    bifs: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    versions: crate::StrictValue,
}

impl Request for VideoStream {
    fn __set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor.clone();

        for value in self.subtitles.values_mut() {
            value.executor = executor.clone();
        }
    }

    fn __get_executor(&self) -> Option<Arc<Executor>> {
        Some(self.executor.clone())
    }
}

impl VideoStream {
    pub async fn from_id(crunchy: &Crunchyroll, id: String) -> Result<Self> {
        let endpoint = format!(
            "https://beta-api.crunchyroll.com/cms/v2/{}/videos/{}/streams",
            crunchy.executor.details.bucket, id
        );
        let builder = crunchy
            .executor
            .client
            .get(endpoint)
            .query(&crunchy.executor.media_query());

        crunchy.executor.request(builder).await
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct PlaybackStream {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub audio_locale: Locale,
    pub subtitles: HashMap<Locale, StreamSubtitle>,
    #[serde(rename = "streams")]
    #[serde(deserialize_with = "deserialize_streams")]
    #[default(HashMap::new())]
    pub variants: HashMap<Locale, PlaybackVariants>,

    #[cfg(feature = "__test_strict")]
    #[serde(rename = "QoS")]
    qos: crate::StrictValue,
}

impl Request for PlaybackStream {
    fn __set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor.clone();

        for value in self.subtitles.values_mut() {
            value.executor = executor.clone();
        }
    }

    fn __get_executor(&self) -> Option<Arc<Executor>> {
        Some(self.executor.clone())
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct StreamSubtitle {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub locale: Locale,
    pub url: String,
    pub format: String,
}

impl StreamSubtitle {
    pub async fn write_to(self, w: &mut impl Write) -> Result<()> {
        let resp = self.executor.client.get(self.url).send().await?;
        let body = resp.bytes().await?;
        w.write_all(body.as_ref())
            .map_err(|e| CrunchyrollError::External(CrunchyrollErrorContext::new(e.to_string())))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct VideoVariant {
    /// Language of this variant.
    pub hardsub_locale: Locale,
    /// Url to the actual stream.
    /// Usually a [HLS](https://en.wikipedia.org/wiki/HTTP_Live_Streaming)
    /// or [MPEG-DASH](https://en.wikipedia.org/wiki/Dynamic_Adaptive_Streaming_over_HTTP) stream.
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct PlaybackVariant {
    /// Language of this variant.
    pub hardsub_locale: Locale,
    /// Url to the actual stream.
    /// Usually a [HLS](https://en.wikipedia.org/wiki/HTTP_Live_Streaming)
    /// or [MPEG-DASH](https://en.wikipedia.org/wiki/Dynamic_Adaptive_Streaming_over_HTTP) stream.
    pub url: String,

    /// Video codec of the variant. Usually h264
    pub vcodec: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Default)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct VideoVariants {
    pub adaptive_dash: Option<VideoVariant>,
    pub adaptive_hls: Option<VideoVariant>,
    pub download_dash: Option<VideoVariant>,
    pub download_hls: Option<VideoVariant>,
    pub drm_adaptive_dash: Option<VideoVariant>,
    pub drm_adaptive_hls: Option<VideoVariant>,
    pub drm_download_dash: Option<VideoVariant>,
    pub drm_download_hls: Option<VideoVariant>,
    pub drm_multitrack_adaptive_hls_v2: Option<VideoVariant>,
    pub multitrack_adaptive_hls_v2: Option<VideoVariant>,
    pub vo_adaptive_dash: Option<VideoVariant>,
    pub vo_adaptive_hls: Option<VideoVariant>,
    pub vo_drm_adaptive_dash: Option<VideoVariant>,
    pub vo_drm_adaptive_hls: Option<VideoVariant>,

    #[cfg(feature = "__test_strict")]
    urls: Option<crate::StrictValue>,
}

impl FixStream for VideoVariants {
    type Variant = VideoVariant;
}

#[derive(Clone, Debug, Deserialize, Default)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct PlaybackVariants {
    pub adaptive_dash: Option<PlaybackVariant>,
    pub adaptive_hls: Option<PlaybackVariant>,
    pub download_hls: Option<PlaybackVariant>,
    pub drm_adaptive_dash: Option<PlaybackVariant>,
    pub drm_adaptive_hls: Option<PlaybackVariant>,
    pub drm_download_hls: Option<PlaybackVariant>,
    pub trailer_dash: Option<PlaybackVariant>,
    pub trailer_hls: Option<PlaybackVariant>,
    pub vo_adaptive_dash: Option<PlaybackVariant>,
    pub vo_adaptive_hls: Option<PlaybackVariant>,
    pub vo_drm_adaptive_dash: Option<PlaybackVariant>,
    pub vo_drm_adaptive_hls: Option<PlaybackVariant>,
}

impl FixStream for PlaybackVariants {
    type Variant = PlaybackVariant;
}