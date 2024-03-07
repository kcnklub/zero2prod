-- Add migration script here
INSERT INTO users (user_id, name, password_hash)
VALUES (
    'f3542cdc-4a01-4335-aa24-50c8e14779b8', 
    'admin', 
    '$argon2id$v=19$m=19456,t=2,p=1$z/hn3nFSOQyH'
    'aEh3ffCPKw$OlauPfNwVUWXRdmYLdsE6SQ8DpVPTGarFVD4A3lWpZ'
)
