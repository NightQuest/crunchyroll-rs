use crate::categories::Category;
use crate::common::{BulkResult, Image};
use crate::error::{CrunchyrollError, CrunchyrollErrorContext};
use crate::media::old_media::{OldEpisode, OldMovie, OldSeason};
use crate::media::{PlaybackStream, VideoStream};
use crate::{options, Crunchyroll, Executor, Locale, Request, Result};
use chrono::{DateTime, Duration, Utc};
use serde::de::{DeserializeOwned, IntoDeserializer};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::sync::Arc;

pub trait Video: Default + DeserializeOwned + Request {}

#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct SearchMetadata {
    /// `None` if queried by [`Crunchyroll::by_query`].
    pub last_public: Option<DateTime<Utc>>,
    /// `None` if queried by [`Crunchyroll::by_query`].
    pub rank: Option<u32>,

    pub score: f64,
    /// `None` if not queried by [`Series::similar`] or [`MovieListing::similar`].
    pub popularity_score: Option<f64>,
}

/// Metadata for a [`Media`] series.
#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Series {
    pub extended_description: String,

    pub series_launch_year: Option<u32>,

    pub episode_count: u32,
    pub season_count: u32,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub is_simulcast: bool,
    pub audio_locales: Vec<Locale>,
    pub subtitle_locales: Vec<Locale>,

    #[serde(default)]
    #[serde(rename = "tenant_categories")]
    pub categories: Vec<Category>,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub availability_notes: String,

    #[cfg(feature = "__test_strict")]
    pub(crate) extended_maturity_rating: crate::StrictValue,
}
impl Video for Series {}

/// Metadata for a [`Media`] season.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Season {
    /// Sometimes populated, sometimes not. idk why. Crunchyroll.
    #[serde(deserialize_with = "crate::internal::serde::deserialize_maybe_broken_locale_vec")]
    pub audio_locales: Vec<Locale>,
    /// Sometimes populated, sometimes not. idk why. Crunchyroll.
    pub subtitle_locales: Vec<Locale>,

    /// Currently only populated if this season got generated by [`Media<Series>::seasons`].
    #[serde(default)]
    pub season_number: u32,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    #[cfg(feature = "__test_strict")]
    pub(crate) audio_locale: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) season_display_number: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) season_sequence_number: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) versions: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) identifier: crate::StrictValue,
}
impl Video for Season {}

/// Metadata for a [`Media`] episode.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Episode {
    pub series_id: String,
    pub series_title: String,
    pub series_slug_title: String,

    pub season_id: String,
    pub season_title: String,
    pub season_slug_title: String,
    pub season_number: u32,

    /// Usually the same as [`Episode::episode_number`], just as string.
    pub episode: String,
    #[serde(deserialize_with = "crate::internal::serde::deserialize_maybe_null_to_default")]
    pub episode_number: u32,
    /// Usually the same as [`Episode::episode_number`], I don't know the purpose of this.
    pub sequence_number: u32,
    #[serde(alias = "duration_ms")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[default(Duration::milliseconds(0))]
    pub duration: Duration,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub episode_air_date: DateTime<Utc>,
    /// The same as episode_air_date as far as I can see.
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub upload_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub free_available_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub premium_available_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub availability_starts: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub availability_ends: DateTime<Utc>,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub closed_captions_available: bool,
    /// Would be very useful, but is (currently) always empty.
    #[serde(default)]
    pub audio_locale: String,
    pub subtitle_locales: Vec<Locale>,

    pub is_clip: bool,
    pub is_premium_only: bool,

    #[serde(default)]
    #[serde(rename = "tenant_categories")]
    pub categories: Vec<Category>,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub available_offline: bool,
    pub availability_notes: String,

    pub eligible_region: String,

    #[cfg(feature = "__test_strict")]
    pub(crate) extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) available_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) premium_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) versions: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) identifier: crate::StrictValue,
}
impl Video for Episode {}

/// Metadata for a [`Media`] movie listing.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct MovieListing {
    // wtf is this again
    pub first_movie_id: String,

    pub extended_description: String,

    pub movie_release_year: u32,

    #[serde(alias = "duration_ms")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[default(Duration::milliseconds(0))]
    pub duration: Duration,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub subtitle_locales: Vec<Locale>,

    pub is_premium_only: bool,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub free_available_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub premium_available_date: DateTime<Utc>,

    #[serde(default)]
    #[serde(rename = "tenant_categories")]
    pub categories: Vec<Category>,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub available_offline: bool,
    pub availability_notes: String,

    #[cfg(feature = "__test_strict")]
    pub(crate) extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) available_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) premium_date: crate::StrictValue,
}

