use crate::constants::{LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY};
use crate::models::ctx::Ctx;
use crate::runtime::msg::{Action, ActionCtx};
use crate::runtime::{Env, EnvFutureExt, Runtime, RuntimeAction, TryEnvFuture};
use crate::types::api::{APIResult, SuccessResponse};
use crate::types::events::DismissedEventsBucket;
use crate::types::library::{LibraryBucket, LibraryItem};
use crate::types::notifications::NotificationsBucket;
use crate::types::profile::{Auth, AuthKey, GDPRConsent, Profile, User};
use crate::types::search_history::SearchHistoryBucket;
use crate::types::server_urls::ServerUrlsBucket;
use crate::types::streams::StreamsBucket;
use crate::types::True;
use crate::unit_tests::{
    default_fetch_handler, Request, TestEnv, FETCH_HANDLER, NOW, REQUESTS, STORAGE,
};
use chrono::{TimeZone, Utc};
use futures::future;
use std::any::Any;
use stremio_derive::Model;

#[test]
fn actionctx_removefromlibrary() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    fn fetch_handler(request: Request) -> TryEnvFuture<Box<dyn Any + Send>> {
        match request {
            Request {
                url, method, body, ..
            } if url == "https://localhost:8080/api/datastorePut"
                && method == "POST"
                && body == "{\"authKey\":\"auth_key\",\"collection\":\"libraryItem\",\"changes\":[{\"_id\":\"id\",\"name\":\"name\",\"type\":\"type\",\"poster\":null,\"posterShape\":\"poster\",\"removed\":true,\"temp\":false,\"_ctime\":\"2020-01-01T00:00:00Z\",\"_mtime\":\"2020-01-02T00:00:00Z\",\"state\":{\"lastWatched\":null,\"timeWatched\":0,\"timeOffset\":0,\"overallTimeWatched\":0,\"timesWatched\":0,\"flaggedWatched\":0,\"duration\":0,\"video_id\":null,\"watched\":null,\"noNotif\":false},\"behaviorHints\":{\"defaultVideoId\":null,\"featuredVideoId\":null,\"hasScheduledVideos\":false}}]}" =>
            {
                future::ok(Box::new(APIResult::Ok(
                    SuccessResponse { success: True {} },
                )) as Box<dyn Any + Send>).boxed_env()
            }
            _ => default_fetch_handler(request),
        }
    }
    let library_item = LibraryItem {
        id: "id".to_owned(),
        removed: false,
        temp: false,
        ctime: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
        mtime: Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
        state: Default::default(),
        name: "name".to_owned(),
        r#type: "type".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        behavior_hints: Default::default(),
    };
    let library_item_removed = LibraryItem {
        removed: true,
        mtime: Utc.with_ymd_and_hms(2020, 1, 2, 0, 0, 0).unwrap(),
        ..library_item.to_owned()
    };
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    *FETCH_HANDLER.write().unwrap() = Box::new(fetch_handler);
    *NOW.write().unwrap() = Utc.with_ymd_and_hms(2020, 1, 2, 0, 0, 0).unwrap();
    STORAGE.write().unwrap().insert(
        LIBRARY_RECENT_STORAGE_KEY.to_owned(),
        serde_json::to_string(&LibraryBucket::new(
            Some("id".to_owned()),
            vec![library_item.to_owned()],
        ))
        .unwrap(),
    );
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::new(
                Profile {
                    auth: Some(Auth {
                        key: AuthKey("auth_key".to_owned()),
                        user: User {
                            id: "user_id".to_owned(),
                            email: "user_email".to_owned(),
                            fb_id: None,
                            avatar: None,
                            last_modified: TestEnv::now(),
                            date_registered: TestEnv::now(),
                            trakt: None,
                            premium_expire: None,
                            gdpr_consent: GDPRConsent {
                                tos: true,
                                privacy: true,
                                marketing: true,
                                from: Some("tests".to_owned()),
                            },
                        },
                    }),
                    ..Default::default()
                },
                LibraryBucket {
                    uid: Some("id".to_owned()),
                    items: vec![("id".to_owned(), library_item.to_owned())]
                        .into_iter()
                        .collect(),
                },
                StreamsBucket::default(),
                ServerUrlsBucket::new::<TestEnv>(None),
                NotificationsBucket::new::<TestEnv>(None, vec![]),
                SearchHistoryBucket::default(),
                DismissedEventsBucket::default(),
            ),
        },
        vec![],
        1000,
    );
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::RemoveFromLibrary(library_item.id.to_owned())),
        })
    });
    assert_eq!(
        runtime
            .model()
            .unwrap()
            .ctx
            .library
            .items
            .get(&library_item.id),
        Some(&library_item_removed),
        "Library updated successfully in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_RECENT_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<LibraryBucket>(data).unwrap()
                    == LibraryBucket::new(Some("id".to_owned()), vec![library_item_removed])
            }),
        "Library recent slot updated successfully in storage"
    );
    assert!(
        STORAGE.read().unwrap().get(LIBRARY_STORAGE_KEY).is_none(),
        "Library slot updated successfully in storage"
    );
    assert_eq!(
        REQUESTS.read().unwrap().len(),
        1,
        "One request has been sent"
    );
    assert_eq!(
        REQUESTS.read().unwrap().first().unwrap().url.to_owned(),
        "https://localhost:8080/api/datastorePut".to_owned(),
        "datastorePut request has been sent"
    );
}

