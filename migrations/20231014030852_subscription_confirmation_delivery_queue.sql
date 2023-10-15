CREATE TABLE subscription_confirmation_delivery_queue (
  subscriber_id uuid NOT NULL
    REFERENCES subscriptions (id),
  PRIMARY KEY(subscriber_id)
);
