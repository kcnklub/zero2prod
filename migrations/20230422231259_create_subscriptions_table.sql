-- Add migration script here
CREATE TABLE subscriptions(
    id uuid NOT NULL, 
    PRIMARY KEY (ID),
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    subcribed_at timestamptz NOT NULL
)
