use std::time::Duration;

use crate::api_types::*;
use crate::db::models::{DbEvent, Timestamptz};
use crate::extract::ExtractInfo;
use crate::prelude::*;
use crate::util::iso_string_to_datetime;
use futures::StreamExt;
use tokio_stream::StreamExt as TokioStreamExt;
use warp::{reply::json, Filter, Rejection};

pub mod progress_events {
    use std::sync::Arc;

    use crate::prelude::*;
    use tokio::sync::broadcast::{Receiver, Sender};

    #[derive(Debug, Serialize, TypeScriptify)]
    pub struct ProgressReport {
        call_id: String,
        call_desc: String,
        state: Vec<ProgressState>,
    }
    type SharedProgressReport = Arc<ProgressReport>;

    lazy_static::lazy_static! {
        static ref CHAN: (Sender<SharedProgressReport>, Receiver<SharedProgressReport>) = tokio::sync::broadcast::channel(1);
    }
    fn get_sender() -> Sender<SharedProgressReport> {
        CHAN.0.clone()
    }
    pub fn get_receiver() -> Receiver<SharedProgressReport> {
        CHAN.0.subscribe()
    }

    #[derive(Debug)]
    struct StreamingReporter {
        call_id: String,
        call_desc: String,
        sender: Sender<SharedProgressReport>,
    }
    impl ProgressReporter for StreamingReporter {
        fn report(&self, p: Vec<ProgressState>) {
            let report = ProgressReport {
                call_id: self.call_id.clone(),
                call_desc: self.call_desc.clone(),
                state: p,
            };
            self.sender
                .send(Arc::new(report))
                .expect("Could not send progress");
        }
    }

    pub fn new_progress(desc: impl Into<String>) -> Progress {
        let id = libxid::new_generator().new_id().unwrap().encode();
        Progress::root(Arc::new(StreamingReporter {
            call_id: id,
            call_desc: desc.into(),
            sender: get_sender(),
        }))
    }
}

async fn get_known_tags(
    db: DatyBasy,
    _req: Api::get_known_tags::request,
) -> Api::get_known_tags::response {
    let tags = sqlx::query_scalar!("select text from extracted.tags")
        .fetch_all(&db.db)
        .await?;
    Ok(ApiResponse { data: tags })
}

async fn time_range(db: DatyBasy, req: Api::time_range::request) -> Api::time_range::response {
    // println!("handling...");
    // println!("querying...");
    let before = iso_string_to_datetime(&req.before).context("could not parse before date")?;
    let after = iso_string_to_datetime(&req.after).context("could not parse after date")?;

    let progress = progress_events::new_progress("Extracting time range");

    let now = Instant::now();
    let data = db
        .get_extracted_for_time_range(
            &Timestamptz(after),
            &Timestamptz(before),
            req.tag.as_deref(),
            progress,
        )
        .await
        .context("Could not get extracted events")?;

    log::debug!("time-range request took {:?}", now.elapsed());
    Ok(ApiResponse { data })
}

async fn single_event(
    db: DatyBasy,
    req: Api::single_event::request,
) -> Api::single_event::response {
    // println!("handling...");
    // println!("querying...");
    let a = sqlx::query_as!(
        DbEvent,
        r#"select insertion_sequence, id, timestamp_unix_ms as "timestamp_unix_ms: _",
    data_type, duration_ms, data from raw_events.events where id = ?"#,
        req.id
    )
    .fetch_one(&db.db)
    .await?;

    let r = a.deserialize_data();
    let progress = progress_events::new_progress("Single Event");
    let v = match r {
        Ok(raw) => {
            if let Some(data) = raw.extract_info() {
                let (tags, tags_reasons, _iterations) =
                    get_tags_with_reasons(&db, data, progress).await;
                //let (tags, iterations) = get_tags(&db, data);
                Some(SingleExtractedEventWithRaw {
                    id: a.id,
                    timestamp_unix_ms: a.timestamp_unix_ms,
                    duration_ms: a.duration_ms,
                    tags_reasons,
                    tags,
                    raw,
                })
            } else {
                None
            }
        }
        Err(e) => {
            println!("deser of {} error: {:?}", a.id, e);
            // println!("data=||{}", a.data);
            None
        }
    };

    Ok(ApiResponse { data: v })
}