impl Video for MovieListing {}

/// Metadata for a [`Media`] movie.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Movie {
    pub movie_listing_id: String,

    pub movie_listing_title: String,
    pub movie_listing_slug_title: String,

    #[serde(alias = "duration_ms")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[default(Duration::milliseconds(0))]
    pub duration: Duration,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub closed_captions_available: bool,

    pub is_premium_only: bool,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub available_offline: bool,
    pub availability_notes: String,

    #[cfg(feature = "__test_strict")]
    pub(crate) extended_maturity_rating: crate::StrictValue,
}
impl Video for Movie {}

/// Collection of all media types. Useful in situations where [`Media`] can contain more than one
/// specific media.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Request)]
pub enum MediaCollection {
    Series(Media<Series>),
    Season(Media<Season>),
    Episode(Media<Episode>),
    MovieListing(Media<MovieListing>),
    Movie(Media<Movie>),
}

impl Default for MediaCollection {
    fn default() -> Self {
        Self::Series(Media::default())
    }
}

impl<'de> Deserialize<'de> for MediaCollection {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let as_map = serde_json::Map::deserialize(deserializer)?;

        let err_conv = |e: serde_json::Error| serde::de::Error::custom(e.to_string());

        if as_map.contains_key("series_metadata") {
            Ok(MediaCollection::Series(
                Media::deserialize(Value::from(as_map).into_deserializer()).map_err(err_conv)?,
            ))
        } else if as_map.contains_key("season_metadata") {
            Ok(MediaCollection::Season(
                Media::deserialize(Value::from(as_map).into_deserializer()).map_err(err_conv)?,
            ))
        } else if as_map.contains_key("episode_metadata") {
            Ok(MediaCollection::Episode(
                Media::deserialize(Value::from(as_map).into_deserializer()).map_err(err_conv)?,
            ))
        } else if as_map.contains_key("movie_listing_metadata") {
            Ok(MediaCollection::MovieListing(
                Media::deserialize(Value::from(as_map).into_deserializer()).map_err(err_conv)?,
            ))
        } else if as_map.contains_key("movie_metadata") {
            Ok(MediaCollection::Movie(
                Media::deserialize(Value::from(as_map).into_deserializer()).map_err(err_conv)?,
            ))
        } else {
            Err(serde::de::Error::custom(
                "no metadata were found".to_string(),
            ))
        }
    }
}

impl MediaCollection {
    pub async fn from_id(crunchy: &Crunchyroll, id: String) -> Result<MediaCollection> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/cms/v2/{}/objects/{}",
            crunchy.executor.details.bucket, &id
        );
        let result: BulkResult<MediaCollection> = crunchy
            .executor
            .get(endpoint)
            .apply_media_query()
            .apply_locale_query()
            .request()
            .await?;

        if result.items.is_empty() {
            Err(CrunchyrollError::Input(
                format!("no media could be found for id '{}'", id).into(),
            ))
        } else if result.items.len() >= 2 {
            // if this ever gets fired, crunchyroll hopefully unified episode and movie on the api
            // level (this functions was only implemented so `Crunchyroll::parse_url` can work
            // easily)
            Err(CrunchyrollError::Internal(format!("multiple media were found for id '{}'. this shouldn't happen, please report it immediately!", id).into()))
        } else {
            Ok(result.items.into_iter().next().unwrap())
        }
    }
}

macro_rules! impl_try_into_media {
    ($($generic:path = $enum_field:ident)*) => {
        $(
            impl TryInto<Media<$generic>> for MediaCollection {
                type Error = CrunchyrollError;

                fn try_into(self) -> std::result::Result<Media<$generic>, Self::Error> {
                    if let MediaCollection::$enum_field(value) = self {
                        Ok(value)
                    } else {
                        Err(CrunchyrollError::Input(CrunchyrollErrorContext::new(format!("collection is no '{}'", stringify!($generic)).to_string())))
                    }
                }
            }
        )*
    }
}

impl_try_into_media! {
    Series = Series
    Season = Season
    Episode = Episode
    MovieListing = MovieListing
    Movie = Movie
}

macro_rules! impl_from_media {
    ($($generic:path = $enum_field:ident)*) => {
        $(
            impl From<Media<$generic>> for MediaCollection {
                fn from(value: Media<$generic>) -> Self {
                    Self::$enum_field(value)
                }
            }
        )*
    }
}

impl_from_media! {
    Series = Series
    Season = Season
    Episode = Episode
    MovieListing = MovieListing
    Movie = Movie
}

