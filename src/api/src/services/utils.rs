pub fn get_event_metadata_path(event_id: &str) -> String {
  format!("{}-event_metadata.json", event_id)
}

pub fn get_event_file_path(event_id: &str, content_type: &str) -> String {
  format!("{}-event_file.{}", event_id, content_type)
}