async fn rule_groups(db: DatyBasy) -> Api::rule_groups::response {
    let groups = sqlx::query_as!(
        TagRuleGroup,
        r#"select global_id, data as "data: _" from config.tag_rule_groups"#
    )
    .fetch_all(&db.db)
    .await
    .context("fetching from db")?
    .into_iter()
    .chain(get_default_tag_rule_groups())
    .collect::<Vec<_>>();

    Ok(ApiResponse { data: groups })
}

async fn update_rule_groups(
    db: DatyBasy,
    req: Api::update_rule_groups::request,
) -> Api::update_rule_groups::response {
    // println!("handling...");
    // println!("querying...");

    for g in req {
        sqlx::query!(
            "insert into config.tag_rule_groups (global_id, data) values (?, ?)
                on conflict(global_id) do update set data = excluded.data",
            g.global_id,
            g.data
        )
        .execute(&db.db)
        .await?;
    }

    Ok(ApiResponse { data: () })
}

#[derive(Debug)]
pub struct ErrAsJson {
    err: anyhow::Error,
}
impl warp::reject::Reject for ErrAsJson {}
impl ErrAsJson {
    pub fn to_json(&self) -> warp::reply::Json {
        warp::reply::json(&serde_json::json!({
            "message": format!("{:?}", &self.err)
        }))
    }
}

fn map_error(err: anyhow::Error) -> warp::Rejection {
    warp::reject::custom(ErrAsJson { err })
}

pub fn with_db(
    db: DatyBasy,
) -> impl warp::Filter<Extract = (DatyBasy,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

pub fn api_routes(
    db: DatyBasy,
) -> impl warp::Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone + Send {
    let rule_groups = with_db(db.clone())
        .and(warp::path("rule-groups"))
        .and_then(|db| async { rule_groups(db).await.map(|e| json(&e)).map_err(map_error) });

    let time_range = with_db(db.clone())
        .and(warp::path("time-range"))
        .and(warp::query::<Api::time_range::request>())
        .and_then(|db, query| async {
            time_range(db, query)
                .await
                .map(|e| json(&e))
                .map_err(map_error)
        });

    let get_known_tags = with_db(db.clone())
        .and(warp::path("get-known-tags"))
        .and(warp::query::<Api::get_known_tags::request>())
        .and_then(|db, query| async move {
            get_known_tags(db, query)
                .await
                .map(|e| json(&e))
                .map_err(map_error)
        });
    let single_event = with_db(db.clone())
        .and(warp::path("single-event"))
        .and(warp::query::<Api::single_event::request>())
        .and_then(|db, query| async move {
            single_event(db, query)
                .await
                .map(|e| json(&e))
                .map_err(map_error)
        });

    let update_rule_groups = with_db(db.clone())
        .and(warp::path("update-rule-groups"))
        .and(warp::body::json())
        .and_then(|db, req| async move {
            update_rule_groups(db, req)
                .await
                .map(|e| json(&e))
                .map_err(map_error)
        });

    let progress_events = warp::path("progress-events").and(warp::get()).map(|| {
        let events = tokio_stream::wrappers::BroadcastStream::new(progress_events::get_receiver());
        // filter out and ignore the Lagged() err caused by polling only sometimes
        let events = StreamExt::filter_map(events, |e| {
            futures::future::ready(match e {
                Ok(e) => Some(warp::sse::Event::default().json_data(e)),
                Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(_)) => None,
            })
        });
        let events = events.throttle(Duration::from_millis(250));
        warp::sse::reply(events)
    });

    let filter = warp::get()
        .and(rule_groups)
        .or(time_range)
        .unify()
        .or(get_known_tags)
        .unify()
        .or(single_event)
        .unify()
        .or(update_rule_groups)
        .unify()
        .or(progress_events);

    filter
}