#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct MediaImages {
    pub thumbnail: Option<Vec<Vec<Image>>>,
    pub poster_tall: Option<Vec<Vec<Image>>>,
    pub poster_wide: Option<Vec<Vec<Image>>>,
    pub promo_image: Option<Vec<Vec<Image>>>,
}

/// Base struct which stores all information about series, seasons, episodes, movie listings and
/// movies. The generic this struct takes specifies which media type it actually contains.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[request(executor(metadata))]
#[serde(bound = "M: Video")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Media<M: Video> {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,
    #[serde(rename = "__links__")]
    #[serde(default)]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_stream_id_option")]
    pub stream_id: Option<String>,
    #[serde(rename = "playback")]
    pub playback_url: Option<String>,
    pub external_id: String,
    pub channel_id: String,

    pub slug: String,
    pub title: String,
    pub slug_title: String,
    pub promo_title: String,
    pub description: String,
    pub promo_description: String,

    #[serde(alias = "series_metadata")]
    #[serde(alias = "season_metadata")]
    #[serde(alias = "episode_metadata")]
    #[serde(alias = "movie_listing_metadata")]
    #[serde(alias = "movie_metadata")]
    pub metadata: M,

    /// only populated if `Panel` results from search query 'src/search-series'
    pub search_metadata: Option<SearchMetadata>,

    pub images: Option<MediaImages>,

    #[cfg(feature = "__test_strict")]
    #[serde(alias = "type")]
    pub(crate) collection_type: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) new: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    pub(crate) new_content: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    pub(crate) last_public: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    pub(crate) linked_resource_key: crate::StrictValue,
}

impl<M: Video> Media<M> {
    pub async fn from_id(crunchy: &Crunchyroll, id: String) -> Result<Media<M>> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/cms/v2/{}/objects/{}",
            crunchy.executor.details.bucket, &id
        );
        let result: BulkResult<Media<M>> = crunchy
            .executor
            .get(endpoint)
            .apply_media_query()
            .apply_locale_query()
            .request()
            .await?;

        if result.items.is_empty() {
            Err(CrunchyrollError::Input(
                format!("no media could be found for id '{}'", id).into(),
            ))
        } else if result.items.len() >= 2 {
            // if this ever gets fired, crunchyroll hopefully unified episode and movie on the api
            // level (this functions was only implemented so `Crunchyroll::parse_url` can work
            // easily)
            Err(CrunchyrollError::Internal(format!("multiple media were found for id '{}'. this shouldn't happen, please report it immediately!", id).into()))
        } else {
            Ok(result.items.into_iter().next().unwrap())
        }
    }

    pub async fn playback(&self) -> Result<PlaybackStream> {
        if let Some(playback_url) = &self.playback_url {
            self.executor.get(playback_url).request().await
        } else {
            Err(CrunchyrollError::Request("no playback id available".into()))
        }
    }
}

impl Media<Series> {
    pub async fn seasons(&self) -> Result<Vec<Media<Season>>> {
        Media::<Season>::from_series_id(
            &Crunchyroll {
                executor: self.executor.clone(),
            },
            self.id.clone(),
        )
        .await
    }
}

impl Media<Season> {
    pub async fn from_series_id(
        crunchy: &Crunchyroll,
        series_id: String,
    ) -> Result<Vec<Media<Season>>> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/cms/v2/{}/seasons",
            crunchy.executor.details.bucket
        );
        let result: BulkResult<OldSeason> = crunchy
            .executor
            .get(endpoint)
            .query(&[("series_id", series_id)])
            .apply_media_query()
            .apply_locale_query()
            .request()
            .await?;
        Ok(result.items.into_iter().map(|i| i.into()).collect())
    }

    pub async fn episodes(&self) -> Result<Vec<Media<Episode>>> {
        Media::<Episode>::from_season_id(
            &Crunchyroll {
                executor: self.executor.clone(),
            },
            self.id.clone(),
        )
        .await
    }
}

impl Media<Episode> {
    pub async fn from_season_id(
        crunchy: &Crunchyroll,
        season_id: String,
    ) -> Result<Vec<Media<Episode>>> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/cms/v2/{}/episodes",
            crunchy.executor.details.bucket
        );
        let result: BulkResult<OldEpisode> = crunchy
            .executor
            .get(endpoint)
            .query(&[("season_id", season_id)])
            .apply_media_query()
            .apply_locale_query()
            .request()
            .await?;
        Ok(result.items.into_iter().map(|i| i.into()).collect())
    }
}

impl Media<MovieListing> {
    pub async fn movies(&self) -> Result<Vec<Media<Movie>>> {
        Media::<Movie>::from_movie_listing_id(
            &Crunchyroll {
                executor: self.executor.clone(),
            },
            self.id.clone(),
        )
        .await
    }
}

