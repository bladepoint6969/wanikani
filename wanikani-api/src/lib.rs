#![forbid(unsafe_code)]
#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(missing_docs, clippy::todo, clippy::unwrap_used)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # Introduction
//!
//! Welcome to "WaniKani: The API!" You can use our API to access progress data
//! for a user's account and a ton of general reference data for the subjects
//! within WaniKani.
//!
//! This version is built around a [REST](http://en.wikipedia.org/wiki/Representational_State_Transfer)ful
//! structure, with consistent, resource-oriented URLs. We support that
//! structure with standard HTTP features:
//! [HTTP verbs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods) for
//! all our endpoints to indicate different actions,
//! [HTTP authentication headers](https://developer.mozilla.org/en-US/docs/Web/HTTP/Authentication),
//! and [HTTP response codes](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status)
//! to indicate both success and various errors. We've turned on cross-origin
//! resource sharing to allow for secure client-side access. We respond to all
//! requests with JSON, making it easy to parse those responses into native
//! objects in a variety of languages. These should open up the API to any
//! client that supports these features and data structures.
//!
//! We've got information on general usage, like authentication and error codes,
//! in [Getting Started](#getting-started). We make a few suggestions on how to
//! optimize your usage of the API in [Best Practices](#best-practices) and
//! clarify a few obscure topics under
//! [Additional Information](#additional-information). Finally, details for all
//! of the available resources and endpoints are under their respective
//! sections.
//!
//! Feel free to reach out [via email](mailto:hello@wanikani.com) or
//! [through the community](https://community.wanikani.com/) if you have any
//! questions, comments, or requests about the API.
//!
//! ## Getting Started
//!
//! ### Authentication
//!
//! WaniKani uses your secret API token to authenticate requests to the API. You
//! can obtain and manage your v2 token in
//! [Settings / API Tokens](https://www.wanikani.com/settings/personal_access_tokens)
//! on WaniKani. The token **has** to be included with every request, and should be
//! delivered in a HTTP header that looks like:
//!
//! `Authorization: Bearer <api_token_here>`
//!
//! Also note that all requests must be made over HTTPS. Any requests made over
//! HTTP or without authentication headers will fail.
//!
//! > **Warning**: You must replace `<api_token_here>` with your API key.
//!
//! ### Response Structure
//!
//! We return JSON from all the API endpoints, even when an error occurs.
//!
//! There are two main structures we return: resources and collections. Singular
//! resource endpoints deliver information about a single entity, such as an
//! assignment or subject. Collections contain summary data about a bunch of
//! resources, and also include each of the resources.
//!
//! There's a third type of structure that's less common: a report. Reports
//! summarize disparate or novel information into a single place, and don't
//! follow the same structure as collections.
//!
//! All of the responses have a few shared, high-level attributes: `object`,
//! `url`, `data_updated_at`, and `data`.
//!
//! #### Object Types
//!
//! Every successful API response contains an object attribute that tells you which
//! kind of thing you're getting. As mentioned before, there are two object types
//! that return information on many different resources:
//!
//! - `collection`: [`Collection`]
//! - `report`: [`Summary`](summary::Summary)
//!
//! The following are singular resources:
//!
//! - `assignment`: [`Assignment`](assignment::Assignment)
//! - `kana_vocabulary`: [`KanaVocabulary`](subject::KanaVocabulary)
//! - `kanji`: [`Kanji`](subject::Kanji)
//! - `level_progression`: [`LevelProgression`](level_progression::LevelProgression)
//! - `radical`: [`Radical`](subject::Radical)
//! - `reset`: [`Reset`](reset::Reset)
//! - `review_statistic`: [`ReviewStatistic`](review_statistic::ReviewStatistic)
//! - `review`: [`Review`](review::Review)
//! - `spaced_repetition_system`: [`SpacedRepetitionSystem`](srs::SpacedRepetitionSystem)
//! - `study_material`: [`StudyMaterial`](study_material::StudyMaterial)
//! - `user`: [`User`](user::User)
//! - `vocabulary`: [`Vocabulary`](subject::Vocabulary)
//! - `voice_actor`: [`VoiceActor`](voice_actor::VoiceActor)
//!
//! #### Data Types
//!
//! We stick to the common JSON data types in our responses: strings, integers,
//! booleans, arrays, and objects. We follow the Javascript standard for date
//! formatting, returning them in [ISO 8601](https://xkcd.com/1179/) format,
//! rounded to the microsecond.
//!
//! ### Pagination
//!
//! #### Collection Size
//!
//! By default, the maximum number of resources returned for collection
//! endpoints is 500. Some endpoints may return a different size — `reviews` and
//! `subjects` have a maximum size of 1,000.
//!
//! Any collection response has the per-page count in the pages.per_page
//! attribute. Those same responses have a total_count attribute, too. That is a
//! count of all resources available within the [specified scope](#filters),
//! **not** limited to pagination.
//!
//! #### Pagination in Action
//!
//! When there are more resources to return than the per-page limit, we use a
//! [cursor-based pagination](https://www.sitepoint.com/paginating-real-time-data-cursor-based-pagination/)
//! scheme to move through the pages of results. We use the id of a resource as
//! the cursor.
//!
//! Collections have a [`Pages`] object nested within a `pages` attribute.
//!
//! > **Pro tip:** the first page has no previous page, and the last page has no
//! > next page.
//!
//! The previous page of results can be requested by passing in the
//! `page_before_id` parameter, with the value being the id you want to look
//! before. Similar logic applies for the next page. Pass in the `page_after_id`
//! parameter with with the id you want to look after.
//!
//! If a cursor is outside the range of `id`s for the collection, an empty
//! result set is returned for `data`.
//!
//! ##### Example
//!
//! Let’s say there are four resources with IDs of 1, 2, 3, 4.
//!
//! - If we make a request with `...?page_after_id=2`, then we'll get resources
//!   with IDs 3 and 4.
//! - If we make a request with `...?page_before_id=3`, then we'll get resources
//!   with IDs 1 and 2.
//! - If we make a request with `...?page_after_id=5`, then we'll get a
//!   collection with an empty `data` field.
//!
//! ### Filters
//!
//! Collections have optional filters to help narrow the results returned. The
//! filters are passed in as URL parameters, like
//! `?parameter=value&other_parameter=value`.
//!
//! Any time we take a query parameter that's listed as an array data type, we
//! take that array as a comma delimited list of values. A single value is also
//! valid.
//!
//! So, if a collection endpoint takes `subject_ids` as an argument for
//! filtering results, your requests might have the following formats:
//!
//! - A single-member `subject_ids` request: `...?subject_ids=8`
//! - A multiple-member `subject_ids` request: `...?subject_ids=8,16,64`
//!
//! ### Errors
//!
//! We use standard HTTP response codes to indicate the status of the response.
//! Codes in the 200s indicate success, 400s usually indicate a client
//! configuration problem (that's you), while 500s indicate that something bad
//! is happening on the server (that's us).
//!
//! The codes are presented in the header of the response; some error responses
//! will also contain a body with the message specified below:
//!
//! Code | Meaning               | Message
//! -----|-----------------------|--------
//! 200  | Success               | n/a
//! 401  | Unauthorized          | `Unauthorized. Nice try.`
//! 403  | Forbidden             |
//! 404  | Not Found             |
//! 422  | Unprocessable Entity  | Description of how the request was malformed
//! 429  | Too Many Requests     |
//! 500  | Internal Server Error |
//! 503  | Service Unavailable   |
//!
//! ### Rate Limit
//!
//! We enforce the following rate limits to ensure decent response times for
//! everyone using the API:
//!
//! Throttle            | Value
//! --------------------|------
//! Requests per minute | 60
//!
//! An HTTP status code of `429` (Too Many Requests) and a body with the message
//! `Rate Limit Exceeded` is returned if the limits are exceeded (shocking, we
//! know).
//!
//! In the response headers, the following rate limit information is provided:
//!
//! Header              | Description
//! --------------------|------------
//! RateLimit-Limit     | The rate limit for the current period.
//! RateLimit-Remaining | The remaining rate available for the current period.
//! RateLimit-Reset     | The timestamp of when the rate limit will reset. The value is epoch time in seconds.
//!
//! It is recommended to make use of the header rate limit details to
//! programatically handle HTTP status code `429` responses in an optimal way.
//!
//! ### Revisions (aka Versioning)
//!
//! Any time we make 'breaking changes' to the API, we release a new,
//! timestamped revision of the API. Non-breaking changes don't trigger a new
//! revision, and those changes are available in all versions of the API.
//!
//! - A breaking change is anything that changes the existing structure of a
//!   response, e.g. the renaming of a field in a resource.
//! - Non-breaking changes are things like exposing new resource attributes or
//!   adding whole new endpoints.
//!
//! Revisions are designated by timestamps in the format `YYYYMMDD`. We expect
//! the revision to be included in all API requests to the server in a header
//! that looks like the following: `Wanikani-Revision: 20170710`.
//!
//! > If you don't specify a revision, the API will default to the first
//! > revision: [20170710](https://docs.api.wanikani.com/20170710).
//!
//! ## Best Practices
//!
//! We're always working to make the API as performant as possible, but there
//! are a few things you can do to optimize your use of the data we deliver and
//! speed things up when you need to make new requests:
//! [cache data locally](#caching) whenever possible,
//! [make conditional requests](#conditional-requests) to minimize network load,
//! and [make use of the `updated_after` filter](#leveraging-the-updated_after-filter)
//! on a lot of the endpoints.
//!
//! When you're building applications or services that other people will use,
//! there's also some work to be done to
//! [respect the access to content granted by a subscription](respecting-subscription-restrictions)
//! to WaniKani (per our terms and generally being a good citizen).
//!
//! ### Caching
//!
//! Most of the data on WaniKani doesn't change that often, so long-lived caches
//! or more permanent stores that are periodically updated can eliminate a lot
//! of time-consuming requests and help with offline functionality, if that's
//! something you're after.
//!
//! Here are a few recommendations that might influence what you cache and how
//! long you keep it around:
//!
//! - Cache `subject`s as aggressively as possible. They aren't very
//!   frequently updated, and you'll probably need to access them frequently.
//!   They're the object that ties together assignments, review statistics, and
//!   study materials.
//! - Reviews and resets are never changed once recorded, but reviews are
//!   created frequently. You can put these two in long-term storage if you need
//!   them.
//! - Assignments, review statistics, and study materials have moderate levels
//!   of updates. When a user levels up or passes a a subject, there might be a
//!   small flurry of activity with new assignments being created and existing
//!   records being updated. As an assignment gets further and further along in
//!   the SRS stages, those updates will become less and less frequent.
//! - The summary report changes every hour. Caching the results of this request
//!   might help with offline activity, but the data changes, well, every hour.
//! - The user endpoint isn't updated a ton, but when it does, it's going to be
//!   pretty important to capture.
//!
//! Do take note any of the above recommendations may become outdated, but we
//! will try out best to communicate these changes.
//!
//! Caching is always tricky business. When do you expire it? How do you refresh
//! it? Who's in charge of it?
//!
//! We've done a couple things to try and help with a couple of the problems
//! around caching. The first is to support [conditional requests](#conditional-requests),
//! which lets us quickly tell you that a record hasn't changed since you got it
//! last. The second is to give you tools to
//! [get only the updated or new records after any point in time](#leveraging-the-updated_after-filter),
//! letting you easily refresh your local data caches and stores without having
//! to parse all the records.
//!
//! ### Conditional Requests
//!
//! We accept the `If-None-Match` and `If-Modified-Since` headers for every
//! endpoint. If the response body hasn't changed since the last request, then a
//! HTTP status code `304` (Not Modified) and an empty response body is
//! returned. The advantage to using these headers is a faster response time
//! since we don't have to generate a full response; we assume you still have
//! the unmodified data cached.
//!
//! Each response includes the `ETag` and `Last-Modified` headers that are used
//! to populate `If-None-Match` and `If-Modified-Since`, respectively. These
//! values can be used in future requests at the matching endpoint.
//!
//! If both `If-None-Match` and `If-Modified-Since` are passed in, then
//! `If-None-Match` takes precedence.
//!
//! #### If-Modified-Since
//!
//! The `If-Modified-Since` request header takes in a `Last-Modified` value from
//! the last request — or any datetime — in the following format:
//!
//! `If-Modified-Since: <day-name>, <day> <month> <year> <hour>:<minute>:<second> GMT`
//!
//! Where:
//!
//! - `<day-name>` — One of "Mon", "Tue", "Wed", "Thu", "Fri", "Sat", or "Sun"
//!   (case-sensitive).
//! - `<day>` — 2 digit day number, e.g. "04" or "23".
//! - `<month>` — One of "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug",
//!   "Sep", "Oct", "Nov", "Dec" (case sensitive).
//! - `<year>` — 4 digit year number, e.g. "1990" or "2016".
//! - `<hour>` — 2 digit hour number, e.g. "09" or "23".
//! - `<minute>` — 2 digit minute number, e.g. "04" or "59".
//! - `<second>` — 2 digit second number, e.g. "04" or "59".
//! - `GMT` — Greenwich Mean Time. HTTP dates are always expressed in GMT, never
//!   in local time.
//!
//! Example: `If-Modified-Since: Fri, 11 Nov 2011 11:11:11 GMT`
//!
//! The generally-excellent [MDN web docs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/If-Modified-Since)
//! have more information on the `If-Modified-Since` header.
//!
//! #### If-None-Match
//!
//! The `If-None-Match` request header takes in an `ETag` value from the last
//! request's response header:
//!
//! `If-None-Match: <etag_here>`
//!
//! The [MDN web docs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/If-None-Match)
//! have more information on the `If-None-Match` header, too.
//!
//! ### Leveraging the `updated_after` Filter
//!
//! All of the collection endpoints support an `updated_after` filter. As you'd
//! guess, that's going to only return records that have been updated after the
//! timestamp you pass to us.
//!
//! #### Example/Scenario/Not a Fable
//!
//! How does that help with performance and caching? By only returning the
//! records you need.
//!
//! Let's say you're building a [statistics site](https://www.wkstats.com/). You
//! need to know about all the subjects plus get all of a user's assignments,
//! review statistics, reviews, resets, and level progressions to figure out how
//! they've done in the past and do some guesswork on how they might do in the
//! future.
//!
//! Focusing in on the assignments, let's say you decide to re-calculate a
//! user's progress every time they log in to use your site. Without the
//! `updated_after` filter, you'd have to grab all their assignments, since
//! there'd be no way to tell which ones had changed until after you retrieved
//! them all. For high level users, that could be 18 sequential requests! Once
//! you've made them sit through that progress bar, you'd need to parse all the
//! results and compare to them what you've stored locally.
//!
//! With the `updated_after` filter, though, you can ask for only the records
//! that have changed since the last time that user logged in, getting a
//! smaller, faster response full of records you know you have to update or add
//! internally. Even high activity users are only going to touch a small portion
//! of their assignments at a time. We can generate that list of records far
//! more rapidly, it'll be a smaller payload, and you probably won't need to
//! page through results to get everything that you need.
//!
//! ### Respecting Subscription Restrictions
//!
//! WaniKani has [paid subscriptions](https://www.wanikani.com/account/subscription).
//! That may come as a surprise in 2023, but it's true. Those subscriptions
//! grant access to all the content past level three and let people to do
//! lessons and reviews for that content.
//!
//! When the API is used for your own uses (populating spreadsheets, backing up
//! progress, etc.), those access restrictions don't have that much of an
//! impact. Most of the data delivered by the API belongs to you: assignments,
//! study materials, review statistics, and those bits about how you progress
//! through WaniKani. The only data that isn't yours is the content in subjects.
//! All those mnemonics, hints, and relationships have been painstakingly
//! crafted by the WaniKani staff to make learning kanji faster and better.
//! That content is covered by pertinent copyright laws — which also means that
//! fair use allows you to use it to learn on your own.
//!
//! Once you start building tools that can be used by other people, things
//! change, though. First, you can't use the content to build anything that's
//! for profit. Second, you need to respect the limitations put in place by the
//! subscriptions. Both of those requirements are per
//! [our terms](https://www.wanikani.com/terms). So, how do you do meet those
//! requirements?
//!
//! The `user` endpoint has a `subscription` attribute. That should have
//! everything you need.
//!
//! - `max_level_granted` defines the maximum level of content that's available
//!   to the user. It has two possible values: 3 and 60. The user shouldn't be
//!   able to access subjects above those levels. Lessons and reviews above
//!   those levels shouldn't be available at all and will be rejected if you try
//!   and submit them to us.
//! - `active` is a boolean that tells you if the person has an active
//!   subscription.
//! - `type` defines the kind of subscription, and works closely with
//!   `period_ends_at`. There are four values:
//!   - `free` subscriptions aren't really subscriptions, but can represent
//!     people who've never subscribed or have an inactive subscription. There's
//!     no `period_ends_at` for free subscriptions.
//!   - `recurring` subscriptions renew on a periodic basis. `period_ends_at`
//!     tells you when the subscription renews or expires. Since we give people
//!     access until the end of their subscription period even if they cancel,
//!     you can generally not check their subscription status until that time.
//!   - `lifetime` means the user can access WaniKani forever. `period_ends_at`
//!     is null, mainly because `∞` is hard for computers to get. It's possible
//!     that a lifetime user will ask for a refund or have payment difficulties,
//!     so scheduled checks on the subscription status are still needed.
//!   - `unknown` means the user subscription state isn't exactly known. This is
//!     a weird state on WaniKani, should be treated as `free`, and reported to
//!     the WaniKani developers.
//!
//! Your application can use `max_level_granted` as a first, easy line of
//! defense. That restricts content access based on their subscription, and is
//! most of what you need to do. The active, type, and period_ends_at fields are
//! all their to let you build more robust solutions. Those help you figure out
//! when your application needs to check up on subscription status (if ever) or
//! do things like expire access if the user hasn't connected in a while.
//!
//! ## Additional Information
//!
//! ### Spaced Repetition System
//!
//! Our spaced repetition systems determine how subjects progress from
//! being unavailable to users (locked) through complete memorization (burned).
//! The [knowledge guide](https://knowledge.wanikani.com/wanikani/srs-stages/)
//! has some good general information about how we use SRS in WaniKani.
//!
//! A single spaced repetition system consists of `N` number of sequential
//! stages. Each stage describes its position in the sequence as well as the
//! time interval that’s used to determine when the subject will appear next in
//! reviews.
//!
//! Each system has the following common characteristics.
//!
//! Special stage name | Stage position/number | Description
//! -------------------|-----------------------|------------
//! Unlocking stage    | 0                     | This is the initial stage for an assignment; it generally indicates the subject will appear in lessons.
//! Starting stage     | 1                     | The minimum stage for a subject to appear in reviews.
//! Passing stage      | Value from the starting stage position up to the burning stage position | Reaching this milestone counts towards level progression and the unlocking of additional subjects.
//! Burning stage      | N                     | This is the stage when the subject is complete, exits out of reviews and is no longer tested.
//!
//! As mentioned before, we use the SRS stages to calculate the time until the
//! next review (the 'space' in the 'spaced-repetition').
//!
//! - If the review goes well and there are no wrong answers, we move the
//!   assignment up to the next SRS stage. We make the assignment available
//!   'interval' hours from now, at the top of the hour. For example: given an
//!   assignment at stage 1, when we submit a correct answer at 3:31pm, the
//!   assignment would move to SRS stage 2 and become available for another
//!   review at 11:00pm.
//! - If there are wrong answers, we decrease the SRS stage based on the number
//!   of times it was wrong, and then again make it available according to the
//!   interval for that SRS stage.
//!
//! ### User Resets
//!
//! Users have the option to reset their account to a target level at or below
//! their current level.
//!
//! Resets will show up in a variety of places. Explicit records will show up
//! under resets. They'll get a fresh
//! `level_progression` for the target level of the reset,
//! and the level progression for the level they abandoned gets an
//! `abandoned_at` timestamp. Finally, the `assignments` and `review_statistics`
//! for the affected levels will get set back to their default state, waiting to
//! be unlocked or started, depending on the levels.
//!

