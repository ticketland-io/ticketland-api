use bolt_client::{Params};
use ticketland_core::{
  actor::neo4j::{create_params},
  error::Error,
};
use std::sync::Arc;
use actix_web::{
  web::{Data},
  HttpResponse,
};
use common_data::{
  helpers::{send_read},
};
use crate::{
  utils::store::Store,
};

pub fn dummy_query() -> (&'static str, Option<Params>) {
  let query = r#"
    MATCH (evt:Event)-[:HAS_SALE]->(s:Sale)-[:HAS_TYPE]->(st:SaleType)
    return evt{
      .*,
      sales: collect(distinct s {.*, saleTypes:  st})
    }
  "#;

  let params = create_params(vec![
  ]);

  (query, params)
}

pub async fn exec(
  store: Data<Store>
) -> Result<HttpResponse, Error> {
  let (query, db_query_params) = dummy_query();

  let result = send_read(
    Arc::clone(&store.neo4j),
    query,
    db_query_params,
  ).await?;

  Ok(HttpResponse::Ok().json(result))
}