impl Media<Movie> {
    pub async fn from_movie_listing_id(
        crunchy: &Crunchyroll,
        movie_listing_id: String,
    ) -> Result<Vec<Media<Movie>>> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/cms/v2/{}/movies",
            crunchy.executor.details.bucket
        );
        let result: BulkResult<OldMovie> = crunchy
            .executor
            .get(endpoint)
            .query(&[("movie_listing_id", movie_listing_id)])
            .apply_media_query()
            .apply_locale_query()
            .request()
            .await?;
        Ok(result.items.into_iter().map(|i| i.into()).collect())
    }
}

impl Crunchyroll {
    /// Get a media by its id. The `M` generic says what media exactly should be requested. Available
    /// options are [`Series`], [`Season`], [`Episode`], [`MovieListing`] and [`Movie`].
    pub async fn media_from_id<M: Video>(&self, id: impl AsRef<str>) -> Result<Media<M>> {
        Media::from_id(self, id.as_ref().to_string()).await
    }

    pub async fn media_collection_from_id<S: AsRef<str>>(&self, id: S) -> Result<MediaCollection> {
        MediaCollection::from_id(self, id.as_ref().to_string()).await
    }
}

options! {
    SimilarOptions;
    /// Limit of results to return.
    limit(u32, "n") = Some(20),
    /// Specifies the index from which the entries should be returned.
    start(u32, "start") = None
}

macro_rules! impl_from_id {
    ($($media:ident)*) => {
        $(
            impl $media {
                /// Return a [`Media`] implementation of this type by its id.
                pub async fn from_id(crunchy: &Crunchyroll, id: String) -> Result<Media<$media>> {
                    Media::from_id(crunchy, id).await
                }
            }
        )*
    }
}

impl_from_id! {
    Series Season Episode
    MovieListing Movie
}

macro_rules! impl_media_video_collection {
    ($($media_video:ident)*) => {
        $(
            impl Media<$media_video> {
                /// Similar series or movie listing to the current item.
                pub async fn similar(&self, options: SimilarOptions) -> Result<BulkResult<MediaCollection>> {
                    let endpoint = format!("https://beta.crunchyroll.com/content/v1/{}/similar_to", self.executor.details.account_id);
                    self.executor.get(endpoint)
                        .query(&[("guid", &self.id)])
                        .query(&options.into_query())
                        .apply_locale_query()
                        .request()
                        .await
                }
            }
        )*
    }
}

impl_media_video_collection! {
    Series MovieListing
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
struct VideoIntroResult {
    media_id: String,

    #[serde(rename = "startTime")]
    start_time: f64,
    #[serde(rename = "endTime")]
    end_time: f64,
    duration: f64,

    /// Id of the next episode.
    #[serde(rename = "comparedWith")]
    compared_with: String,

    /// It seems that this represents the episode number relative to the season the episode is part
    /// of. But in a weird way. It is, for example, '0003.00' instead of simply 3 if it's the third
    /// episode in a season.
    ordering: String,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    last_updated: DateTime<Utc>,
}

macro_rules! impl_media_video {
    ($($media_video:ident)*) => {
        $(
            impl Media<$media_video> {
                /// Streams for this episode / movie.
                pub async fn streams(&self) -> Result<VideoStream> {
                    let endpoint = format!(
                        "https://beta.crunchyroll.com/cms/v2/{}/videos/{}/streams",
                        self.executor.details.bucket, self.stream_id.as_ref().unwrap_or(&self.id)
                    );
                    self.executor.get(endpoint)
                        .apply_media_query()
                        .apply_locale_query()
                        .request()
                        .await
                }

                /// Check if the episode / movie can be watched.
                pub async fn available(&self) -> bool {
                    self.executor.details.premium || !self.metadata.is_premium_only
                }

                /// Get time _in seconds_ when the episode / movie intro begins and ends.
                pub async fn intro(&self) -> Result<Option<(f64, f64)>> {
                    let endpoint = format!(
                        "https://static.crunchyroll.com/datalab-intro-v2/{}.json",
                        self.id
                    );
                    let result: serde_json::Value = self.executor.get(endpoint)
                        .request()
                        .await?;
                    if result.to_string().contains("</Error>") {
                        Ok(None)
                    } else {
                        let video_intro_result: VideoIntroResult = serde_json::from_value(result)?;
                        Ok(Some((video_intro_result.start_time, video_intro_result.end_time)))
                    }
                }
            }
        )*
    }
}

impl_media_video! {
    Episode Movie
}