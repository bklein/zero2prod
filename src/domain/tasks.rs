use super::NewSubscriber;
use uuid::Uuid;

#[derive(Debug)]
pub struct SubscriptionConfirmationTask {
    pub subscriber_id: Uuid,
    pub subscriber: NewSubscriber,
    pub confirmation_token: String,
}
