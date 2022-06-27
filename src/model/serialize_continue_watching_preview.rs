use crate::model::deep_links_ext::DeepLinksExt;
use serde::Serialize;
use stremio_core::deep_links::{LibraryDeepLinks, LibraryItemDeepLinks};
use stremio_core::models::continue_watching_preview::ContinueWatchingPreview;
use stremio_core::types::resource::PosterShape;
use url::Url;
use wasm_bindgen::JsValue;

mod model {
    use super::*;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LibraryItemState<'a> {
        pub video_id: Option<&'a String>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LibraryItem<'a> {
        #[serde(rename = "_id")]
        pub id: &'a String,
        pub name: &'a String,
        pub r#type: &'a String,
        pub poster: &'a Option<Url>,
        pub poster_shape: &'a PosterShape,
        pub progress: f64,
        pub deep_links: LibraryItemDeepLinks,
        pub state: LibraryItemState<'a>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ContinueWatchingPreview<'a> {
        pub library_items: Vec<LibraryItem<'a>>,
        pub deep_links: LibraryDeepLinks,
    }
}

pub fn serialize_continue_watching_preview(
    continue_watching_preview: &ContinueWatchingPreview,
) -> JsValue {
    JsValue::from_serde(&model::ContinueWatchingPreview {
        library_items: continue_watching_preview
            .library_items
            .iter()
            .map(|library_item| model::LibraryItem {
                id: &library_item.id,
                name: &library_item.name,
                r#type: &library_item.r#type,
                poster: &library_item.poster,
                poster_shape: if library_item.poster_shape == PosterShape::Landscape {
                    &PosterShape::Square
                } else {
                    &library_item.poster_shape
                },
                progress: if library_item.state.time_offset > 0 && library_item.state.duration > 0 {
                    library_item.state.time_offset as f64 / library_item.state.duration as f64
                } else {
                    0.0
                },
                deep_links: LibraryItemDeepLinks::from(library_item).into_web_deep_links(),
                state: model::LibraryItemState {
                    video_id: library_item.state.video_id.as_ref(),
                },
            })
            .collect::<Vec<_>>(),
        deep_links: LibraryDeepLinks::from(&"continuewatching".to_owned()).into_web_deep_links(),
    })
    .unwrap()
}