use std::fmt::{Debug, Display};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error as E;
use url::Url;

/// Convenience type for working with timestamps.
pub type Timestamp = DateTime<Utc>;

/// Expressive type for IDs
pub type Id = u64;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "level_progression")]
pub mod level_progression;

#[cfg(feature = "reset")]
pub mod reset;

#[cfg(feature = "review_statistic")]
pub mod review_statistic;

#[cfg(feature = "study_material")]
pub mod study_material;

#[cfg(feature = "subject")]
pub mod subject;
#[cfg(not(feature = "subject"))]
pub mod subject {
    //! Subjects are the radicals, kanji, vocabulary, and kana_vocabulary that
    //! are learned through lessons and reviews. They contain basic dictionary
    //! information, such as meanings and/or readings, and information about
    //! their relationship to other items with WaniKani, like their level.

    pub use crate::cross_feature::SubjectType;
}

#[cfg(feature = "summary")]
pub mod summary;

#[cfg(feature = "user")]
pub mod user;
#[cfg(not(feature = "user"))]
pub mod user {
    //! The user summary returns basic information for the user making the API
    //! request, identified by their API key.
    pub use crate::cross_feature::LessonPresentationOrder;
}

#[cfg(feature = "voice_actor")]
pub mod voice_actor;
#[cfg(not(feature = "voice_actor"))]
pub mod voice_actor {
    //! Available voice actors used for vocabulary reading pronunciation audio.