#[test]
fn actionctx_removefromlibrary_not_added() {
    #[derive(Model, Clone, Default)]
    #[model(TestEnv)]
    struct TestModel {
        ctx: Ctx,
    }
    let library_item = LibraryItem {
        id: "id".to_owned(),
        removed: false,
        temp: false,
        ctime: Some(Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()),
        mtime: Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
        state: Default::default(),
        name: "name".to_owned(),
        r#type: "type".to_owned(),
        poster: None,
        poster_shape: Default::default(),
        behavior_hints: Default::default(),
    };
    let _env_mutex = TestEnv::reset().expect("Should have exclusive lock to TestEnv");
    STORAGE.write().unwrap().insert(
        LIBRARY_RECENT_STORAGE_KEY.to_owned(),
        serde_json::to_string(&LibraryBucket::new(None, vec![library_item.to_owned()])).unwrap(),
    );
    let (runtime, _rx) = Runtime::<TestEnv, _>::new(
        TestModel {
            ctx: Ctx::new(
                Profile::default(),
                LibraryBucket {
                    uid: None,
                    items: vec![("id".to_owned(), library_item.to_owned())]
                        .into_iter()
                        .collect(),
                },
                StreamsBucket::default(),
                ServerUrlsBucket::new::<TestEnv>(None),
                NotificationsBucket::new::<TestEnv>(None, vec![]),
                SearchHistoryBucket::default(),
                DismissedEventsBucket::default(),
            ),
        },
        vec![],
        1000,
    );
    TestEnv::run(|| {
        runtime.dispatch(RuntimeAction {
            field: None,
            action: Action::Ctx(ActionCtx::RemoveFromLibrary("id2".to_owned())),
        })
    });
    assert_eq!(
        runtime
            .model()
            .unwrap()
            .ctx
            .library
            .items
            .get(&library_item.id),
        Some(&library_item),
        "Library not updated in memory"
    );
    assert!(
        STORAGE
            .read()
            .unwrap()
            .get(LIBRARY_RECENT_STORAGE_KEY)
            .map_or(false, |data| {
                serde_json::from_str::<LibraryBucket>(data).unwrap()
                    == LibraryBucket::new(None, vec![library_item])
            }),
        "Library recent slot not updated in storage"
    );
    assert!(
        STORAGE.read().unwrap().get(LIBRARY_STORAGE_KEY).is_none(),
        "Library slot not updated in storage"
    );
    assert!(
        REQUESTS.read().unwrap().is_empty(),
        "No requests have been sent"
    );
}
