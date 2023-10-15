use sqlx::{Postgres, Transaction};

type PgTransaction<'a> = Transaction<'a, Postgres>;

pub mod subscriber;
pub use subscriber::*;

pub mod subscription_confirmation_task;
pub use subscription_confirmation_task::*;

pub mod user;
pub use user::*;

pub mod newsletter_issue;
pub use newsletter_issue::*;

pub mod newsletter_delivery_task;
pub use newsletter_delivery_task::*;

pub mod confirmation_token;
pub use confirmation_token::*;