    pub use crate::cross_feature::Gender;
}
mod serde_helpers;
mod cross_feature {
    use std::fmt::Display;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    /// The order in which lessons are presented.
    pub enum LessonPresentationOrder {
        #[default]
        /// Lessons are presented in order of level, then by subject `id`.
        AscendingLevelThenSubject,
        /// Lessons are presented in a random order.
        Shuffled,
        /// Lessons are presented in order of level, then randomly.
        AscendingLevelThenShuffled,
    }

    #[cfg(all(feature = "lesson_order_sort"))]
    impl LessonPresentationOrder {
        /// Return an ordering for a pair of subjects according to the selected
        /// presentation order and
        pub fn order_subjects<R: rand::Rng>(
            &self,
            rng: &mut R,
            subject: &crate::subject::SubjectCommon,
            other: &crate::subject::SubjectCommon,
        ) -> std::cmp::Ordering {
            use std::cmp::Ordering;

            match self {
                LessonPresentationOrder::AscendingLevelThenSubject => {
                    match subject.level.cmp(&other.level) {
                        Ordering::Equal => subject.lesson_position.cmp(&other.lesson_position),
                        ord => ord,
                    }
                }
                LessonPresentationOrder::AscendingLevelThenShuffled => {
                    match subject.level.cmp(&other.level) {
                        Ordering::Equal => rng.gen::<u32>().cmp(&rng.gen()),
                        ord => ord,
                    }
                }
                LessonPresentationOrder::Shuffled => rng.gen::<u32>().cmp(&rng.gen()),
            }
        }
    }

