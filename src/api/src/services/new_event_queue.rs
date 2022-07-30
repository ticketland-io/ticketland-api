// use borsh::{BorshSerialize};
// use amqp_helpers::producer::retry_producer::RetryProducer;
// // use ticketland_signdrop::model::NewUser;

// pub struct NewEventQueue {
//   producer: RetryProducer,
//   exchange_name: String,
//   routing_key: String,
// }

// impl NewEventQueue {
//   pub async fn new(
//     rabbitmq_uri: String,
//     exchange_name: String,
//     queue_name: String,
//     routing_key: String,
//     retry_ttl: u16,
//   ) -> Self {
//     let producer = RetryProducer::new(
//       &rabbitmq_uri,
//       &exchange_name,
//       &queue_name,
//       &routing_key,
//       retry_ttl,
//     ).await;

//     Self {
//       producer,
//       exchange_name,
//       routing_key,
//     }
//   }

//   pub async fn on_new_event(&self, sol_address: String) {
//     let msg = NewUser { 
//       sol_address 
//     };

//     self.producer.publish(
//       &self.exchange_name,
//       &self.routing_key,
//       &msg.try_to_vec().unwrap()
//     ).await;
    
//   }
// }
