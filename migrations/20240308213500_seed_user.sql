-- Add migration script here
--
-- $argon2id$v=19$m=19456,t=2,p=1$pbBl1ll1DDuvGKae7l8PEA$RoS9WfNgXp6lkm9G8DkxvpJ+seQw6o5amOVWxXraQm8
--

INSERT INTO users 
(user_id, name, password_hash)
VALUES
( 
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11', 
    'admin', 
    '$argon2id$v=19$m=19456,t=2,p=1$pbBl1ll1DDuvGKae7l8PEA$RoS9WfNgXp6lkm9G8DkxvpJ+seQw6o5amOVWxXraQm8'
);