    #[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
    #[serde(rename_all = "snake_case")]
    /// Subjects are a subset of resources, learned through lessons and reviews.
    pub enum SubjectType {
        /// A radical.
        Radical,
        /// A kanji.
        Kanji,
        /// A word with kanji in it.
        Vocabulary,
        /// A word with only kana.
        KanaVocabulary,
    }

    impl Display for SubjectType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::KanaVocabulary => write!(f, "kana_vocabulary"),
                Self::Kanji => write!(f, "kanji"),
                Self::Radical => write!(f, "radical"),
                Self::Vocabulary => write!(f, "vocabulary"),
            }
        }
    }

    #[cfg(feature = "subject")]
    #[derive(Debug, Clone, Copy)]
    /// Not all resources are subjects, so this will be returned as an error if
    /// an attempted conversion fails.
    pub struct ConversionError(crate::ResourceType);

    #[cfg(feature = "subject")]
    impl Display for ConversionError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?} is not a subject type.", self.0)
        }
    }

    #[cfg(feature = "subject")]
    impl std::error::Error for ConversionError {}

    #[cfg(feature = "subject")]
    impl TryFrom<crate::ResourceType> for SubjectType {
        type Error = ConversionError;
        fn try_from(value: crate::ResourceType) -> Result<Self, Self::Error> {
            match value {
                crate::ResourceType::KanaVocabulary => Ok(Self::KanaVocabulary),
                crate::ResourceType::Kanji => Ok(Self::Kanji),
                crate::ResourceType::Radical => Ok(Self::Radical),
                crate::ResourceType::Vocabulary => Ok(Self::Vocabulary),
                r => Err(ConversionError(r)),
            }
        }
    }

    #[cfg(feature = "subject")]
    impl From<SubjectType> for crate::ResourceType {
        fn from(value: SubjectType) -> Self {
            match value {
                SubjectType::KanaVocabulary => Self::KanaVocabulary,
                SubjectType::Kanji => Self::Kanji,
                SubjectType::Radical => Self::Radical,
                SubjectType::Vocabulary => Self::Vocabulary,
            }
        }
    }

    #[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    #[allow(missing_docs)]
    pub enum Gender {
        Male,
        Female,
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// Possible resource types
pub enum ResourceType {
    /// A `collection`
    Collection,
    #[cfg(feature = "level_progression")]
    /// A `level_progression
    LevelProgression,
    #[cfg(feature = "reset")]
    /// A `reset`
    Reset,
    #[cfg(feature = "review_statistic")]
    /// A `review_statistic`
    ReviewStatistic,
    #[cfg(feature = "study_material")]
    /// A `study_material`
    StudyMaterial,
    #[cfg(feature = "subject")]
    /// A radical.
    Radical,
    #[cfg(feature = "subject")]
    /// A kanji.
    Kanji,
    #[cfg(feature = "subject")]
    /// A word with kanji in it.
    Vocabulary,
    #[cfg(feature = "subject")]
    /// A word with only kana.
    KanaVocabulary,
    #[cfg(feature = "summary")]
    /// A Summary `report`
    Report,
    #[cfg(feature = "user")]
    /// A `user`
    User,
    #[cfg(feature = "voice_actor")]
    /// A `voice_actor`
    VoiceActor,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// Struct that contains fields common to all resources
pub struct ResourceCommon {
    /// An object's type
    pub object: ResourceType,
    /// The URL of the request. For collections, that will contain all the
    /// filters and options you've passed to the API. Resources have a single
    /// URL and don't need to be filtered, so the URL will be the same in both
    /// resource and collection responses.
    pub url: Url,
    /// For collections, this is the timestamp of the most recently updated
    /// resource in the [specified scope](index.html#filters) and is not limited by
    /// pagination. If no resources were returned for the specified scope, then
    /// this will be `null`. For a resource, then this is the last time that
    /// particular resource was updated.
    pub data_updated_at: Option<Timestamp>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// Pagination information for collections
pub struct Pages {
    /// The URL for the next page of resources, if one exists.
    pub next_url: Option<Url>,
    /// The URL for the previous page of resources, if one exists.
    pub previous_url: Option<Url>,
    /// The number of resources per page.
    pub per_page: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// A generic resource type.
///
/// Most resources returned by the WaniKani
pub struct Resource<T> {
    /// The resource's unique ID.
    pub id: Id,
    #[serde(flatten)]
    /// Common resource data.
    pub common: ResourceCommon,
    /// A resource's data
    pub data: T,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// A collection of resources
pub struct Collection<T> {
    #[serde(flatten)]
    /// Common resource data
    pub common: ResourceCommon,
    /// Pagination data for the collection
    pub pages: Pages,
    /// The total count of resources in the collection
    pub total_count: Id,
    /// The collection's data
    pub data: Vec<Resource<T>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
/// The WaniKani API error object
pub struct WanikaniError {
    /// The numeric error code. This is likely going to match the HTTP status
    /// code.
    pub code: i32,
    /// The returned error message, if one was sent by the API
    pub error: Option<String>,
}

impl Display for WanikaniError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.error {
            None => write!(f, "Error code {} received", self.code),
            Some(ref message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for WanikaniError {}

#[derive(Debug, E)]
/// Possible error conditions
pub enum Error {
    #[error("WaniKani error: {0}")]
    /// An error was returned by WaniKani.
    WaniKaniError(#[from] WanikaniError),
    #[cfg(feature = "client")]
    #[error("HTTP client error: {0}")]
    /// There was some error in the HTTP client.
    Client(#[from] reqwest::Error),
    #[error("WaniKani error: {error}. Limit will reset at {reset_time}")]
    /// Rate Limits have been exceeded. Please wait for the limit to reset.
    ///
    /// This enum includes timestamp information for when the rate limit should
    /// reset. This can be used to pause requests until they will start
    /// returning data again.
    ///
    /// ### Example:
    ///
    /// ```rust
    /// # use wanikani_api::{Error, WanikaniError};
    /// # fn do_things() {}
    /// # let error = Error::RateLimit {
    /// #     error: WanikaniError {code: 429, error: None},
    /// #     reset_time: chrono::Utc::now() + chrono::Duration::seconds(10)};
    /// # async {
    /// match error {
    ///     Error::RateLimit{error, reset_time} => {
    ///         let duration = (reset_time - chrono::Utc::now())
    ///             .to_std()
    ///             .expect("Reset time should be relatively short");
    ///         // You may want to add some time onto this duration, to account for any
    ///         // slight time differences between the API server and client
    ///         tokio::time::sleep_until(tokio::time::Instant::now() + duration).await
    ///     },
    /// // ...
    /// #   error => {}
    /// # }
    /// # };
    /// # Ok::<(), Error>(())
    /// ```
    RateLimit {
        /// The error struct returned by WaniKani
        error: WanikaniError,
        /// The time when the rate limit should reset
        reset_time: Timestamp,
    },
}

/// The version of the API supported by this library
pub const API_VERSION: &str = "20170710";

/// The base URL of the WaniKani V2 API
pub const URL_BASE: &str = "https://api.wanikani.com/v2";

/// A convenience module appropriate for glob imports (use wanikani_api::prelude::*;).
pub mod prelude {
    #[cfg(feature = "client")]
    pub use crate::client::WKClient;
    #[cfg(feature = "client")]
    pub use reqwest;

    pub use crate::{Collection, Error, Resource, ResourceType};
}
